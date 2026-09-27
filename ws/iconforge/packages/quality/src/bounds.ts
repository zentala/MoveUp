import type { ArcShape, CurveShape, Point, Shape } from '@iconforge/schema';

/** Axis-aligned bounding box in icon coordinates. */
export interface Bounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

/** Translate then rotate (clockwise, y-down) around the translated origin. */
export interface Transform {
  translate: Point;
  rotationDeg: number;
}

export const IDENTITY_TRANSFORM: Transform = { translate: [0, 0], rotationDeg: 0 };

function transformPoint([x, y]: Point, t: Transform): Point {
  if (t.rotationDeg === 0) return [x + t.translate[0], y + t.translate[1]];
  const rad = (t.rotationDeg * Math.PI) / 180;
  const cos = Math.cos(rad);
  const sin = Math.sin(rad);
  return [x * cos - y * sin + t.translate[0], x * sin + y * cos + t.translate[1]];
}

function boundsFromPoints(points: readonly Point[]): Bounds {
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  for (const [x, y] of points) {
    if (x < minX) minX = x;
    if (x > maxX) maxX = x;
    if (y < minY) minY = y;
    if (y > maxY) maxY = y;
  }
  return { minX, minY, maxX, maxY };
}

function mergeBounds(a: Bounds, b: Bounds): Bounds {
  return {
    minX: Math.min(a.minX, b.minX),
    minY: Math.min(a.minY, b.minY),
    maxX: Math.max(a.maxX, b.maxX),
    maxY: Math.max(a.maxY, b.maxY),
  };
}

function cornersOf(b: Bounds): Point[] {
  return [
    [b.minX, b.minY],
    [b.maxX, b.minY],
    [b.minX, b.maxY],
    [b.maxX, b.maxY],
  ];
}

/** Exact bbox extremes of an arc's centerline: endpoints plus any cardinal angle in the sweep. */
function arcBounds(shape: ArcShape): Bounds {
  const { center, radius, startDeg, sweepDeg } = shape;
  const start = startDeg;
  const end = startDeg + sweepDeg;
  const lo = Math.min(start, end);
  const hi = Math.max(start, end);
  const angles = [start, end];
  const eps = 1e-9;
  for (const base of [0, 90, 180, 270]) {
    for (const k of [-360, 0, 360]) {
      const a = base + k;
      if (a >= lo - eps && a <= hi + eps) angles.push(a);
    }
  }
  const points = angles.map((deg): Point => {
    const rad = (deg * Math.PI) / 180;
    return [center[0] + radius * Math.cos(rad), center[1] + radius * Math.sin(rad)];
  });
  return boundsFromPoints(points);
}

function bezierPoint(p0: Point, p1: Point, p2: Point, p3: Point, t: number): Point {
  const mt = 1 - t;
  const a = mt * mt * mt;
  const b = 3 * mt * mt * t;
  const c = 3 * mt * t * t;
  const d = t * t * t;
  return [a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0], a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1]];
}

/** Exact bbox of a cubic bezier via derivative roots (per-axis quadratic). */
function curveBounds(shape: CurveShape): Bounds {
  const { from, control1, control2, to } = shape;
  const ts = [0, 1];
  for (const axis of [0, 1] as const) {
    const p0 = from[axis];
    const p1 = control1[axis];
    const p2 = control2[axis];
    const p3 = to[axis];
    const d0 = p1 - p0;
    const d1 = p2 - p1;
    const d2 = p3 - p2;
    const a = d0 - 2 * d1 + d2;
    const b = 2 * (d1 - d0);
    const c = d0;
    if (Math.abs(a) < 1e-12) {
      if (Math.abs(b) > 1e-12) {
        const t = -c / b;
        if (t > 0 && t < 1) ts.push(t);
      }
      continue;
    }
    const disc = b * b - 4 * a * c;
    if (disc < 0) continue;
    const sq = Math.sqrt(disc);
    const t1 = (-b + sq) / (2 * a);
    const t2 = (-b - sq) / (2 * a);
    if (t1 > 0 && t1 < 1) ts.push(t1);
    if (t2 > 0 && t2 < 1) ts.push(t2);
  }
  return boundsFromPoints(ts.map((t) => bezierPoint(from, control1, control2, to, t)));
}

/** Bbox of a shape's own geometry, ignoring any transform (used before applying a group's own transform). */
function localBounds(shape: Shape): Bounds {
  switch (shape.type) {
    case 'line':
      return boundsFromPoints([shape.from, shape.to]);
    case 'polyline':
      return boundsFromPoints(shape.points);
    case 'circle':
      return {
        minX: shape.center[0] - shape.radius,
        maxX: shape.center[0] + shape.radius,
        minY: shape.center[1] - shape.radius,
        maxY: shape.center[1] + shape.radius,
      };
    case 'ellipse':
      return {
        minX: shape.center[0] - shape.rx,
        maxX: shape.center[0] + shape.rx,
        minY: shape.center[1] - shape.ry,
        maxY: shape.center[1] + shape.ry,
      };
    case 'rect':
    case 'roundedRect':
      return {
        minX: shape.origin[0],
        minY: shape.origin[1],
        maxX: shape.origin[0] + shape.width,
        maxY: shape.origin[1] + shape.height,
      };
    case 'arc':
      return arcBounds(shape);
    case 'curve':
      return curveBounds(shape);
    case 'group': {
      const t: Transform = { translate: shape.translate ?? [0, 0], rotationDeg: shape.rotationDeg ?? 0 };
      let acc: Bounds | undefined;
      for (const child of shape.children) {
        const b = shapeBounds(child, t);
        acc = acc ? mergeBounds(acc, b) : b;
      }
      // shape.children is non-empty per schema (min(1)), so acc is always set.
      return acc as Bounds;
    }
  }
}

/**
 * Axis-aligned bbox of a shape's centerline geometry in icon coordinates.
 *
 * `parentTransform` is an additional transform applied on top of the shape's own
 * geometry (used to compose ancestor group transforms). For a `group` shape, its
 * children's bboxes are computed recursively and the group's own translate/rotate
 * is applied to those child bboxes' corners, then merged — a conservative
 * approximation (rotating an axis-aligned box can only grow it, never shrink it).
 */
export function shapeBounds(shape: Shape, parentTransform: Transform = IDENTITY_TRANSFORM): Bounds {
  const local = localBounds(shape);
  if (parentTransform.rotationDeg === 0 && parentTransform.translate[0] === 0 && parentTransform.translate[1] === 0) {
    return local;
  }
  const corners = cornersOf(local).map((p) => transformPoint(p, parentTransform));
  return boundsFromPoints(corners);
}
