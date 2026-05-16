/**
 * useDataCatalog.ts — React hook that fetches the static data catalog
 * from the Rust backend via `invoke('get_data_catalog')`.
 *
 * The catalog is static metadata, so we fetch it once per mount.
 * Surfaces a discriminated state: loading → ready | error. Never throws.
 */
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DataCatalog } from "../types/catalog";

export type DataCatalogState =
  | { status: "loading"; data: null }
  | { status: "ready"; data: DataCatalog }
  | { status: "error"; data: null; error: string };

/**
 * Fetches the data catalog from the backend.
 *
 * Fetches once on mount. Returns `loading` until the invoke resolves,
 * then `ready` with the payload, or `error` with a message on failure.
 *
 * @param skip - When `true`, skips the fetch entirely and stays in `loading`.
 *   Used by `CatalogTab` when an explicit `data` prop is provided (mockup mode).
 */
export function useDataCatalog(skip = false): DataCatalogState {
  const [state, setState] = useState<DataCatalogState>({
    status: "loading",
    data: null,
  });

  useEffect(() => {
    if (skip) return;
    let cancelled = false;
    invoke<DataCatalog>("get_data_catalog")
      .then((data) => {
        if (cancelled) return;
        setState({ status: "ready", data });
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        const message = err instanceof Error ? err.message : String(err);
        setState({ status: "error", data: null, error: message });
      });
    return () => {
      cancelled = true;
    };
  }, [skip]);

  return state;
}
