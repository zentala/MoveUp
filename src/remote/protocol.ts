/**
 * protocol.ts — runtime half of the remote message contract (E022-T01).
 *
 * The payload *shapes* are owned by Rust and re-exported from `src/generated/`
 * (ADR 017); this file adds the runtime validation a browser needs, because a
 * type assertion proves nothing about a message that arrived over a socket.
 * The two halves are pinned together by the `Exact<>` assertions at the bottom
 * of each section: rename a field in `remote_protocol.rs` and `pnpm typecheck`
 * fails here.
 *
 * The same file is imported by the relay Worker (T02, via a tsconfig path
 * alias), so relay, desk and phone validate against one schema.
 */
import { z } from "zod";

import type { ClientInfo } from "../generated/ClientInfo";
import type { Command } from "../generated/Command";
import type { CommandResult } from "../generated/CommandResult";
import type { DeskStatus } from "../generated/DeskStatus";
import type { ErrorBody } from "../generated/ErrorBody";
import type { Hello } from "../generated/Hello";
import type { Role } from "../generated/Role";
import type { Welcome } from "../generated/Welcome";

/** Wire version this build speaks. Mirrors `remote_protocol::PROTOCOL_VERSION`. */
export const PROTOCOL_VERSION = 1;

/**
 * WebSocket close codes, mirroring `remote_protocol.rs`.
 * Never write these as literals at a call site.
 */
export const CLOSE_CODES = {
  PROTOCOL_ERROR: 4400,
  UNAUTHENTICATED: 4401,
  UNENTITLED: 4402,
  REVOKED: 4403,
  REPLACED: 4409,
  RATE_LIMITED: 4429,
  SHEDDING: 4503,
} as const;

/** Close codes after which reconnecting is pointless — the credential is dead. */
export const TERMINAL_CLOSE_CODES: readonly number[] = [
  CLOSE_CODES.UNENTITLED,
  CLOSE_CODES.REVOKED,
  CLOSE_CODES.REPLACED,
];

// ─── Payload schemas ────────────────────────────────────────────────────────

const jsonObject = z.record(z.string(), z.unknown());

export const RoleSchema = z.enum(["desk", "viewer"]);
export const ClientInfoSchema = z.object({
  app: z.string().min(1),
  version: z.string().min(1),
});
export const HelloSchema = z.object({
  role: RoleSchema,
  desk_id: z.string().min(1),
  token: z.string().min(1),
  client: ClientInfoSchema,
});
export const WelcomeSchema = z.object({
  role: RoleSchema,
  desk_id: z.string().min(1),
  desk_online: z.boolean(),
  viewer_count: z.number().int().nonnegative(),
  snapshot: jsonObject.nullable(),
  snapshot_ts: z.number().int().nullable(),
});
export const DeskStatusSchema = z.object({
  online: z.boolean(),
  since: z.number().int(),
});
export const ErrorBodySchema = z.object({
  code: z.string().min(1),
  message: z.string(),
});
export const CommandSchema = z.object({
  name: z.string().min(1),
  args: jsonObject,
  // A viewer sends no `viewer_id`; the relay stamps it before forwarding, so
  // absent normalises to null rather than failing.
  viewer_id: z.string().nullable().default(null),
});
export const CommandResultSchema = z.object({
  command_id: z.string().min(1),
  ok: z.boolean(),
  error: ErrorBodySchema.nullable(),
});
/**
 * A `DisplayEvent` stays opaque here on purpose: an event name this build does
 * not know must survive the envelope layer and be dropped by the reducer, not
 * by the transport (forward compatibility).
 */
export const DisplayEventSchema = jsonObject;

// ─── Envelope ───────────────────────────────────────────────────────────────

/** Every message type that carries meaning in v1. */
export const MESSAGE_TYPES = [
  "hello",
  "welcome",
  "event",
  "desk_status",
  "command",
  "command_result",
  "ping",
  "pong",
  "error",
] as const;

export type MessageType = (typeof MESSAGE_TYPES)[number];

const PAYLOAD_SCHEMAS = {
  hello: HelloSchema,
  welcome: WelcomeSchema,
  event: DisplayEventSchema,
  desk_status: DeskStatusSchema,
  command: CommandSchema,
  command_result: CommandResultSchema,
  ping: z.null(),
  pong: z.null(),
  error: ErrorBodySchema,
} as const satisfies Record<MessageType, z.ZodType>;

/** Builds the envelope schema around one payload schema. */
export const envelopeSchema = <T extends z.ZodType>(payload: T) =>
  z.object({
    v: z.literal(PROTOCOL_VERSION),
    type: z.string().min(1),
    id: z.string().min(1),
    ts: z.number().int(),
    payload,
  });

/** Envelope with the payload left unchecked — the first parse of any message. */
const HeaderSchema = z.object({
  v: z.number().int(),
  type: z.string().min(1),
  id: z.string().min(1),
  ts: z.number().int(),
  payload: z.unknown(),
});

export interface Envelope<T> {
  v: number;
  type: string;
  id: string;
  ts: number;
  payload: T;
}

/** A message whose `type` is one this build knows and whose payload validated. */
export type KnownMessage = {
  [K in MessageType]: Envelope<z.infer<(typeof PAYLOAD_SCHEMAS)[K]>> & { type: K };
}[MessageType];

/**
 * Outcome of parsing one inbound frame.
 *
 * `unknown_type` is deliberately not an error: a newer peer may send a type we
 * have no schema for, and the contract says clients ignore it. Callers must
 * still be able to tell "ignored" from "rejected", so the two are separate
 * variants rather than one silent `null`.
 */
export type ParseOutcome =
  | { status: "ok"; message: KnownMessage }
  | { status: "unknown_type"; type: string; envelope: Envelope<unknown> }
  | { status: "error"; error: ErrorBody };

const fail = (code: string, message: string): ParseOutcome => ({
  status: "error",
  error: { code, message },
});

/** Validates one already-JSON-parsed frame against the v1 contract. */
export function parseMessage(raw: unknown): ParseOutcome {
  if (typeof raw !== "object" || raw === null || !("payload" in raw)) {
    return fail("bad_envelope", "message must be an object carrying a payload key");
  }
  const header = HeaderSchema.safeParse(raw);
  if (!header.success) {
    return fail("bad_envelope", header.error.issues[0]?.message ?? "malformed envelope");
  }
  if (header.data.v !== PROTOCOL_VERSION) {
    return fail(
      "unsupported_version",
      `expected protocol v${PROTOCOL_VERSION}, got v${header.data.v}`,
    );
  }
  const type = header.data.type;
  if (!(MESSAGE_TYPES as readonly string[]).includes(type)) {
    return { status: "unknown_type", type, envelope: header.data as Envelope<unknown> };
  }
  const schema = PAYLOAD_SCHEMAS[type as MessageType];
  const payload = schema.safeParse(header.data.payload);
  if (!payload.success) {
    return fail("bad_payload", `${type}: ${payload.error.issues[0]?.message ?? "invalid"}`);
  }
  return {
    status: "ok",
    message: { ...header.data, type, payload: payload.data } as KnownMessage,
  };
}

// ─── Command allowlist, mirroring COMMAND_ALLOWLIST in Rust ─────────────────

interface IntArg {
  name: string;
  min: number;
  max: number;
}
interface StringArg {
  name: string;
  /** Empty means "any profile-name-shaped slug". */
  allowed: readonly string[];
  maxLen: number;
}
export interface CommandSpec {
  name: string;
  ints: readonly IntArg[];
  strings: readonly StringArg[];
  requireOneInt: boolean;
}

export const COMMAND_ALLOWLIST: readonly CommandSpec[] = [
  { name: "ack_alert", ints: [], strings: [], requireOneInt: false },
  {
    name: "set_limits",
    ints: [
      { name: "sit_min", min: 5, max: 240 },
      { name: "stand_min", min: 1, max: 120 },
    ],
    strings: [],
    requireOneInt: true,
  },
  {
    name: "switch_profile",
    ints: [],
    strings: [
      { name: "kind", allowed: ["ergonomic", "communication"], maxLen: 16 },
      { name: "name", allowed: [], maxLen: 32 },
    ],
    requireOneInt: false,
  },
];

export const specFor = (name: string): CommandSpec | undefined =>
  COMMAND_ALLOWLIST.find((s) => s.name === name);

const isSlug = (s: string, maxLen: number) =>
  s.length > 0 && s.length <= maxLen && /^[a-z0-9_-]+$/.test(s);

const bad = (message: string): ErrorBody => ({ code: "bad_args", message });

/**
 * The relay's copy of the desk's argument check. Both run it — the desk never
 * trusts that the relay validated anything (ADR 023).
 */
export function validateCommand(
  name: string,
  args: Record<string, unknown>,
): ErrorBody | null {
  const spec = specFor(name);
  if (!spec) return { code: "unknown_command", message: `no such command: ${name}` };

  for (const key of Object.keys(args)) {
    const known =
      spec.ints.some((a) => a.name === key) || spec.strings.some((a) => a.name === key);
    if (!known) return bad(`unknown argument: ${key}`);
  }

  let intsPresent = 0;
  for (const arg of spec.ints) {
    if (!(arg.name in args)) continue;
    const value = args[arg.name];
    if (typeof value !== "number" || !Number.isInteger(value)) {
      return bad(`${arg.name} must be an integer`);
    }
    if (value < arg.min || value > arg.max) {
      return bad(`${arg.name} must be ${arg.min}..=${arg.max}`);
    }
    intsPresent += 1;
  }
  if (spec.requireOneInt && intsPresent === 0) {
    return bad(`${name} needs at least one argument`);
  }

  for (const arg of spec.strings) {
    const value = args[arg.name];
    if (value === undefined) return bad(`missing argument: ${arg.name}`);
    if (typeof value !== "string") return bad(`${arg.name} must be a string`);
    const ok =
      arg.allowed.length === 0 ? isSlug(value, arg.maxLen) : arg.allowed.includes(value);
    if (!ok) return bad(`${arg.name} has an unacceptable value`);
  }

  return null;
}

// ─── Schema ↔ generated-type alignment ──────────────────────────────────────

/** `true` only when the two types are mutually assignable. */
type Exact<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;

/**
 * Compile-time proof that every zod schema still matches the type ts-rs wrote
 * from the Rust struct. A field renamed on either side turns one of these
 * `false` and `tsc --noEmit` fails.
 */
export const SCHEMAS_MATCH_RUST: [
  Exact<z.infer<typeof RoleSchema>, Role>,
  Exact<z.infer<typeof ClientInfoSchema>, ClientInfo>,
  Exact<z.infer<typeof HelloSchema>, Hello>,
  Exact<z.infer<typeof WelcomeSchema>, Welcome>,
  Exact<z.infer<typeof DeskStatusSchema>, DeskStatus>,
  Exact<z.infer<typeof CommandSchema>, Command>,
  Exact<z.infer<typeof CommandResultSchema>, CommandResult>,
  Exact<z.infer<typeof ErrorBodySchema>, ErrorBody>,
] = [true, true, true, true, true, true, true, true];
