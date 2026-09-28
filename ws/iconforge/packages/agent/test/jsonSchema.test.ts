import { describe, expect, it } from 'vitest';
import { IconSpecSchema } from '@iconforge/schema';
import { iconSpecJsonSchema } from '../src/jsonSchema.ts';
import { validateMiniSchema } from './mini-json-schema.ts';
import { validSpec } from './fixtures.ts';

type Json = Record<string, unknown>;
const schema = iconSpecJsonSchema() as Json;
const accepts = (value: unknown): boolean => validateMiniSchema(schema, value);

const circle = (id: string) => ({ id, type: 'circle', center: [1, 1], radius: 2 });
const nest = (depth: number): Json =>
  depth === 0 ? circle('leaf') : { id: `g${depth}`, type: 'group', children: [nest(depth - 1)] };
const specWith = (shape: unknown) => ({ version: 1, name: 'x', profile: 'outline-24-v1', shapes: [shape] });

/** Every $ref reachable from a node, as def names. */
function refs(node: unknown, out: Set<string> = new Set()): Set<string> {
  if (Array.isArray(node)) node.forEach((n) => refs(n, out));
  else if (node && typeof node === 'object') {
    for (const [k, v] of Object.entries(node)) {
      if (k === '$ref' && typeof v === 'string') out.add(v.replace('#/$defs/', ''));
      else refs(v, out);
    }
  }
  return out;
}

describe('provider-safe IconSpec JSON schema', () => {
  it('has no reference cycles (providers reject recursive schemas)', () => {
    const defs = schema.$defs as Record<string, Json>;
    const visiting = new Set<string>();
    const visit = (name: string): void => {
      expect(visiting.has(name), `cycle through ${name}`).toBe(false);
      visiting.add(name);
      refs(defs[name]).forEach(visit);
      visiting.delete(name);
    };
    refs(schema.properties).forEach(visit);
  });

  it('uses no keywords that strict structured-output modes reject', () => {
    const text = JSON.stringify(schema);
    for (const kw of ['minimum', 'maximum', 'exclusiveMinimum', 'minLength', 'maxLength', 'pattern', '"not"', 'minItems', 'maxItems']) {
      expect(text).not.toContain(kw);
    }
  });

  it('accepts the fixture spec, a path shape and groups nested to depth 3', () => {
    const path = {
      id: 'heart',
      type: 'path',
      start: [12, 20],
      segments: [
        { kind: 'cubic', control1: [4, 14], control2: [6, 5], to: [12, 9] },
        { kind: 'arc', to: [12, 20], radius: 5, clockwise: true },
      ],
      closed: true,
    };
    for (const spec of [validSpec, specWith(path), specWith(nest(3))]) {
      expect(IconSpecSchema.safeParse(spec).success).toBe(true);
      expect(accepts(spec)).toBe(true);
    }
  });

  it('rejects unknown properties and groups deeper than the unrolled depth', () => {
    expect(accepts({ ...validSpec, extra: 'nope' })).toBe(false);
    expect(accepts(specWith({ ...circle('c'), fill: 'red' }))).toBe(false);
    expect(accepts(specWith(nest(4)))).toBe(false);
  });

  it('leaves value constraints to parseIconSpec (negative radius passes the schema, fails zod)', () => {
    const bad = specWith({ ...circle('c'), radius: -1 });
    expect(accepts(bad)).toBe(true);
    expect(IconSpecSchema.safeParse(bad).success).toBe(false);
  });
});
