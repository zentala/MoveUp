/**
 * The TypeScript half of the protocol contract test.
 *
 * It reads the SAME files as `src-tauri/src/remote_protocol_tests.rs`. A field
 * renamed on one side of the wire fails on the other, which is the only reason
 * the fixtures exist.
 */
import { describe, expect, it } from "vitest";

import {
  CLOSE_CODES,
  COMMAND_ALLOWLIST,
  MESSAGE_TYPES,
  PROTOCOL_VERSION,
  parseMessage,
  validateCommand,
} from "./protocol";

/** Named in the epic handoff. An empty glob is a failure, never a pass. */
const MIN_FIXTURES = 12;

const modules = import.meta.glob("../../tests/fixtures/relay-protocol/*.json", {
  eager: true,
  import: "default",
}) as Record<string, unknown>;

const fixtures = Object.entries(modules)
  .map(([path, body]) => [path.split("/").pop() ?? path, body] as const)
  .sort((a, b) => a[0].localeCompare(b[0]));

/** Deep key sort, so "equal" does not depend on property order. */
const canonical = (v: unknown): unknown => {
  if (Array.isArray(v)) return v.map(canonical);
  if (v && typeof v === "object") {
    return Object.fromEntries(
      Object.keys(v as Record<string, unknown>)
        .sort()
        .map((k) => [k, canonical((v as Record<string, unknown>)[k])]),
    );
  }
  return v;
};

describe("relay protocol fixtures", () => {
  it("finds the fixture directory and it is not empty", () => {
    expect(fixtures.length).toBeGreaterThanOrEqual(MIN_FIXTURES);
  });

  it.each(fixtures)("%s parses and round-trips", (_name, body) => {
    const outcome = parseMessage(body);
    expect(outcome.status).toBe("ok");
    if (outcome.status !== "ok") return;
    expect(canonical(outcome.message)).toEqual(canonical(body));
  });

  it("covers every message type that carries a payload shape", () => {
    const seen = new Set(
      fixtures.map(([, body]) => (body as { type: string }).type),
    );
    for (const type of [
      "hello",
      "welcome",
      "event",
      "desk_status",
      "command",
      "command_result",
      "ping",
      "error",
    ]) {
      expect(seen, `no fixture of type ${type}`).toContain(type);
    }
  });

  it("accepts every allowlisted command fixture", () => {
    const commands = fixtures
      .map(([, body]) => body as { type: string; payload: { name: string; args: Record<string, unknown> } })
      .filter((b) => b.type === "command");
    expect(commands.length).toBeGreaterThan(0);
    for (const c of commands) {
      expect(validateCommand(c.payload.name, c.payload.args)).toBeNull();
    }
  });
});

describe("envelope validation", () => {
  const env = (over: Record<string, unknown> = {}) => ({
    v: PROTOCOL_VERSION,
    type: "ping",
    id: "a3f1",
    ts: 1757170000000,
    payload: null,
    ...over,
  });

  it("accepts a null ping payload but requires the key", () => {
    expect(parseMessage(env()).status).toBe("ok");
    const { payload: _dropped, ...withoutPayload } = env();
    const outcome = parseMessage(withoutPayload);
    expect(outcome.status).toBe("error");
    if (outcome.status === "error") expect(outcome.error.code).toBe("bad_envelope");
  });

  it("rejects an unsupported version with a typed error", () => {
    const outcome = parseMessage(env({ v: 2 }));
    expect(outcome.status).toBe("error");
    if (outcome.status === "error") {
      expect(outcome.error.code).toBe("unsupported_version");
      expect(outcome.error.message).toContain("v2");
    }
  });

  it("reports an unknown type as ignorable, not as an error", () => {
    const outcome = parseMessage(env({ type: "tomorrow", payload: {} }));
    expect(outcome.status).toBe("unknown_type");
    if (outcome.status === "unknown_type") expect(outcome.type).toBe("tomorrow");
  });

  it("rejects a payload that does not match its type", () => {
    const outcome = parseMessage(env({ type: "desk_status", payload: { online: "yes" } }));
    expect(outcome.status).toBe("error");
    if (outcome.status === "error") expect(outcome.error.code).toBe("bad_payload");
  });

  it("rejects a non-object frame", () => {
    for (const raw of [null, 42, "ping", []]) {
      expect(parseMessage(raw).status).toBe("error");
    }
  });

  it("keeps an unknown DisplayEvent inside a valid event envelope", () => {
    const outcome = parseMessage(
      env({ type: "event", payload: { event: "desk:invented-later", payload: { x: 1 } } }),
    );
    expect(outcome.status).toBe("ok");
  });

  it("normalises a viewer's command with no viewer_id to null", () => {
    const outcome = parseMessage(
      env({ type: "command", payload: { name: "ack_alert", args: {} } }),
    );
    expect(outcome.status).toBe("ok");
    if (outcome.status === "ok" && outcome.message.type === "command") {
      expect(outcome.message.payload.viewer_id).toBeNull();
    }
  });
});

describe("command allowlist", () => {
  it("holds exactly the three v1 commands", () => {
    expect(COMMAND_ALLOWLIST.map((s) => s.name)).toEqual([
      "ack_alert",
      "set_limits",
      "switch_profile",
    ]);
  });

  it("treats empty args as valid for ack_alert and invalid for set_limits", () => {
    expect(validateCommand("ack_alert", {})).toBeNull();
    expect(validateCommand("set_limits", {})?.code).toBe("bad_args");
  });

  it("enforces the set_limits bounds", () => {
    expect(validateCommand("set_limits", { sit_min: 30 })).toBeNull();
    expect(validateCommand("set_limits", { stand_min: 1 })).toBeNull();
    expect(validateCommand("set_limits", { sit_min: 4 })?.code).toBe("bad_args");
    expect(validateCommand("set_limits", { sit_min: 241 })?.code).toBe("bad_args");
    expect(validateCommand("set_limits", { stand_min: 121 })?.code).toBe("bad_args");
    expect(validateCommand("set_limits", { sit_min: "30" })?.code).toBe("bad_args");
    expect(validateCommand("set_limits", { sit_min: 30.5 })?.code).toBe("bad_args");
  });

  it("checks switch_profile kind and name shape", () => {
    expect(validateCommand("switch_profile", { kind: "communication", name: "gentle" })).toBeNull();
    for (const args of [
      { kind: "ergonomic" },
      { kind: "sideways", name: "gentle" },
      { kind: "ergonomic", name: "Strict" },
      { kind: "ergonomic", name: "" },
      { kind: "ergonomic", name: "../../etc/passwd" },
    ]) {
      expect(validateCommand("switch_profile", args)?.code).toBe("bad_args");
    }
  });

  it("rejects unknown arguments and unknown command names", () => {
    expect(validateCommand("ack_alert", { force: true })?.code).toBe("bad_args");
    expect(validateCommand("save_settings", {})?.code).toBe("unknown_command");
  });
});

describe("constants", () => {
  it("carries the close codes from the protocol table", () => {
    expect(CLOSE_CODES).toEqual({
      PROTOCOL_ERROR: 4400,
      UNAUTHENTICATED: 4401,
      UNENTITLED: 4402,
      REVOKED: 4403,
      REPLACED: 4409,
      RATE_LIMITED: 4429,
      SHEDDING: 4503,
    });
  });

  it("lists nine message types", () => {
    expect(MESSAGE_TYPES).toHaveLength(9);
  });
});
