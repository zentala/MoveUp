/**
 * desks.ts — the desk-side REST routes (E022-T03).
 *
 * `register` is the only unauthenticated one; the rest carry the desk token as
 * a bearer header and are checked by `requireDesk`. Nothing here ever puts a
 * token or a licence key in a URL, a log line or a response beyond the single
 * moment a credential is minted.
 */
import { countDesks, db, findLicense, isActive, sha256Hex } from "../auth/licenses";
import { hashToken, mintToken, verifyToken } from "../auth/tokens";
import type { Env } from "../env";
import { RPC, callRoom, type ClosedReply, type OnlineReply } from "../room/rpc";
import { readJson, str } from "./body";
import { REGISTER_LIMIT, clientIp, take } from "./rate-limit";
import { fail, json } from "./responses";

const MAX_NAME = 64;
const MAX_VERSION = 32;

/** The bearer token, or `null` when the header is absent or not a bearer. */
export function bearer(request: Request): string | null {
  const header = request.headers.get("Authorization") ?? "";
  const [scheme, ...rest] = header.split(" ");
  if (scheme.toLowerCase() !== "bearer") return null;
  const token = rest.join(" ").trim();
  return token.length > 0 ? token : null;
}

/**
 * Guards a desk-only route. Returns `null` when the caller is that desk, and
 * the refusal Response otherwise.
 *
 * A revoked or unentitled desk is answered `401 unauthorized` like any other
 * failure: the difference matters on the socket, where it selects a close code
 * the client uses to decide whether to retry, but telling an unauthenticated
 * REST caller *why* its token failed tells an attacker which desk ids exist.
 */
export async function requireDesk(
  env: Env,
  request: Request,
  deskId: string,
): Promise<Response | null> {
  const token = bearer(request);
  if (token === null) return fail("unauthorized", "a desk token is required", 401);
  const auth = await verifyToken(env, deskId, "desk", token);
  return auth.ok ? null : fail("unauthorized", "that token cannot act for this desk", 401);
}

/** `POST /v1/desks/register` — licence key in, desk credential out. */
export async function register(request: Request, env: Env): Promise<Response> {
  const verdict = await take(env, `register:${clientIp(request)}`, REGISTER_LIMIT);
  if (!verdict.allowed) {
    return fail("rate_limited", "too many registrations from this address", 429);
  }

  const body = await readJson(request);
  const licenseKey = str(body?.license_key, 128);
  if (licenseKey === null) return fail("invalid_license", "license_key is required", 400);
  const deskName = str(body?.desk_name, MAX_NAME) ?? "desk";
  const appVersion = str(body?.app_version, MAX_VERSION) ?? "unknown";

  const keyHash = await sha256Hex(licenseKey);
  const license = await findLicense(env, keyHash);
  if (license === null) return fail("invalid_license", "no such license key", 400);

  const now = Date.now();
  if (!isActive(license, now)) return fail("invalid_license", "this license has expired", 403);

  if ((await countDesks(env, keyHash)) >= license.max_desks) {
    return fail("license_exhausted", `this license allows ${license.max_desks} desk(s)`, 409);
  }

  const deskId = crypto.randomUUID();
  const token = mintToken("desk");
  await db(env)
    .prepare(
      "INSERT INTO desks (desk_id, license_key_hash, token_hash, desk_name, app_version, created_at, last_seen)" +
        " VALUES (?, ?, ?, ?, ?, ?, NULL)",
    )
    .bind(deskId, keyHash, await hashToken(token), deskName, appVersion, now)
    .run();

  return json(
    { desk_id: deskId, desk_token: token, plan: license.plan, expires_at: license.expires_at },
    201,
  );
}

interface ViewerListRow {
  viewer_id: string;
  device_name: string;
  paired_at: number;
  last_seen: number | null;
}

/** `GET /v1/desks/{desk_id}/viewers` — what the Settings list renders. */
export async function listViewers(request: Request, env: Env, deskId: string): Promise<Response> {
  const refused = await requireDesk(env, request, deskId);
  if (refused) return refused;

  const rows = await db(env)
    .prepare(
      "SELECT viewer_id, device_name, paired_at, last_seen FROM viewers" +
        " WHERE desk_id = ? AND revoked_at IS NULL ORDER BY paired_at",
    )
    .bind(deskId)
    .all<ViewerListRow>();

  const { viewer_ids: online } = await callRoom<OnlineReply>(env, deskId, RPC.online);
  const onlineSet = new Set(online);

  return json(
    (rows.results ?? []).map((row) => ({ ...row, online: onlineSet.has(row.viewer_id) })),
  );
}

/** `DELETE /v1/desks/{desk_id}/viewers/{viewer_id}` — revoke one phone. */
export async function revokeViewer(
  request: Request,
  env: Env,
  deskId: string,
  viewerId: string,
): Promise<Response> {
  const refused = await requireDesk(env, request, deskId);
  if (refused) return refused;

  const result = await db(env)
    .prepare("UPDATE viewers SET revoked_at = ? WHERE viewer_id = ? AND desk_id = ? AND revoked_at IS NULL")
    .bind(Date.now(), viewerId, deskId)
    .run();

  if ((result.meta?.changes ?? 0) === 0) {
    return fail("not_found", "no such active viewer on this desk", 404);
  }

  // The row is dead before the socket is: a viewer that reconnects between the
  // two is refused by `verifyToken`, so the close is a courtesy, not the gate.
  await callRoom<ClosedReply>(env, deskId, RPC.revoke, { viewer_id: viewerId });
  return new Response(null, { status: 204 });
}

/** `DELETE /v1/desks/{desk_id}` — the desktop's "disable relay". */
export async function deleteDesk(request: Request, env: Env, deskId: string): Promise<Response> {
  const refused = await requireDesk(env, request, deskId);
  if (refused) return refused;

  // Explicit, in this order: D1's foreign keys are not relied on for deletion,
  // and a half-applied cascade must not leave viewer rows pointing at nothing.
  await db(env).prepare("DELETE FROM viewers WHERE desk_id = ?").bind(deskId).run();
  await db(env).prepare("DELETE FROM desks WHERE desk_id = ?").bind(deskId).run();
  await callRoom<ClosedReply>(env, deskId, RPC.shutdown);
  return new Response(null, { status: 204 });
}
