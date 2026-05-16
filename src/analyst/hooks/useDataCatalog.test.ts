/**
 * useDataCatalog.test.ts — Unit tests for the catalog hook.
 *
 * Verifies:
 * - loading → ready transition with returned data
 * - error path on rejected invoke
 * - invoke is called exactly once on mount
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useDataCatalog } from "./useDataCatalog";
import type { DataCatalog } from "../types/catalog";

const SAMPLE: DataCatalog = {
  generated_at: "2026-05-16T12:00:00Z",
  sources: [
    {
      id: "sensor",
      name: "Sensor",
      kind: "serial",
      location: "COM3",
      retention: "stream",
      fields: [{ name: "mm", type: "u16", description: "distance" }],
      sample_row: null,
      description: "Distance sensor.",
    },
  ],
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
});

describe("useDataCatalog", () => {
  it("starts in loading and transitions to ready with the payload", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(SAMPLE);

    const { result } = renderHook(() => useDataCatalog());

    expect(result.current.status).toBe("loading");
    expect(result.current.data).toBeNull();

    await waitFor(() => expect(result.current.status).toBe("ready"));
    if (result.current.status !== "ready") throw new Error("expected ready");
    expect(result.current.data).toEqual(SAMPLE);
    expect(vi.mocked(invoke)).toHaveBeenCalledTimes(1);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_data_catalog");
  });

  it("transitions to error when invoke rejects", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("backend down"));

    const { result } = renderHook(() => useDataCatalog());

    await waitFor(() => expect(result.current.status).toBe("error"));
    if (result.current.status !== "error") throw new Error("expected error");
    expect(result.current.error).toBe("backend down");
    expect(result.current.data).toBeNull();
  });

  it("coerces non-Error rejection values to a string message", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("plain string");

    const { result } = renderHook(() => useDataCatalog());

    await waitFor(() => expect(result.current.status).toBe("error"));
    if (result.current.status !== "error") throw new Error("expected error");
    expect(result.current.error).toBe("plain string");
  });
});
