import type { ArcShape, CurveShape, PathShape, Point, Shape } from '@iconforge/schema';
import { arcAngleBounds, cubicBounds, quadBounds, svgArcToCenter } from './arcMath.ts';

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

export function transformPoint([x, y]: Point, t: Transform): Point {
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
  return arcAngleBounds(shape.center, shape.radius, shape.startDeg, shape.sweepDeg);
}

/** Exact bbox of a cubic bezier via derivative roots (per-axis quadratic). */
function curveBounds(shape: CurveShape): Bounds {
  return cubicBounds(shape.from, shape.control1, shape.control2, shape.to);
}

/**
 * Exact bbox of a `path`'s centerline: walk the segments, tracking the current point,
 * and merge each segment's own exact bbox (line: endpoints; quad/cubic: derivative-root
 * extrema; arc: SVG endpoint form converted to center form, then the same cardinal-angle
 * logic as `arc`).
 */
function pathBounds(shape: PathShape): Bounds {
  let acc = boundsFromPoints([shape.start]);
  let current: Point = shape.start;
  for (const seg of shape.segments) {
    let segBounds: Bounds;
    switch (seg.kind) {
      case 'line':
        segBounds = boundsFromPoints([seg.to]);
        break;
      case 'quad':
        segBounds = quadBounds(current, seg.control, seg.to);
        break;
      case 'cubic':
        segBounds = cubicBounds(current, seg.control1, seg.control2, seg.to);
        break;
      case 'arc': {
        const { center, radius, startDeg, sweepDeg } = svgArcToCenter(
          current,
          seg.to,
          seg.radius,
          seg.large ?? false,
          seg.clockwise ?? true,
        );
        segBounds = arcAngleBounds(center, radius, startDeg, sweepDeg);
        break;
      }
    }
    acc = mergeBounds(acc, segBounds);
    current = seg.to;
  }
  return acc;
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
    case 'path':
      return pathBounds(shape);
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
