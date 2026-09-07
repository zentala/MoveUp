/**
 * pairing.ts — the two routes that turn a code into a viewer credential.
 *
 * The code itself never touches D1 (see `room/pairing.ts`); these handlers move
 * between the room, which owns the code and the lockout, and D1, which owns the
 * licence and the device rows.
 */
import { countActiveViewers, db, findLicense, isActive } from "../auth/licenses";
import { findDesk, hashToken, mintToken } from "../auth/tokens";
import type { Env } from "../env";
import { CODE_LENGTH, normaliseCode } from "../room/pairing";
import { RPC, callRoom, type IssuedCode, type RedeemReply } from "../room/rpc";
import { readJson, str } from "./body";
import { requireDesk } from "./desks";
import { PAIR_LIMIT, clientIp, take } from "./rate-limit";
import { fail, json } from "./responses";

const MAX_DEVICE_NAME = 64;

/** `POST /v1/desks/{desk_id}/pairings` — the desk asks for a code to show. */
export async function issuePairing(request: Request, env: Env, deskId: string): Promise<Response> {
  const refused = await requireDesk(env, request, deskId);
  if (refused) return refused;

  const issued = await callRoom<IssuedCode>(env, deskId, RPC.issue);
  return json(issued, 201);
}

const REDEEM_FAILURES: Record<Exclude<RedeemReply["status"], "ok">, number> = {
  bad_code: 400,
  code_expired: 400,
  pairing_locked: 429,
};

/** `POST /v1/pair` — the phone submits a code and receives a viewer token. */
export async function pair(request: Request, env: Env): Promise<Response> {
  const verdict = await take(env, `pair:${clientIp(request)}`, PAIR_LIMIT);
  if (!verdict.allowed) {
    return fail("rate_limited", "too many pairing attempts from this address", 429);
  }

  const body = await readJson(request);
  const deskId = str(body?.desk_id, 64);
  if (deskId === null) return fail("not_found", "desk_id is required", 404);

  // An empty or wrong-length code is refused before the room sees it, so a
  // client with a broken form cannot spend a desk's lockout budget.
  const rawCode = str(body?.code, 32);
  if (rawCode === null || normaliseCode(rawCode).length !== CODE_LENGTH) {
    return fail("bad_code", `a code is ${CODE_LENGTH} characters`, 400);
  }
  const deviceName = str(body?.device_name, MAX_DEVICE_NAME) ?? "phone";

  const desk = await findDesk(env, deskId);
  if (desk === null) return fail("not_found", "no such desk", 404);

  const license = await findLicense(env, desk.license_key_hash);
  if (license === null || !isActive(license, Date.now())) {
    return fail("invalid_license", "this desk's license is not active", 403);
  }
  // Checked before the code is spent: a desk at its viewer limit should be able
  // to reuse the same code once the owner has revoked a device.
  if ((await countActiveViewers(env, deskId)) >= license.max_viewers) {
    return fail("viewer_limit", `this desk allows ${license.max_viewers} paired device(s)`, 409);
  }

  const outcome = await callRoom<RedeemReply>(env, deskId, RPC.redeem, { code: rawCode });
  if (outcome.status !== "ok") {
    const status = REDEEM_FAILURES[outcome.status];
    const suffix =
      outcome.retry_after_ms === undefined
        ? ""
        : ` retry in ${Math.ceil(outcome.retry_after_ms / 1000)}s`;
    return fail(outcome.status, `pairing refused: ${outcome.status}.${suffix}`, status);
  }

  const viewerId = crypto.randomUUID();
  const token = mintToken("viewer");
  await db(env)
    .prepare(
      "INSERT INTO viewers (viewer_id, desk_id, token_hash, device_name, paired_at, last_seen, revoked_at)" +
        " VALUES (?, ?, ?, ?, ?, NULL, NULL)",
    )
    .bind(viewerId, deskId, await hashToken(token), deviceName, Date.now())
    .run();

  return json({ viewer_id: viewerId, viewer_token: token, desk_name: desk.desk_name }, 201);
}
