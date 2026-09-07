/**
 * room-admin.ts — the room's non-protocol duties (E022-T03).
 *
 * Issuing a pairing code, redeeming one, and closing sockets on a revoke are
 * the parts of `DeskRoom` the REST layer drives. They are functions over the
 * socket list and a mutable pairing box, not methods, so each can be tested
 * without a Durable Object and `desk-room.ts` stays a protocol file.
 */
import { findLicense, isActive, sha256Hex } from "../auth/licenses";
import { findDesk } from "../auth/tokens";
import type { Env } from "../env";
import { CLOSE_CODES } from "./messages";
import {
  emptyPairing,
  generateCode,
  issue,
  normaliseCode,
  redeem,
  type PairingLimits,
  type PairingState,
} from "./pairing";
import type { IssuedCode, RedeemReply } from "./rpc";
import { socketsInRole } from "./sockets";

/** A mutable holder, so these functions can replace the room's pairing state. */
export interface PairingBox {
  pairing: PairingState;
  readonly limits: PairingLimits;
}

/** A fresh code, replacing any outstanding one. The plaintext leaves once. */
export async function issueCode(box: PairingBox): Promise<IssuedCode> {
  const code = generateCode();
  box.pairing = issue(box.pairing, await sha256Hex(code), Date.now(), box.limits);
  return { code, expires_at: box.pairing.expires_at };
}

/**
 * Checks a submitted code and consumes it on success.
 *
 * The state is replaced with the decision before this function yields again, so
 * two requests carrying the same code cannot both be told `ok` — the second
 * finds `code_hash: null` and is answered `bad_code`.
 */
export async function redeemCode(box: PairingBox, code: string): Promise<RedeemReply> {
  const hash = await sha256Hex(normaliseCode(code));
  const outcome = redeem(box.pairing, hash, Date.now(), box.limits);
  box.pairing = outcome.next;
  return outcome.retry_after_ms === undefined
    ? { status: outcome.status }
    : { status: outcome.status, retry_after_ms: outcome.retry_after_ms };
}

/** Closes the sockets of a viewer whose row was just revoked. */
export function closeViewer(sockets: WebSocket[], viewerId: string): number {
  let closed = 0;
  for (const { ws, state } of socketsInRole(sockets, "viewer")) {
    if (state.viewerId !== viewerId) continue;
    ws.close(CLOSE_CODES.REVOKED, "revoked");
    closed += 1;
  }
  return closed;
}

/** The desk turned the relay off: every socket goes, pairing state included. */
export function closeAll(box: PairingBox, sockets: WebSocket[]): number {
  box.pairing = emptyPairing();
  let closed = 0;
  for (const ws of sockets) {
    ws.close(CLOSE_CODES.REVOKED, "relay disabled for this desk");
    closed += 1;
  }
  return closed;
}

/**
 * Re-checks the desk's licence against D1. `hello`-time verification alone
 * cannot see a licence that expires under an already-open connection — pings
 * refresh `lastSeen` without ever re-authenticating (security review 2026-09-07,
 * finding High #1). `null` means the check itself failed (D1 unreachable) and
 * must not be treated as "expired" — a transient outage must not close every
 * live desk in the fleet.
 */
export async function isDeskLicenseActive(env: Env, deskId: string): Promise<boolean | null> {
  try {
    const desk = await findDesk(env, deskId);
    if (!desk) return false;
    const license = await findLicense(env, desk.license_key_hash);
    if (!license) return false;
    return isActive(license, Date.now());
  } catch {
    return null;
  }
}

/** Closes every socket in the room because its licence has expired. */
export function closeForExpiredLicense(sockets: WebSocket[]): number {
  let closed = 0;
  for (const ws of sockets) {
    ws.close(CLOSE_CODES.UNENTITLED, "license expired");
    closed += 1;
  }
  return closed;
}

/** Viewer ids with a live socket — the `online` column in the Settings list. */
export function onlineViewers(sockets: WebSocket[]): string[] {
  const ids: string[] = [];
  for (const { state } of socketsInRole(sockets, "viewer")) {
    if (state.viewerId !== null) ids.push(state.viewerId);
  }
  return ids;
}
