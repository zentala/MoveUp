/**
 * storage.ts — the phone's stored pairing record (E022-T08).
 *
 * One `localStorage` key holds everything the viewer needs to reconnect to a
 * paired desk. Every access is wrapped: `localStorage` throws outright in
 * private-mode Safari and in a sandboxed iframe, and a viewer that cannot
 * remember a pairing must still render the LAN path rather than crash.
 *
 * A record that fails validation is treated as absent AND removed — a
 * half-written record from an older build would otherwise make every
 * connection attempt fail in a way that looks like a network fault.
 */
import { z } from "zod";

/** `localStorage` key. Versioned so a v2 record can coexist during migration. */
export const RELAY_STORAGE_KEY = "moveup.relay.v1";

const RelayRecordSchema = z.object({
  relay_url: z.string().min(1),
  desk_id: z.string().min(1),
  viewer_id: z.string().min(1),
  viewer_token: z.string().min(1),
  desk_name: z.string(),
});

/** What `POST /v1/pair` gave us, plus the relay it came from. */
export type RelayRecord = z.infer<typeof RelayRecordSchema>;

/** The `Storage` API, or null where the browser denies it. */
function store(): Storage | null {
  try {
    return globalThis.localStorage ?? null;
  } catch {
    return null;
  }
}

/**
 * Reads the stored pairing.
 *
 * @returns the record, or `null` when there is none, storage is unavailable,
 *   or what is stored does not validate (which also clears it).
 */
export function loadRelayRecord(): RelayRecord | null {
  const storage = store();
  if (!storage) return null;
  let raw: string | null;
  try {
    raw = storage.getItem(RELAY_STORAGE_KEY);
  } catch {
    return null;
  }
  if (!raw) return null;

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    clearRelayRecord();
    return null;
  }
  const result = RelayRecordSchema.safeParse(parsed);
  if (!result.success) {
    clearRelayRecord();
    return null;
  }
  return result.data;
}

/**
 * Persists a pairing.
 *
 * @returns `false` when storage refused the write (quota, private mode) —
 *   the caller can still connect this session, it just will not be remembered.
 */
export function saveRelayRecord(record: RelayRecord): boolean {
  const storage = store();
  if (!storage) return false;
  try {
    storage.setItem(RELAY_STORAGE_KEY, JSON.stringify(record));
    return true;
  } catch {
    return false;
  }
}

/** Forgets the paired desk. Never throws. */
export function clearRelayRecord(): void {
  const storage = store();
  if (!storage) return;
  try {
    storage.removeItem(RELAY_STORAGE_KEY);
  } catch {
    /* nothing to do — the record is already unreachable */
  }
}

/** Whether this phone has been paired with a desk. */
export function hasRelayRecord(): boolean {
  return loadRelayRecord() !== null;
}
