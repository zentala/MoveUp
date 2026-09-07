/**
 * pairing.ts — the phone's half of `POST /v1/pair` (E022-T09).
 *
 * One request, one stored record. Everything that can go wrong on the way
 * comes back as a `PairFailed` carrying the relay's own error code, because
 * the screen renders three of those codes differently — a wrong code clears
 * the field, a lockout tells the user when to come back, and a network fault
 * says nothing about the code at all.
 */
import type { RelayRecord } from "./storage";

/**
 * Relay this build pairs against when nothing else says otherwise.
 * Mirrors `RELAY_DEFAULT_URL` on the desktop side (D5).
 */
export const RELAY_DEFAULT_URL = "https://relay.desk.zentala.io";

/** What a deep link (or the absence of one) says about where to pair. */
export interface PairLink {
  deskId: string;
  code: string;
  /** `?r=` override; null means "use {@link defaultRelayUrl}". */
  relayUrl: string | null;
}

/**
 * Reads `#/pair?d=<desk_id>&c=<code>` — the payload behind the desktop's QR.
 *
 * Missing parameters come back as empty strings rather than an error: an
 * unparameterised `#/pair` is the normal manual-entry path, not a fault.
 */
export function parsePairLink(hash: string): PairLink {
  const query = hash.slice(hash.indexOf("?") + 1);
  const params = new URLSearchParams(hash.includes("?") ? query : "");
  return {
    deskId: params.get("d")?.trim() ?? "",
    code: params.get("c")?.trim() ?? "",
    relayUrl: params.get("r")?.trim() || null,
  };
}

/**
 * Which relay to talk to.
 *
 * A viewer served from the relay itself (`/app/...`) pairs against its own
 * origin — that is the only way a self-hosted or staging relay ever works.
 * Anywhere else (the LAN build at `/display`) there is no local relay, so the
 * default host is the only sensible target.
 */
export function defaultRelayUrl(): string {
  const { origin, pathname } = globalThis.location ?? { origin: "", pathname: "" };
  return pathname.startsWith("/app") && origin ? origin : RELAY_DEFAULT_URL;
}

/** A pairing attempt the relay (or the network) refused. */
export class PairFailed extends Error {
  /** Relay error code: `bad_code`, `code_expired`, `pairing_locked`, … */
  readonly code: string;
  /** Seconds until a retry can succeed, when the relay said so. */
  readonly retryAfterSecs: number | null;

  constructor(code: string, message: string, retryAfterSecs: number | null = null) {
    super(message);
    this.name = "PairFailed";
    this.code = code;
    this.retryAfterSecs = retryAfterSecs;
  }
}

/** Everything `POST /v1/pair` needs. */
export interface PairRequest {
  relayUrl: string;
  deskId: string;
  code: string;
  deviceName: string;
}

interface PairResponse {
  viewer_id?: unknown;
  viewer_token?: unknown;
  desk_name?: unknown;
}

/** Pulls `{error: {code, message, retry_after_secs}}` out of a failed reply. */
async function failureOf(res: Response): Promise<PairFailed> {
  let body: { error?: { code?: unknown; message?: unknown; retry_after_secs?: unknown } };
  try {
    body = (await res.json()) as typeof body;
  } catch {
    return new PairFailed("http_error", `Pairing failed (${res.status})`);
  }
  const error = body.error ?? {};
  const code = typeof error.code === "string" ? error.code : "http_error";
  const message =
    typeof error.message === "string" && error.message
      ? error.message
      : `Pairing failed (${res.status})`;
  const retry =
    typeof error.retry_after_secs === "number" ? error.retry_after_secs : null;
  return new PairFailed(code, message, retry);
}

/**
 * Pairs this phone with a desk.
 *
 * @returns the record to persist, relay URL included — the record has to
 *   carry it, because the next page load reconnects without a deep link.
 * @throws {PairFailed} on any refusal, including a network fault (`network`)
 *   and a 2xx reply missing fields (`bad_response`).
 */
export async function pairViewer(request: PairRequest): Promise<RelayRecord> {
  const relayUrl = request.relayUrl.replace(/\/+$/, "");
  let res: Response;
  try {
    res = await fetch(`${relayUrl}/v1/pair`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        desk_id: request.deskId,
        code: request.code,
        device_name: request.deviceName,
      }),
    });
  } catch {
    throw new PairFailed("network", "Could not reach the relay");
  }

  if (!res.ok) throw await failureOf(res);

  let body: PairResponse;
  try {
    body = (await res.json()) as PairResponse;
  } catch {
    throw new PairFailed("bad_response", "The relay sent an unreadable reply");
  }
  const { viewer_id: viewerId, viewer_token: viewerToken, desk_name: deskName } = body;
  if (typeof viewerId !== "string" || typeof viewerToken !== "string") {
    throw new PairFailed("bad_response", "The relay sent an incomplete reply");
  }

  return {
    relay_url: relayUrl,
    desk_id: request.deskId,
    viewer_id: viewerId,
    viewer_token: viewerToken,
    desk_name: typeof deskName === "string" ? deskName : "",
  };
}

/** A first guess at this phone's name, so the field is never empty. */
export function defaultDeviceName(): string {
  const ua = globalThis.navigator?.userAgent ?? "";
  if (/iPad/i.test(ua)) return "iPad";
  if (/iPhone/i.test(ua)) return "iPhone";
  if (/Android/i.test(ua)) return "Android phone";
  return "Phone";
}
