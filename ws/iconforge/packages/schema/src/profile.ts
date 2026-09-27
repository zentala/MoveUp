import { z } from 'zod';
import { SHAPE_TYPES } from './iconSpec.ts';

const num = z.number().finite();

/** Style profile. The model picks geometry; the profile imposes style. */
export const StyleProfileSchema = z.strictObject({
  id: z.string().min(1),
  viewBox: z.tuple([num, num, num.positive(), num.positive()]),
  stroke: z.literal('currentColor'),
  strokeWidth: num.positive(),
  strokeLinecap: z.enum(['round', 'butt', 'square']),
  strokeLinejoin: z.enum(['round', 'miter', 'bevel']),
  fill: z.literal('none'),
  /** Optical margin from the viewBox edge; applies to the visible outline incl. half stroke. */
  margin: num.nonnegative(),
  /** Advisory grid; end points off-grid produce a warning. Curve control points are never quantized. */
  grid: num.positive(),
  /** Minimum gap between unrelated shapes; violations are warnings (intended crossings are common). */
  minGap: num.nonnegative(),
  /** Output precision in decimal places. */
  precision: z.number().int().min(0).max(4),
  limits: z.strictObject({
    maxShapes: z.number().int().positive(),
    maxGroupDepth: z.number().int().positive(),
    /** Allowed raw coordinate range (inclusive), checked before stroke/margin rules. */
    coordMin: num,
    coordMax: num,
  }),
  allowedShapes: z.array(z.enum(SHAPE_TYPES)).min(1),
});
export type StyleProfile = z.infer<typeof StyleProfileSchema>;

export const OUTLINE_24_V1: StyleProfile = {
  id: 'outline-24-v1',
  viewBox: [0, 0, 24, 24],
  stroke: 'currentColor',
  strokeWidth: 1.5,
  strokeLinecap: 'round',
  strokeLinejoin: 'round',
  fill: 'none',
  margin: 2,
  grid: 0.5,
  minGap: 1,
  precision: 2,
  limits: { maxShapes: 64, maxGroupDepth: 3, coordMin: -24, coordMax: 48 },
  allowedShapes: [...SHAPE_TYPES],
};

export const BUILTIN_PROFILES: Readonly<Record<string, StyleProfile>> = {
  [OUTLINE_24_V1.id]: OUTLINE_24_V1,
};
