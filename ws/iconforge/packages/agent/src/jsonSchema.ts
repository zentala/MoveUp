import { z } from 'zod';
import { IconSpecSchema } from '@iconforge/schema';

/**
 * JSON Schema for IconSpec v1, used as the `response_format.json_schema.schema`
 * for structured-output requests.
 *
 * `IconSpecSchema` is recursive (shapes may contain `group` shapes with nested
 * `children`) via `z.lazy`. If `z.toJSONSchema` cannot convert it directly
 * (some providers reject `$ref`/`$defs` cycles in strict mode), fall back to
 * the hand-written schema below, which is structurally equivalent up to the
 * recursion depth allowed by profile limits (checked by a dedicated test that
 * both schemas accept the same set of sample specs).
 */
export function iconSpecJsonSchema(): object {
  try {
    return z.toJSONSchema(IconSpecSchema, { target: 'draft-7' });
  } catch {
    return HAND_WRITTEN_ICON_SPEC_JSON_SCHEMA;
  }
}

const pointSchema = {
  type: 'array',
  items: [{ type: 'number' }, { type: 'number' }],
  minItems: 2,
  maxItems: 2,
} as const;

const idSchema = {
  type: 'string',
  minLength: 1,
  maxLength: 64,
  pattern: '^[a-z0-9][a-z0-9-]*$',
} as const;

/** Hand-written equivalent of the zod IconSpec schema, for providers that reject $ref cycles. */
export const HAND_WRITTEN_ICON_SPEC_JSON_SCHEMA = {
  $schema: 'http://json-schema.org/draft-07/schema#',
  $ref: '#/$defs/IconSpec',
  $defs: {
    IconSpec: {
      type: 'object',
      additionalProperties: false,
      required: ['version', 'name', 'profile', 'shapes'],
      properties: {
        version: { const: 1 },
        name: { ...idSchema, maxLength: 64 },
        profile: { type: 'string', minLength: 1 },
        shapes: { type: 'array', items: { $ref: '#/$defs/Shape' }, minItems: 1 },
      },
    },
    Shape: {
      anyOf: [
        { $ref: '#/$defs/LineShape' },
        { $ref: '#/$defs/PolylineShape' },
        { $ref: '#/$defs/CircleShape' },
        { $ref: '#/$defs/EllipseShape' },
        { $ref: '#/$defs/RectShape' },
        { $ref: '#/$defs/RoundedRectShape' },
        { $ref: '#/$defs/ArcShape' },
        { $ref: '#/$defs/CurveShape' },
        { $ref: '#/$defs/GroupShape' },
      ],
    },
    LineShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'from', 'to'],
      properties: {
        id: idSchema,
        type: { const: 'line' },
        from: pointSchema,
        to: pointSchema,
      },
    },
    PolylineShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'points'],
      properties: {
        id: idSchema,
        type: { const: 'polyline' },
        points: { type: 'array', items: pointSchema, minItems: 2, maxItems: 64 },
        closed: { type: 'boolean' },
      },
    },
    CircleShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'center', 'radius'],
      properties: {
        id: idSchema,
        type: { const: 'circle' },
        center: pointSchema,
        radius: { type: 'number', exclusiveMinimum: 0 },
      },
    },
    EllipseShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'center', 'rx', 'ry'],
      properties: {
        id: idSchema,
        type: { const: 'ellipse' },
        center: pointSchema,
        rx: { type: 'number', exclusiveMinimum: 0 },
        ry: { type: 'number', exclusiveMinimum: 0 },
      },
    },
    RectShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'origin', 'width', 'height'],
      properties: {
        id: idSchema,
        type: { const: 'rect' },
        origin: pointSchema,
        width: { type: 'number', exclusiveMinimum: 0 },
        height: { type: 'number', exclusiveMinimum: 0 },
      },
    },
    RoundedRectShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'origin', 'width', 'height', 'radius'],
      properties: {
        id: idSchema,
        type: { const: 'roundedRect' },
        origin: pointSchema,
        width: { type: 'number', exclusiveMinimum: 0 },
        height: { type: 'number', exclusiveMinimum: 0 },
        radius: { type: 'number', minimum: 0 },
      },
    },
    ArcShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'center', 'radius', 'startDeg', 'sweepDeg'],
      properties: {
        id: idSchema,
        type: { const: 'arc' },
        center: pointSchema,
        radius: { type: 'number', exclusiveMinimum: 0 },
        startDeg: { type: 'number' },
        sweepDeg: { type: 'number', minimum: -360, maximum: 360, not: { const: 0 } },
      },
    },
    CurveShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'from', 'control1', 'control2', 'to'],
      properties: {
        id: idSchema,
        type: { const: 'curve' },
        from: pointSchema,
        control1: pointSchema,
        control2: pointSchema,
        to: pointSchema,
      },
    },
    GroupShape: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'type', 'children'],
      properties: {
        id: idSchema,
        type: { const: 'group' },
        children: { type: 'array', items: { $ref: '#/$defs/Shape' }, minItems: 1 },
        translate: pointSchema,
        rotationDeg: { type: 'number' },
      },
    },
  },
} as const;
