import type { IconSpec, PathShape, Point, Shape } from '@iconforge/schema';
import { transformPoint, type Transform } from './bounds.ts';
import { sampleArcAngle, sampleCubic, sampleQuad, svgArcToCenter } from './arcMath.ts';

/** Chords no longer than this approximate curves/arcs/circles/ellipses as polylines. */
const MAX_CHORD = 0.1;

/**
 * One shape flattened to its centerline polyline(s) in icon (world) coordinates —
 * group `translate`/`rotationDeg` already applied. `closed` marks a loop (circle,
 * ellipse, rect, roundedRect, closed polyline/path): its two array ends are not
 * "open ends" for join checks.
 */
export interface FlattenedContour {
  shapeId: string;
  path: string;
  points: Point[];
  closed: boolean;
}

function sampleCircle(center: Point, radius: number): Point[] {
  const circumference = 2 * Math.PI * radius;
  const steps = Math.max(8, Math.ceil(circumference / MAX_CHORD));
  const pts: Point[] = [];
  for (let i = 0; i <= steps; i++) {
    const t = (2 * Math.PI * i) / steps;
    pts.push([center[0] + radius * Math.cos(t), center[1] + radius * Math.sin(t)]);
  }
  return pts;
}

function sampleEllipse(center: Point, rx: number, ry: number): Point[] {
  const approxCirc = 2 * Math.PI * Math.max(rx, ry);
  const steps = Math.max(8, Math.ceil(approxCirc / MAX_CHORD));
  const pts: Point[] = [];
  for (let i = 0; i <= steps; i++) {
    const t = (2 * Math.PI * i) / steps;
    pts.push([center[0] + rx * Math.cos(t), center[1] + ry * Math.sin(t)]);
  }
  return pts;
}

/** Corners only — sharp-cornered approximation. roundedRect is always closed, so its
 * corner rounding never matters for the open-end join rules this function feeds. */
function sampleRectCorners(origin: Point, width: number, height: number): Point[] {
  const [x, y] = origin;
  return [
    [x, y],
    [x + width, y],
    [x + width, y + height],
    [x, y + height],
  ];
}

function samplePath(shape: PathShape): { points: Point[]; closed: boolean } {
  const points: Point[] = [shape.start];
  let current: Point = shape.start;
  for (const seg of shape.segments) {
    switch (seg.kind) {
      case 'line':
        points.push(seg.to);
        current = seg.to;
        break;
      case 'quad': {
        const pts = sampleQuad(current, seg.control, seg.to, MAX_CHORD);
        points.push(...pts.slice(1));
        current = seg.to;
        break;
      }
      case 'cubic': {
        const pts = sampleCubic(current, seg.control1, seg.control2, seg.to, MAX_CHORD);
        points.push(...pts.slice(1));
        current = seg.to;
        break;
      }
      case 'arc': {
        const { center, radius, startDeg, sweepDeg } = svgArcToCenter(
          current,
          seg.to,
          seg.radius,
          seg.large ?? false,
          seg.clockwise ?? true,
        );
        const pts = sampleArcAngle(center, radius, startDeg, sweepDeg, MAX_CHORD);
        points.push(...pts.slice(1));
        current = seg.to;
        break;
      }
    }
  }
  return { points, closed: shape.closed ?? false };
}

function localContour(shape: Shape): { points: Point[]; closed: boolean } {
  switch (shape.type) {
    case 'line':
      return { points: [shape.from, shape.to], closed: false };
    case 'polyline':
      return { points: [...shape.points], closed: shape.closed ?? false };
    case 'circle':
      return { points: sampleCircle(shape.center, shape.radius), closed: true };
    case 'ellipse':
      return { points: sampleEllipse(shape.center, shape.rx, shape.ry), closed: true };
    case 'rect':
      return { points: sampleRectCorners(shape.origin, shape.width, shape.height), closed: true };
    case 'roundedRect':
      return { points: sampleRectCorners(shape.origin, shape.width, shape.height), closed: true };
    case 'arc':
      return {
        points: sampleArcAngle(shape.center, shape.radius, shape.startDeg, shape.sweepDeg, MAX_CHORD),
        closed: false,
      };
    case 'curve':
      return { points: sampleCubic(shape.from, shape.control1, shape.control2, shape.to, MAX_CHORD), closed: false };
    case 'path':
      return samplePath(shape);
    case 'group':
      return { points: [], closed: false }; // unreachable: groups are recursed, not flattened directly
  }
}

function applyStack(p: Point, stack: Transform[]): Point {
  let cur = p;
  for (const t of stack) cur = transformPoint(cur, t);
  return cur;
}

/**
 * Flatten every leaf shape of an IconSpec into centerline polylines in world (icon)
 * coordinates, with group `translate`/`rotationDeg` composed exactly (innermost first).
 * Used by the join rules and exported for reuse (e.g. a future CLI preview renderer).
 */
export function flatten(spec: IconSpec): FlattenedContour[] {
  const out: FlattenedContour[] = [];
  const walk = (shapes: Shape[], prefix: string, stack: Transform[]): void => {
    shapes.forEach((shape, i) => {
      const path = `${prefix}/${i}`;
      if (shape.type === 'group') {
        const t: Transform = { translate: shape.translate ?? [0, 0], rotationDeg: shape.rotationDeg ?? 0 };
        walk(shape.children, `${path}/children`, [t, ...stack]);
        return;
      }
      const { points, closed } = localContour(shape);
      out.push({ shapeId: shape.id, path, points: points.map((p) => applyStack(p, stack)), closed });
    });
  };
  walk(spec.shapes, 'shapes', []);
  return out;
}
