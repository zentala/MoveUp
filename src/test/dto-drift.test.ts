/**
 * dto-drift.test.ts — the TypeScript half of the DTO handshake with Rust.
 *
 * Rust owns the wire format. `src-tauri/src/session_dto_fixture_tests.rs`
 * serialises the real structs into `fixtures/session-dto.json`; this test
 * asserts the TypeScript interfaces describe exactly those keys — no field
 * the backend sends is missing, none the backend dropped survives here.
 *
 * The key lists below are typed `Record<keyof T, true>`, so `tsc` fails when
 * a list and its interface disagree, and this test fails when a list and the
 * fixture disagree. Both halves have to move together.
 */
import { describe, it, expect } from "vitest";
import fixtureRaw from "./fixtures/session-dto.json?raw";
import type { SessionStateDto, StateChangedPayload } from "@/types";

const fixture = JSON.parse(fixtureRaw) as {
  session_state_dto: Record<string, unknown>;
  state_changed_payload: Record<string, unknown>;
};

/** Every key of `SessionStateDto` — kept exact by the type annotation. */
const SESSION_STATE_DTO_KEYS: Record<keyof SessionStateDto, true> = {
  state: true,
  sitting_seconds: true,
  standing_seconds: true,
  break_seconds: true,
  session_limit_secs: true,
  stand_limit_secs: true,
  desk_height_cm: true,
  position_changes: true,
  limit_used_secs: true,
  daily_score: true,
  standing_session_secs: true,
  secs_since_last_break: true,
  continuous_computer_secs: true,
  longest_computer_session_secs: true,
  sitting_seconds_total: true,
  idle_secs: true,
  away_bout_secs: true,
  max_continuous_computer_secs: true,
};

/** Every key of `StateChangedPayload` — kept exact by the type annotation. */
const STATE_CHANGED_PAYLOAD_KEYS: Record<keyof StateChangedPayload, true> = {
  state: true,
  standing_seconds: true,
  break_seconds: true,
  desk_height_cm: true,
  position_changes: true,
  last_break_secs: true,
  last_sitting_secs: true,
  break_credit: true,
  limit_used_secs: true,
};

const sorted = (keys: string[]) => [...keys].sort();

describe("DTO drift between Rust and TypeScript", () => {
  it("the fixture is a real payload, not an empty object", () => {
    expect(Object.keys(fixture.session_state_dto).length).toBeGreaterThan(10);
    expect(Object.keys(fixture.state_changed_payload).length).toBeGreaterThan(5);
  });

  it("SessionStateDto declares exactly the keys Rust serialises", () => {
    expect(sorted(Object.keys(SESSION_STATE_DTO_KEYS))).toEqual(
      sorted(Object.keys(fixture.session_state_dto)),
    );
  });

  it("StateChangedPayload declares exactly the keys Rust serialises", () => {
    expect(sorted(Object.keys(STATE_CHANGED_PAYLOAD_KEYS))).toEqual(
      sorted(Object.keys(fixture.state_changed_payload)),
    );
  });

  it("the deleted second counter is gone from both wire types (E015 D2)", () => {
    expect(fixture.session_state_dto).not.toHaveProperty("current_session_secs");
    expect(fixture.state_changed_payload).not.toHaveProperty("current_session_secs");
  });

  it("the credited counter is on both wire types", () => {
    expect(fixture.session_state_dto).toHaveProperty("limit_used_secs");
    expect(fixture.state_changed_payload).toHaveProperty("limit_used_secs");
  });

  it("the fixture parses into the declared TypeScript types", () => {
    const dto = fixture.session_state_dto as unknown as SessionStateDto;
    expect(dto.limit_used_secs).toBe(1260);
    expect(dto.secs_since_last_break).toBe(300);
    // The Debug-only diagnostic is a different number from the credited one.
    expect(dto.secs_since_last_break).not.toBe(dto.limit_used_secs);
  });
});
