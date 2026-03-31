/**
 * registry.test.ts — tests for the widget registry.
 */
import { describe, it, expect } from "vitest";
import { ActiveWidget } from "./registry";
import { OneBarWidget } from "./OneBarWidget";

describe("ActiveWidget", () => {
  it("is the OneBarWidget", () => {
    expect(ActiveWidget).toBe(OneBarWidget);
  });
});
