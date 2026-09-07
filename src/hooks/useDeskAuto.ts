/**
 * useDeskAuto.ts — auto-selects useDesk (Tauri) or useRemoteDesk (browser).
 *
 * Checks for `window.__TAURI_INTERNALS__` at module load time to determine
 * the runtime environment. This check is stable and safe for hook rules.
 */
import { useDesk } from "./useDesk";
import { useRemoteDesk } from "./useRemoteDesk";
import type { UseDeskConnection } from "./useDeskTypes";

/** Whether we are running inside a Tauri desktop app. */
const isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;

/**
 * Auto-selects the correct desk data hook based on runtime environment.
 * - Tauri desktop app -> useDesk() (IPC)
 * - Browser/kiosk -> useRemoteDesk() (WebSocket)
 */
export function useDeskAuto(): UseDeskConnection {
  // `isTauri` is a module-level constant resolved once at load time (see
  // above) and never changes across renders, so this component's hook count
  // and order are stable for its whole lifetime — the one case where a
  // conditional hook call is safe. Calling both hooks unconditionally would
  // open a real WebSocket connection (useRemoteDesk) even in the Tauri build,
  // or issue Tauri IPC calls (useDesk) in the browser build, which is a
  // behavior change, not a lint fix.
  if (isTauri) {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    return useDesk();
  }
  // eslint-disable-next-line react-hooks/rules-of-hooks
  return useRemoteDesk();
}
