import { error as logError } from "@tauri-apps/plugin-log";

const SAFE_ERROR_KINDS = new Set([
  "Error",
  "TypeError",
  "RangeError",
  "ReferenceError",
  "SyntaxError",
  "URIError",
  "EvalError",
  "AggregateError",
  "DOMException",
  "NonError",
  "UnknownError",
]);

type FailureSource = "render" | "window" | "rejection";

type ViewCategory = "widget" | "settings" | "analyst" | "other";
type QueryName = "dashboard" | "today_summary";
type QueryResult = {
  outcome: "not_observed" | "succeeded" | "failed";
  last_result_ms: number | null;
  last_success_ms: number | null;
};

type UiDiagnostics = {
  schema_version: 1;
  boot: "before_app" | "mounted" | "fallback" | "unknown";
  view: ViewCategory;
  queries: Record<QueryName, QueryResult>;
  last_error: {
    source: FailureSource;
    kind: string;
    location: { module: string; line: number | null; column: number | null } | null;
  } | null;
};

declare global {
  interface Window {
    __MOVEUP_UI_DIAGNOSTICS__?: UiDiagnostics;
  }
}

/** Window-local receipt evidence; never a freshness or renderer-health verdict. */
function diagnostics(): UiDiagnostics {
  return (window.__MOVEUP_UI_DIAGNOSTICS__ ??= {
    schema_version: 1,
    boot: "before_app",
    view: "other",
    queries: {
      dashboard: { outcome: "not_observed", last_result_ms: null, last_success_ms: null },
      today_summary: { outcome: "not_observed", last_result_ms: null, last_success_ms: null },
    },
    last_error: null,
  });
}

export function reportAppMounted(view: ViewCategory): void {
  const snapshot = diagnostics();
  // A parent effect must not replace a child's committed error fallback.
  if (snapshot.boot === "fallback") return;
  snapshot.boot = "mounted";
  snapshot.view = view;
}

export function reportView(view: ViewCategory): void {
  diagnostics().view = view;
}

export function reportAppFallback(): void {
  diagnostics().boot = "fallback";
}

export function reportQueryResult(query: QueryName, outcome: "succeeded" | "failed"): void {
  const receivedAt = Date.now();
  // Never publish malformed times or compare the JS clock with native clocks.
  if (!Number.isSafeInteger(receivedAt) || receivedAt < 0 || receivedAt > 8_640_000_000_000_000) return;
  const previous = diagnostics().queries[query];
  diagnostics().queries[query] = {
    outcome,
    last_result_ms: receivedAt,
    last_success_ms: outcome === "succeeded" ? receivedAt : previous.last_success_ms,
  };
}

type SafeLocation = {
  module: string;
  line?: number;
  column?: number;
};

function errorKind(value: unknown): string {
  try {
    if (value instanceof DOMException) return "DOMException";
    if (!(value instanceof Error)) return "NonError";
    return SAFE_ERROR_KINDS.has(value.name) ? value.name : "Error";
  } catch {
    return "UnknownError";
  }
}

function safeCoordinate(value: number): number | undefined {
  return Number.isSafeInteger(value) && value > 0 && value <= 1_000_000_000
    ? value
    : undefined;
}

function safeModuleBasename(value: string): string | undefined {
  try {
    const current = new URL(window.location.href);
    const candidate = new URL(value, current);
    if (
      current.origin === "null" ||
      candidate.origin !== current.origin ||
      candidate.protocol !== current.protocol ||
      candidate.host !== current.host
    ) {
      return undefined;
    }

    const basename = candidate.pathname.split("/").pop() ?? "";
    if (
      !/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}\.(?:c|m)?(?:js|jsx|ts|tsx)$/i.test(
        basename,
      )
    ) {
      return undefined;
    }
    if (!/^\/(?:src|assets)\/.+/i.test(candidate.pathname)) {
      return undefined;
    }
    return basename;
  } catch {
    return undefined;
  }
}

function locationFromUrl(
  file: string,
  line: number,
  column: number,
): SafeLocation | undefined {
  const module = safeModuleBasename(file);
  if (!module) return undefined;

  const safeLine = safeCoordinate(line);
  const safeColumn = safeCoordinate(column);
  return safeLine && safeColumn
    ? { module, line: safeLine, column: safeColumn }
    : { module };
}

function locationFromStack(value: unknown): SafeLocation | undefined {
  try {
    if (!(value instanceof Error) || typeof value.stack !== "string") {
      return undefined;
    }

    const stackLines = value.stack.split(/\r?\n/);
    const message = typeof value.message === "string" ? value.message : "";
    const messageLines = message.split(/\r?\n/);
    const firstMessageLine = messageLines[0] ?? "";
    const header = stackLines[0] ?? "";
    const hasV8Header =
      header === `${value.name}: ${firstMessageLine}` ||
      (message.length === 0 && header === value.name);
    const hasBareHeader = header === firstMessageLine;
    const frames =
      hasV8Header || hasBareHeader
        ? stackLines.slice(messageLines.length)
        : stackLines;

    for (const frame of frames) {
      const match =
        frame.match(
          /^\s*at\s+(?:.+?\s+\()?((?:https?|tauri|file):\/\/[^\s)]+):(\d+):(\d+)\)?\s*$/,
        ) ??
        frame.match(
          /^\s*[^@\s]+@((?:https?|tauri|file):\/\/\S+):(\d+):(\d+)\s*$/,
        );
      if (!match) continue;
      const location = locationFromUrl(
        match[1],
        Number(match[2]),
        Number(match[3]),
      );
      if (location?.line && location.column) return location;
    }
  } catch {
    return undefined;
  }
  return undefined;
}

function writeDiagnostic(
  source: FailureSource,
  value: unknown,
  location?: SafeLocation,
): void {
  const kind = errorKind(value);
  diagnostics().last_error = {
    source,
    kind,
    location: location
      ? { module: location.module, line: location.line ?? null, column: location.column ?? null }
      : null,
  };
  const fields = ["UI_FAILURE", `source=${source}`, `kind=${kind}`];
  if (location) fields.push(`module=${location.module}`);
  if (location?.line && location.column) {
    fields.push(`line=${location.line}`, `column=${location.column}`);
  }

  try {
    void logError(fields.join(" ")).catch(() => undefined);
  } catch {
    // Diagnostics are best-effort; logging failures must not escape into the UI.
  }
}

function locationFromErrorEvent(event: ErrorEvent): SafeLocation | undefined {
  return locationFromStack(event.error);
}

export function reportRenderFailure(error: unknown): void {
  writeDiagnostic("render", error, locationFromStack(error));
}

export function reportImportFailure(error: unknown): void {
  writeDiagnostic("rejection", error, locationFromStack(error));
}

export function startUiDiagnostics(): () => void {
  diagnostics();
  const onError = (event: ErrorEvent): void => {
    writeDiagnostic("window", event.error, locationFromErrorEvent(event));
  };

  const onUnhandledRejection = (event: PromiseRejectionEvent): void => {
    writeDiagnostic(
      "rejection",
      event.reason,
      locationFromStack(event.reason),
    );
  };

  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onUnhandledRejection);

  return () => {
    window.removeEventListener("error", onError);
    window.removeEventListener("unhandledrejection", onUnhandledRejection);
  };
}
