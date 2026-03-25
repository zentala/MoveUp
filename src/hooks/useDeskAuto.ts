/**
 * useDeskAuto.ts — auto-selects useDesk (Tauri) or useRemoteDesk (browser).
 *
 * Checks for `window.__TAURI_INTERNALS__` at module load time to determine
 * the runtime environment. This check is stable and safe for hook rules.
 */
import { useDesk } from "./useDesk";
import { useRemoteDesk } from "./useRemoteDesk";
import type { UseDeskResult } from "./useDeskTypes";

/** Whether we are running inside a Tauri desktop app. */
const isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;

/**
 * Auto-selects the correct desk data hook based on runtime environment.
 * - Tauri desktop app -> useDesk() (IPC)
 * - Browser/kiosk -> useRemoteDesk() (WebSocket)
 */
export function useDeskAuto(): UseDeskResult {
  if (isTauri) {
    return useDesk();
  }
  return useRemoteDesk();
}
