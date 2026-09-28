/**
 * JSON Schema for IconSpec v1 sent as `response_format.json_schema.schema` in
 * structured-output requests.
 *
 * Providers' strict structured-output modes accept only a subset of JSON Schema
 * (verified 2026-09-28 against OpenRouter → Anthropic/Google/Azure, which all
 * returned 400 "Circular reference detected ... __schema0 -> __schema0" for the
 * recursive zod export). So this schema is deliberately:
 *
 * - **acyclic** — groups are unrolled to `maxGroupDepth` levels; the innermost
 *   level allows leaf shapes only;
 * - **structural only** — shape, keys, types and literals. Value constraints
 *   (positive radius, finite numbers, kebab-case ids, sweep range, point arity)
 *   are NOT expressed here; `parseIconSpec` enforces them locally after every
 *   response and the loop retries once with its diagnostics.
 */

type Json = Record<string, unknown>;

export const DEFAULT_SCHEMA_GROUP_DEPTH = 3;

const num = { type: 'number' } as const;
const str = { type: 'string' } as const;
const bool = { type: 'boolean' } as const;
const point = { type: 'array', items: num } as const;

function obj(type: string, required: Record<string, unknown>, optional: Record<string, unknown> = {}): Json {
  return {
    type: 'object',
    additionalProperties: false,
    required: ['id', 'type', ...Object.keys(required)],
    properties: { id: str, type: { const: type }, ...required, ...optional },
  };
}

const segment = {
  anyOf: [
    { type: 'object', additionalProperties: false, required: ['kind', 'to'], properties: { kind: { const: 'line' }, to: point } },
    {
      type: 'object',
      additionalProperties: false,
      required: ['kind', 'control', 'to'],
      properties: { kind: { const: 'quad' }, control: point, to: point },
    },
    {
      type: 'object',
      additionalProperties: false,
      required: ['kind', 'control1', 'control2', 'to'],
      properties: { kind: { const: 'cubic' }, control1: point, control2: point, to: point },
    },
    {
      type: 'object',
      additionalProperties: false,
      required: ['kind', 'to', 'radius'],
      properties: { kind: { const: 'arc' }, to: point, radius: num, large: bool, clockwise: bool },
    },
  ],
};

const LEAF_DEFS: Record<string, Json> = {
  LineShape: obj('line', { from: point, to: point }),
  PolylineShape: obj('polyline', { points: { type: 'array', items: point } }, { closed: bool }),
  CircleShape: obj('circle', { center: point, radius: num }),
  EllipseShape: obj('ellipse', { center: point, rx: num, ry: num }),
  RectShape: obj('rect', { origin: point, width: num, height: num }),
  RoundedRectShape: obj('roundedRect', { origin: point, width: num, height: num, radius: num }),
  ArcShape: obj('arc', { center: point, radius: num, startDeg: num, sweepDeg: num }),
  CurveShape: obj('curve', { from: point, control1: point, control2: point, to: point }),
  PathShape: obj('path', { start: point, segments: { type: 'array', items: segment } }, { closed: bool }),
};

const leafRefs = Object.keys(LEAF_DEFS).map((name) => ({ $ref: `#/$defs/${name}` }));

/** Build the provider-safe IconSpec schema. `maxGroupDepth` mirrors the profile limit. */
export function iconSpecJsonSchema(maxGroupDepth: number = DEFAULT_SCHEMA_GROUP_DEPTH): object {
  const defs: Record<string, Json> = { ...LEAF_DEFS };
  // Shape<d> may contain Group<d>, whose children are Shape<d+1>; Shape<maxDepth+1> is leaves only.
  defs[`Shape${maxGroupDepth + 1}`] = { anyOf: leafRefs };
  for (let d = maxGroupDepth; d >= 1; d--) {
    defs[`Group${d}`] = obj(
      'group',
      { children: { type: 'array', items: { $ref: `#/$defs/Shape${d + 1}` } } },
      { translate: point, rotationDeg: num },
    );
    defs[`Shape${d}`] = { anyOf: [...leafRefs, { $ref: `#/$defs/Group${d}` }] };
  }
  return {
    type: 'object',
    additionalProperties: false,
    required: ['version', 'name', 'profile', 'shapes'],
    properties: {
      version: { const: 1 },
      name: str,
      profile: str,
      shapes: { type: 'array', items: { $ref: '#/$defs/Shape1' } },
    },
    $defs: defs,
  };
}
