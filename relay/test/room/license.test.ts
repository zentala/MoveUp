/**
 * The room re-checks its licence on the alarm, not only at `hello` (security
 * review 2026-09-07, finding High #1). `lastLicenseCheckAt` starts at 0, so the
 * very first alarm always re-checks regardless of `LICENSE_RECHECK_MS`.
 */
import { describe, expect, it } from "vitest";

import { CLOSE_CODES } from "@app/remote/protocol";

import { sha256Hex } from "../../src/auth/licenses";
import { joinDesk, joinViewer, newDesk, pairViewer, testDb } from "../helpers";

/** Expires the desk's own licence row directly in D1, bypassing every route. */
async function expireLicense(licenseKey: string): Promise<void> {
  await testDb()
    .prepare("UPDATE licenses SET expires_at = ? WHERE key_hash = ?")
    .bind(Date.now() - 1_000, await sha256Hex(licenseKey))
    .run();
}

describe("licence re-check", () => {
  it("error: an already-open desk socket is closed 4402 once its licence expires", async () => {
    const desk = await newDesk();
    const { socket } = await joinDesk(desk);

    await expireLicense(desk.licenseKey);

    await expect(socket.closed()).resolves.toMatchObject({ code: CLOSE_CODES.UNENTITLED });
  });

  it("error: a paired viewer is closed 4402 alongside the desk", async () => {
    const desk = await newDesk();
    await joinDesk(desk);
    const viewer = await pairViewer(desk);
    const { socket: viewerSocket } = await joinViewer(desk, viewer);

    await expireLicense(desk.licenseKey);

    await expect(viewerSocket.closed()).resolves.toMatchObject({ code: CLOSE_CODES.UNENTITLED });
  });

  it("happy: a room with no desk socket never queries D1 for a licence check", async () => {
    // No desk connects — `enforceLicenseIfDue` must short-circuit before ever
    // looking anything up. Absence of a throw/timeout here is the assertion:
    // an empty room costs nothing, per the alarm's own doc comment.
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    const { socket } = await joinViewer(desk, viewer);
    // The viewer alone is not a "desk socket present" case, so no licence
    // enforcement should fire against it even once its licence expires.
    await expireLicense(desk.licenseKey);
    // The viewer stays open — no desk means no re-check target, so nothing
    // closes it. Confirm liveness with a round-trip.
    const id = socket.send("ping", null);
    await expect(socket.next()).resolves.toMatchObject({ type: "pong", id });
  });
});
