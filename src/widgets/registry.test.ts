/**
 * registry.test.ts — tests for the widget registry and resolver.
 */
import { describe, it, expect, vi } from "vitest";
import { WIDGET_REGISTRY, DEFAULT_WIDGET_ID, resolveWidget } from "./registry";
import { PlaceholderWidget } from "./PlaceholderWidget";

describe("WIDGET_REGISTRY", () => {
  it("contains at least one widget", () => {
    expect(WIDGET_REGISTRY.length).toBeGreaterThanOrEqual(1);
  });

  it("has unique IDs", () => {
    const ids = WIDGET_REGISTRY.map((w) => w.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every entry has id, name, and component", () => {
    for (const w of WIDGET_REGISTRY) {
      expect(w.id).toBeTruthy();
      expect(w.name).toBeTruthy();
      expect(typeof w.component).toBe("function");
    }
  });
});

describe("DEFAULT_WIDGET_ID", () => {
  it("exists in the registry", () => {
    const found = WIDGET_REGISTRY.find((w) => w.id === DEFAULT_WIDGET_ID);
    expect(found).toBeDefined();
  });
});

describe("resolveWidget", () => {
  it("returns the correct component for a known ID", () => {
    const result = resolveWidget("placeholder");
    expect(result).toBe(PlaceholderWidget);
  });

  it("falls back to default for an unknown ID", () => {
    const spy = vi.spyOn(console, "warn").mockImplementation(() => {});
    const result = resolveWidget("nonexistent-widget-xyz");
    expect(result).toBe(PlaceholderWidget);
    expect(spy).toHaveBeenCalledWith(
      expect.stringContaining("nonexistent-widget-xyz"),
    );
    spy.mockRestore();
  });

  it("falls back to default for empty string", () => {
    const spy = vi.spyOn(console, "warn").mockImplementation(() => {});
    const result = resolveWidget("");
    expect(result).toBe(PlaceholderWidget);
    spy.mockRestore();
  });
});
