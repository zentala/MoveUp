/**
 * useDataCatalog.ts — React hook that fetches the static data catalog
 * from the Rust backend via `invoke('get_data_catalog')`.
 *
 * The catalog is static metadata, so we fetch it once per mount.
 * Surfaces a discriminated state: loading → ready | error, plus a
 * `refetch` function the error UI can wire to a retry button.
 */
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DataCatalog } from "../types/catalog";

interface BaseFields {
  /** Re-trigger the fetch. No-op while `skip` is true. */
  refetch: () => void;
}

type RawCatalogState =
  | { status: "loading"; data: null }
  | { status: "ready"; data: DataCatalog }
  | { status: "error"; data: null; error: string };

export type DataCatalogState = RawCatalogState & BaseFields;

/**
 * Fetches the data catalog from the backend.
 *
 * @param skip - When `true`, skips the fetch entirely and stays in `loading`.
 *   Used by `CatalogTab` when an explicit `data` prop is provided (mockup mode).
 */
export function useDataCatalog(skip = false): DataCatalogState {
  const [nonce, setNonce] = useState(0);
  const [state, setState] = useState<RawCatalogState>({
    status: "loading",
    data: null,
  });

  const refetch = useCallback(() => setNonce((n) => n + 1), []);

  useEffect(() => {
    if (skip) return;
    let cancelled = false;
    setState({ status: "loading", data: null });
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
  }, [skip, nonce]);

  return { ...state, refetch } as DataCatalogState;
}
