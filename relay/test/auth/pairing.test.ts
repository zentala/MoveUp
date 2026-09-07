/**
 * The pairing-code state machine, driven by an injected clock.
 *
 * Expiry is 5 minutes and the lockout 15 in production; asserting either
 * against the wall clock would mean a test that either sleeps or lies, so
 * `redeem` takes `now` and the limits as arguments.
 */
import { describe, expect, it } from "vitest";

import {
  CODE_ALPHABET,
  CODE_LENGTH,
  emptyPairing,
  generateCode,
  issue,
  normaliseCode,
  redeem,
  type PairingLimits,
} from "../../src/room/pairing";

const LIMITS: PairingLimits = { ttlMs: 1_000, lockoutMs: 5_000, maxAttempts: 10 };
const RIGHT = "a".repeat(64);
const WRONG = "b".repeat(64);

describe("generateCode", () => {
  it("is eight characters from the human-readable alphabet", () => {
    for (let i = 0; i < 200; i += 1) {
      const code = generateCode();
      expect(code).toHaveLength(CODE_LENGTH);
      for (const ch of code) expect(CODE_ALPHABET).toContain(ch);
    }
  });

  it("excludes the characters people confuse: I, O, 0, 1", () => {
    for (const ch of "IO01") expect(CODE_ALPHABET).not.toContain(ch);
  });

  it("does not repeat itself across a sample", () => {
    const codes = new Set(Array.from({ length: 200 }, generateCode));
    // 32^8 possibilities; a duplicate in 200 draws would mean a broken source.
    expect(codes.size).toBe(200);
  });
});

describe("normaliseCode", () => {
  it("ignores case, spaces and dashes — a person is reading this off a screen", () => {
    expect(normaliseCode(" ab-cd ef2 ")).toBe("ABCDEF2");
  });
});

describe("redeem", () => {
  it("happy: the right code is accepted and consumed", () => {
    const state = issue(emptyPairing(), RIGHT, 0, LIMITS);
    const first = redeem(state, RIGHT, 100, LIMITS);
    expect(first.status).toBe("ok");
    expect(first.next.code_hash).toBeNull();

    // Hostile QA: the same code a second time. Only one pairing may result.
    expect(redeem(first.next, RIGHT, 101, LIMITS).status).toBe("bad_code");
  });

  it("nil: redeeming when no code was ever issued is a wrong attempt", () => {
    const outcome = redeem(emptyPairing(), RIGHT, 0, LIMITS);
    expect(outcome.status).toBe("bad_code");
    expect(outcome.next.attempts).toBe(1);
  });

  it("error: a code past its ttl is expired, and does not count as an attempt", () => {
    const state = issue(emptyPairing(), RIGHT, 0, LIMITS);
    const outcome = redeem(state, RIGHT, LIMITS.ttlMs, LIMITS);
    expect(outcome.status).toBe("code_expired");
    // A stale QR code must not be able to lock a desk out of pairing.
    expect(outcome.next.attempts).toBe(0);
  });

  it("error: ten wrong codes lock pairing, and the lock states how long", () => {
    let state = issue(emptyPairing(), RIGHT, 0, LIMITS);
    for (let attempt = 1; attempt < LIMITS.maxAttempts; attempt += 1) {
      const outcome = redeem(state, WRONG, 1, LIMITS);
      expect(outcome.status).toBe("bad_code");
      state = outcome.next;
    }
    const locked = redeem(state, WRONG, 1, LIMITS);
    expect(locked.status).toBe("pairing_locked");
    expect(locked.retry_after_ms).toBe(LIMITS.lockoutMs);
    state = locked.next;

    // Even the right code is refused while the lock holds.
    expect(redeem(state, RIGHT, 2, LIMITS).status).toBe("pairing_locked");
    // And a new code from the desk does not clear it.
    const reissued = issue(state, RIGHT, 3, LIMITS);
    expect(redeem(reissued, RIGHT, 4, LIMITS).status).toBe("pairing_locked");
  });

  it("the lock lifts by itself once the window passes", () => {
    let state = issue(emptyPairing(), RIGHT, 0, LIMITS);
    for (let attempt = 0; attempt < LIMITS.maxAttempts; attempt += 1) {
      state = redeem(state, WRONG, 1, LIMITS).next;
    }
    const after = 1 + LIMITS.lockoutMs;
    state = issue(state, RIGHT, after, LIMITS);
    expect(redeem(state, RIGHT, after, LIMITS).status).toBe("ok");
  });

  it("a new code replaces the old one and resets the attempt count", () => {
    let state = redeem(issue(emptyPairing(), RIGHT, 0, LIMITS), WRONG, 1, LIMITS).next;
    expect(state.attempts).toBe(1);
    state = issue(state, WRONG, 2, LIMITS);
    expect(state.attempts).toBe(0);
    expect(redeem(state, RIGHT, 3, LIMITS).status).toBe("bad_code");
  });
});
