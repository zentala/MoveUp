/**
 * body.ts — reading a REST request body without trusting it.
 *
 * Every field the relay stores comes through `str`, which bounds the length as
 * well as the type: `desk_name` and `device_name` are attacker-supplied and end
 * up in a row and on a settings screen.
 */

/** Nothing this API accepts is larger than a few hundred bytes. */
export const MAX_BODY_BYTES = 2_048;

export type Body = Record<string, unknown>;

/**
 * Parses a JSON object body. Returns `null` for anything that is not one —
 * absent, oversize, malformed, or a bare array. The caller decides which error
 * code that means, because "no body" is `invalid_license` on one route and
 * `bad_code` on another.
 */
export async function readJson(request: Request): Promise<Body | null> {
  const declared = Number(request.headers.get("content-length") ?? "0");
  if (Number.isFinite(declared) && declared > MAX_BODY_BYTES) return null;

  let text: string;
  try {
    text = await request.text();
  } catch {
    return null;
  }
  if (text.length === 0 || text.length > MAX_BODY_BYTES) return null;

  try {
    const parsed: unknown = JSON.parse(text);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) return null;
    return parsed as Body;
  } catch {
    return null;
  }
}

/** A trimmed, length-bounded string, or `null` when the field is unusable. */
export function str(value: unknown, maxLen: number): string | null {
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  if (trimmed.length === 0 || trimmed.length > maxLen) return null;
  return trimmed;
}
