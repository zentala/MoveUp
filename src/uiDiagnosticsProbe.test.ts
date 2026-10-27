import { afterEach, expect, it, vi } from "vitest";
import probe from "../src-tauri/src/ui_diagnostics_probe.js?raw";
import { startUiDiagnostics, reportAppMounted, reportAppFallback, reportQueryResult, reportRenderFailure } from "./uiDiagnostics";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn().mockRejectedValue(new Error("sink unavailable")) }));
const stops: Array<() => void> = [];
const collect = () => JSON.parse(JSON.stringify(new Function("window", "document", `return ${probe}`)(window, document)));
afterEach(() => { stops.splice(0).forEach(stop => stop()); delete window.__MOVEUP_UI_DIAGNOSTICS__; document.body.innerHTML = ""; vi.restoreAllMocks(); });

it("collects the actual publisher's two query results without comparing receipt clocks", () => {
  document.body.innerHTML = '<div id="root"><p>Fixture</p></div>';
  stops.push(startUiDiagnostics());
  expect(collect()).toMatchObject({ schema_version: 1, diagnostics_available: true, boot: "before_app", queries: {
    dashboard: { outcome: "not_observed", last_result_ms: null, last_success_ms: null },
    today_summary: { outcome: "not_observed", last_result_ms: null, last_success_ms: null },
  } });
  reportAppMounted("settings");
  const clock = vi.spyOn(Date, "now").mockReturnValue(100);
  reportQueryResult("dashboard", "succeeded");
  clock.mockReturnValue(50);
  reportQueryResult("dashboard", "failed");
  clock.mockReturnValue(200);
  reportQueryResult("today_summary", "succeeded");
  expect(collect()).toMatchObject({ boot: "mounted", view: "settings", root_present: true, root_children: 1, queries: {
    dashboard: { outcome: "failed", last_result_ms: 50, last_success_ms: 100 },
    today_summary: { outcome: "succeeded", last_result_ms: 200, last_success_ms: 200 },
  } });
});

it("keeps the actual sanitized render failure observable after fallback and logging failure", () => {
  stops.push(startUiDiagnostics());
  const error = new TypeError("private fixture message");
  error.stack = `TypeError: private fixture message\n    at render (${new URL("/src/App.tsx", window.location.href).href}:7:2)`;
  reportRenderFailure(error);
  reportAppFallback();
  reportAppMounted("widget");
  const result = collect();
  expect(result).toMatchObject({ boot: "fallback", last_error: {
    source: "render", kind: "TypeError", location: { module: "App.tsx", line: 7, column: 2 },
  } });
  expect(JSON.stringify(result)).not.toContain("private fixture message");
  expect(JSON.stringify(result)).not.toContain("/src/");
});

it("qualifies missing publisher state and rejects actual nonfinite values before JSON", () => {
  delete window.__MOVEUP_UI_DIAGNOSTICS__;
  expect(collect()).toMatchObject({ diagnostics_available: false, boot: "unknown", last_error: null });
  stops.push(startUiDiagnostics());
  window.__MOVEUP_UI_DIAGNOSTICS__!.queries.dashboard.last_result_ms = NaN;
  expect(collect()).toEqual({ collection_error: "invalid_snapshot" });
});
