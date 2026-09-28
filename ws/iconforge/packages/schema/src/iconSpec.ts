import { z } from 'zod';

/** Finite number: rejects NaN and ±Infinity. */
const num = z.number().finite();
export const PointSchema = z.tuple([num, num]).readonly();
export type Point = z.infer<typeof PointSchema>;

const id = z
  .string()
  .min(1)
  .max(64)
  .regex(/^[a-z0-9][a-z0-9-]*$/, 'id must be kebab-case');

const line = z.strictObject({ id, type: z.literal('line'), from: PointSchema, to: PointSchema });
const polyline = z.strictObject({
  id,
  type: z.literal('polyline'),
  points: z.array(PointSchema).min(2).max(64),
  closed: z.boolean().optional(),
});
const circle = z.strictObject({ id, type: z.literal('circle'), center: PointSchema, radius: num.positive() });
const ellipse = z.strictObject({
  id,
  type: z.literal('ellipse'),
  center: PointSchema,
  rx: num.positive(),
  ry: num.positive(),
});
const rect = z.strictObject({
  id,
  type: z.literal('rect'),
  origin: PointSchema,
  width: num.positive(),
  height: num.positive(),
});
const roundedRect = z.strictObject({
  id,
  type: z.literal('roundedRect'),
  origin: PointSchema,
  width: num.positive(),
  height: num.positive(),
  radius: num.nonnegative(),
});
const arc = z.strictObject({
  id,
  type: z.literal('arc'),
  center: PointSchema,
  radius: num.positive(),
  startDeg: num,
  sweepDeg: num.refine((v) => v !== 0 && Math.abs(v) <= 360, '0 < |sweepDeg| <= 360'),
});
const curve = z.strictObject({
  id,
  type: z.literal('curve'),
  from: PointSchema,
  control1: PointSchema,
  control2: PointSchema,
  to: PointSchema,
});

/**
 * One continuous stroke built from segments. Joins between segments use the profile's
 * line join, so connected outlines (a heart, a pulse trace, a seat + backrest) render
 * without the overlapping round caps that separate primitives produce.
 */
const segment = z.discriminatedUnion('kind', [
  z.strictObject({ kind: z.literal('line'), to: PointSchema }),
  z.strictObject({ kind: z.literal('quad'), control: PointSchema, to: PointSchema }),
  z.strictObject({ kind: z.literal('cubic'), control1: PointSchema, control2: PointSchema, to: PointSchema }),
  z.strictObject({
    kind: z.literal('arc'),
    to: PointSchema,
    radius: num.positive(),
    /** SVG large-arc flag. Default false. */
    large: z.boolean().optional(),
    /** Clockwise on screen (SVG sweep-flag 1). Default true. */
    clockwise: z.boolean().optional(),
  }),
]);
export type PathSegment = z.infer<typeof segment>;

const path = z.strictObject({
  id,
  type: z.literal('path'),
  start: PointSchema,
  segments: z.array(segment).min(1).max(64),
  closed: z.boolean().optional(),
});

export type LineShape = z.infer<typeof line>;
export type PathShape = z.infer<typeof path>;
export type PolylineShape = z.infer<typeof polyline>;
export type CircleShape = z.infer<typeof circle>;
export type EllipseShape = z.infer<typeof ellipse>;
export type RectShape = z.infer<typeof rect>;
export type RoundedRectShape = z.infer<typeof roundedRect>;
export type ArcShape = z.infer<typeof arc>;
export type CurveShape = z.infer<typeof curve>;

export type LeafShape =
  | LineShape
  | PolylineShape
  | CircleShape
  | EllipseShape
  | RectShape
  | RoundedRectShape
  | ArcShape
  | CurveShape
  | PathShape;

export type GroupShape = {
  id: string;
  type: 'group';
  children: Shape[];
  translate?: Point;
  /** Rotation around the group origin (after translate), clockwise in SVG space. */
  rotationDeg?: number;
};

export type Shape = LeafShape | GroupShape;

export const ShapeSchema: z.ZodType<Shape> = z.lazy(() =>
  z.discriminatedUnion('type', [
    line,
    polyline,
    circle,
    ellipse,
    rect,
    roundedRect,
    arc,
    curve,
    path,
    z.strictObject({
      id,
      type: z.literal('group'),
      children: z.array(ShapeSchema).min(1),
      translate: PointSchema.optional(),
      rotationDeg: num.optional(),
    }),
  ]),
);

export const IconSpecSchema = z.strictObject({
  version: z.literal(1),
  name: z
    .string()
    .min(1)
    .max(64)
    .regex(/^[a-z0-9][a-z0-9-]*$/, 'name must be kebab-case'),
  profile: z.string().min(1),
  shapes: z.array(ShapeSchema).min(1),
});
export type IconSpec = z.infer<typeof IconSpecSchema>;

export const SHAPE_TYPES = [
  'line',
  'polyline',
  'circle',
  'ellipse',
  'rect',
  'roundedRect',
  'arc',
  'curve',
  'path',
  'group',
] as const;
export type ShapeType = (typeof SHAPE_TYPES)[number];
