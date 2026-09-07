/**
 * pairing.ts — the pairing-code state machine (E022-T03).
 *
 * Pure functions over a plain state object, so the lockout and expiry rules are
 * testable against an injected clock rather than a real 15-minute wait. The
 * state itself lives in `DeskRoom` memory and is **never written to D1**: a
 * pairing code is a 40-bit secret with a five-minute life, and storing it would
 * put a credential in a place the protocol says holds only hashes.
 *
 * That memory residence has a consequence worth stating: a Durable Object
 * eviction drops the pending code, and the phone sees `bad_code`. The desk
 * issues another one — cheaper than persisting a live secret.
 */

/** Excludes I, O, 0 and 1: a person reads this off a screen and types it. */
export const CODE_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
export const CODE_LENGTH = 8;

export const DEFAULT_TTL_MS = 300_000;
export const DEFAULT_LOCKOUT_MS = 900_000;
export const DEFAULT_MAX_ATTEMPTS = 10;

export interface PairingState {
  /** SHA-256 of the outstanding code, or `null` when none is outstanding. */
  code_hash: string | null;
  /** Unix ms after which the outstanding code is refused. */
  expires_at: number;
  /** Wrong codes seen since the last lockout or successful pairing. */
  attempts: number;
  /**
   * Unix ms until which redeeming is refused outright. Kept beside the code
   * rather than inside it: a new code must not clear a lockout, or a brute
   * force would only cost the attacker one extra request to the desk.
   */
  locked_until: number;
}

export const emptyPairing = (): PairingState => ({
  code_hash: null,
  expires_at: 0,
  attempts: 0,
  locked_until: 0,
});

export interface PairingLimits {
  ttlMs: number;
  lockoutMs: number;
  maxAttempts: number;
}

export const DEFAULT_LIMITS: PairingLimits = {
  ttlMs: DEFAULT_TTL_MS,
  lockoutMs: DEFAULT_LOCKOUT_MS,
  maxAttempts: DEFAULT_MAX_ATTEMPTS,
};

/** A code, drawn uniformly: 32 divides 256, so the modulo introduces no bias. */
export function generateCode(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(CODE_LENGTH));
  let code = "";
  for (const b of bytes) code += CODE_ALPHABET[b % CODE_ALPHABET.length];
  return code;
}

/** Normalises what a person typed: case and spacing are not part of the secret. */
export const normaliseCode = (raw: string): string =>
  raw.replace(/[\s-]/g, "").toUpperCase();

/**
 * Replaces the outstanding code. Attempts reset — the desk owner asked for this
 * — but an active lockout survives.
 */
export function issue(
  state: PairingState,
  codeHash: string,
  now: number,
  limits: PairingLimits = DEFAULT_LIMITS,
): PairingState {
  return {
    code_hash: codeHash,
    expires_at: now + limits.ttlMs,
    attempts: 0,
    locked_until: state.locked_until,
  };
}

export type RedeemStatus = "ok" | "bad_code" | "code_expired" | "pairing_locked";

export interface RedeemOutcome {
  status: RedeemStatus;
  next: PairingState;
  /** Milliseconds until pairing reopens. Only set for `pairing_locked`. */
  retry_after_ms?: number;
}

/**
 * Checks a submitted code against the outstanding one.
 *
 * An expired code does not count as a wrong attempt: the phone was slow, not
 * hostile, and charging it toward a lockout would let a stale QR code lock a
 * desk out of pairing.
 */
export function redeem(
  state: PairingState,
  codeHash: string,
  now: number,
  limits: PairingLimits = DEFAULT_LIMITS,
): RedeemOutcome {
  if (now < state.locked_until) {
    return {
      status: "pairing_locked",
      next: state,
      retry_after_ms: state.locked_until - now,
    };
  }

  if (state.code_hash !== null && now >= state.expires_at) {
    return { status: "code_expired", next: { ...state, code_hash: null, expires_at: 0 } };
  }

  if (state.code_hash !== null && constantTimeEqual(state.code_hash, codeHash)) {
    return {
      status: "ok",
      next: { code_hash: null, expires_at: 0, attempts: 0, locked_until: state.locked_until },
    };
  }

  const attempts = state.attempts + 1;
  if (attempts >= limits.maxAttempts) {
    return {
      status: "pairing_locked",
      next: {
        code_hash: null,
        expires_at: 0,
        attempts: 0,
        locked_until: now + limits.lockoutMs,
      },
      retry_after_ms: limits.lockoutMs,
    };
  }
  return { status: "bad_code", next: { ...state, attempts } };
}

/** Constant-time comparison of two hex digests. */
export function constantTimeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i += 1) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}
