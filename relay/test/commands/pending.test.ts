/**
 * The in-flight command table (E022-T04).
 *
 * TTL expiry and the size cap are the two behaviours a socket test cannot
 * reach: one needs a clock the room does not expose, the other needs 512
 * unanswered commands. Both are pure functions over a `Map`, so they are
 * exercised here directly — the routing that uses them is tested over real
 * sockets in `routing.test.ts`.
 */
import { describe, expect, it } from "vitest";

import {
  MAX_PENDING,
  PENDING_TTL_MS,
  claim,
  emptyPending,
  remember,
  type CommandBox,
} from "../../src/room/commands";

const box = (): CommandBox => ({ pending: emptyPending() });

describe("pending commands", () => {
  it("happy: a remembered command is claimed by the viewer that sent it", () => {
    const b = box();
    remember(b, "cmd-1", "viewer-a", 1_000);
    expect(claim(b, "cmd-1", 1_500)).toBe("viewer-a");
  });

  it("a claim consumes the entry — a duplicate result finds nothing", () => {
    const b = box();
    remember(b, "cmd-1", "viewer-a", 1_000);
    claim(b, "cmd-1", 1_100);
    expect(claim(b, "cmd-1", 1_200)).toBeNull();
  });

  it("nil: an id that was never sent claims nothing", () => {
    expect(claim(box(), "never", 1_000)).toBeNull();
  });

  it("empty: an entry past its TTL is refused rather than delivered late", () => {
    const b = box();
    remember(b, "cmd-1", "viewer-a", 1_000);
    expect(claim(b, "cmd-1", 1_000 + PENDING_TTL_MS + 1)).toBeNull();
  });

  it("expired entries are swept by the next remember, not left to accumulate", () => {
    const b = box();
    remember(b, "old", "viewer-a", 1_000);
    remember(b, "new", "viewer-b", 1_000 + PENDING_TTL_MS + 1);
    expect(b.pending.has("old")).toBe(false);
    expect(b.pending.size).toBe(1);
  });

  it("error: a desk that never answers cannot grow the table past the cap", () => {
    const b = box();
    for (let i = 0; i < MAX_PENDING + 50; i += 1) {
      // Same instant throughout, so nothing expires and only the cap can
      // bound the map.
      remember(b, `cmd-${i}`, "viewer-a", 1_000);
    }
    expect(b.pending.size).toBe(MAX_PENDING);
    // The oldest went first; the newest is still routable.
    expect(b.pending.has("cmd-0")).toBe(false);
    expect(claim(b, `cmd-${MAX_PENDING + 49}`, 1_100)).toBe("viewer-a");
  });
});
