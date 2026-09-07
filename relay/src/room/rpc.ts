/**
 * rpc.ts — the private channel between the REST routes and a `DeskRoom`.
 *
 * Pairing codes and open sockets live inside the room; licences and device rows
 * live in D1, which only the Worker reads. Three of the six REST routes need
 * both halves, so they call the room over its own `fetch` under a prefix the
 * public router never forwards. Durable Object stubs are not addressable from
 * the internet, so these paths are unreachable from outside the Worker.
 *
 * Keeping the prefix and the payload shapes in one module is the point: the
 * caller and the callee are in different files and would otherwise agree only
 * by memory.
 */
import type { Env } from "../env";
import type { RedeemStatus } from "./pairing";

/** Never forwarded by `index.ts`; only a DO stub can be given these paths. */
export const RPC_PREFIX = "/__room";

export const RPC = {
  /** Desk asks for a fresh pairing code. */
  issue: `${RPC_PREFIX}/pairings`,
  /** Phone submits a code. */
  redeem: `${RPC_PREFIX}/redeem`,
  /** Close one viewer's sockets after its row was revoked. */
  revoke: `${RPC_PREFIX}/revoke`,
  /** Close every socket in the room; the desk turned the relay off. */
  shutdown: `${RPC_PREFIX}/shutdown`,
  /** Which viewers are connected right now — the `online` column in Settings. */
  online: `${RPC_PREFIX}/online`,
} as const;

export interface IssuedCode {
  code: string;
  expires_at: number;
}

export interface RedeemReply {
  status: RedeemStatus;
  retry_after_ms?: number;
}

export interface ClosedReply {
  closed: number;
}

export interface OnlineReply {
  viewer_ids: string[];
}

/** What a room must be able to do for the REST layer. Implemented by `DeskRoom`. */
export interface RoomOps {
  issuePairing(): Promise<IssuedCode>;
  redeemPairing(code: string): Promise<RedeemReply>;
  revokeViewer(viewerId: string): number;
  shutdown(): number;
  onlineViewerIds(): string[];
}

/** Serves an RPC call, or returns `null` when the path is not one. */
export async function handleRoomRpc(
  request: Request,
  pathname: string,
  ops: RoomOps,
): Promise<Response | null> {
  switch (pathname) {
    case RPC.issue:
      return Response.json(await ops.issuePairing());
    case RPC.redeem: {
      const { code } = (await request.json()) as { code: string };
      return Response.json(await ops.redeemPairing(code));
    }
    case RPC.revoke: {
      const { viewer_id: viewerId } = (await request.json()) as { viewer_id: string };
      return Response.json({ closed: ops.revokeViewer(viewerId) } satisfies ClosedReply);
    }
    case RPC.shutdown:
      return Response.json({ closed: ops.shutdown() } satisfies ClosedReply);
    case RPC.online:
      return Response.json({ viewer_ids: ops.onlineViewerIds() } satisfies OnlineReply);
    default:
      return null;
  }
}

/** Calls one room from the Worker. The room is addressed by `desk_id` alone. */
export async function callRoom<T>(
  env: Env,
  deskId: string,
  path: string,
  body: unknown = {},
): Promise<T> {
  const stub = env.DESK_ROOM.get(env.DESK_ROOM.idFromName(deskId));
  const response = await stub.fetch(`https://desk-room.internal${path}`, {
    method: "POST",
    body: JSON.stringify(body),
  });
  return (await response.json()) as T;
}
