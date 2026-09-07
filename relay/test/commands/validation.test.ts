/**
 * The relay's copy of the command allowlist.
 *
 * Routing is T04; what T02 owns is the shared validator both ends run
 * (`@app/remote/protocol`), and the fixtures that prove relay, desk and phone
 * agree on the same three commands. The fixture glob asserts a non-zero count:
 * a directory that stopped matching would otherwise turn this file into a
 * suite that checks nothing and passes.
 */
import { describe, expect, it } from "vitest";

import { COMMAND_ALLOWLIST, validateCommand } from "@app/remote/protocol";

const fixtures = import.meta.glob("../../../tests/fixtures/relay-protocol/command-*.json", {
  eager: true,
  import: "default",
}) as Record<string, { type: string; payload: { name: string; args: Record<string, unknown> } }>;

describe("command fixtures", () => {
  it("the fixture directory is not empty", () => {
    expect(Object.keys(fixtures).length).toBeGreaterThan(0);
  });

  it.each(Object.entries(fixtures))("%s validates against the allowlist", (_path, fixture) => {
    if (fixture.type !== "command") return;
    expect(validateCommand(fixture.payload.name, fixture.payload.args)).toBeNull();
  });
});

describe("validateCommand", () => {
  it("declares exactly the three commands of v1", () => {
    expect(COMMAND_ALLOWLIST.map((c) => c.name)).toEqual([
      "ack_alert",
      "set_limits",
      "switch_profile",
    ]);
  });

  it("happy: an in-range set_limits passes", () => {
    expect(validateCommand("set_limits", { sit_min: 30, stand_min: 10 })).toBeNull();
  });

  it("empty: set_limits with no arguments is refused", () => {
    expect(validateCommand("set_limits", {})).toMatchObject({ code: "bad_args" });
  });

  it("empty: ack_alert with no arguments is fine", () => {
    expect(validateCommand("ack_alert", {})).toBeNull();
  });

  it("error: an out-of-range limit is refused", () => {
    expect(validateCommand("set_limits", { sit_min: 4 })).toMatchObject({ code: "bad_args" });
    expect(validateCommand("set_limits", { sit_min: 241 })).toMatchObject({ code: "bad_args" });
  });

  it("error: an unknown command name is refused", () => {
    expect(validateCommand("rm_rf", {})).toMatchObject({ code: "unknown_command" });
  });

  it("error: an unknown argument is refused", () => {
    expect(validateCommand("ack_alert", { force: true })).toMatchObject({ code: "bad_args" });
  });

  it("error: switch_profile rejects a kind outside the two we have", () => {
    expect(validateCommand("switch_profile", { kind: "cosmic", name: "strict" })).toMatchObject({
      code: "bad_args",
    });
  });
});
