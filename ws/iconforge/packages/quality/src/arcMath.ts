import type { Point } from '@iconforge/schema';
import type { Bounds } from './bounds.ts';

const EPS = 1e-9;

function boundsOf(points: readonly Point[]): Bounds {
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

function dist(a: Point, b: Point): number {
  return Math.hypot(a[0] - b[0], a[1] - b[1]);
}

/**
 * Exact bbox of a circular arc's centerline in center-form: endpoints plus any cardinal
 * angle crossed by the sweep. Angles use plain atan2/cos/sin in this y-down coordinate
 * space, which is exactly the SVG convention: increasing angle reads as clockwise on
 * screen, matching `sweepDeg`'s "positive = clockwise" contract.
 */
export function arcAngleBounds(center: Point, radius: number, startDeg: number, sweepDeg: number): Bounds {
  const start = startDeg;
  const end = startDeg + sweepDeg;
  const lo = Math.min(start, end);
  const hi = Math.max(start, end);
  const angles = [start, end];
  for (const base of [0, 90, 180, 270]) {
    for (const k of [-360, 0, 360]) {
      const a = base + k;
      if (a >= lo - EPS && a <= hi + EPS) angles.push(a);
    }
  }
  const points = angles.map((deg): Point => {
    const rad = (deg * Math.PI) / 180;
    return [center[0] + radius * Math.cos(rad), center[1] + radius * Math.sin(rad)];
  });
  return boundsOf(points);
}

/** Sample a circular arc (center-form) into a polyline with chords no longer than `maxChord`. */
export function sampleArcAngle(
  center: Point,
  radius: number,
  startDeg: number,
  sweepDeg: number,
  maxChord: number,
): Point[] {
  const arcLen = (Math.abs(sweepDeg) * Math.PI * radius) / 180;
  const steps = Math.max(1, Math.ceil(arcLen / maxChord));
  const pts: Point[] = [];
  for (let i = 0; i <= steps; i++) {
    const deg = startDeg + (sweepDeg * i) / steps;
    const rad = (deg * Math.PI) / 180;
    pts.push([center[0] + radius * Math.cos(rad), center[1] + radius * Math.sin(rad)]);
  }
  return pts;
}

export interface CenterArc {
  center: Point;
  radius: number;
  startDeg: number;
  sweepDeg: number;
}

/**
 * SVG endpoint arc parameterization → center form (SVG 1.1 spec, appendix F.6.5),
 * specialized to a circular arc (rx = ry = radius, x-axis-rotation = 0).
 */
export function svgArcToCenter(p0: Point, p1: Point, radiusIn: number, large: boolean, clockwise: boolean): CenterArc {
  const [x1, y1] = p0;
  const [x2, y2] = p1;
  let r = radiusIn;
  const x1p = (x1 - x2) / 2;
  const y1p = (y1 - y2) / 2;
  const sumSq = x1p * x1p + y1p * y1p;
  if (sumSq < EPS) return { center: p0, radius: r, startDeg: 0, sweepDeg: clockwise ? 360 : -360 };
  const lambda = sumSq / (r * r);
  if (lambda > 1) r *= Math.sqrt(lambda);
  const ratio = Math.max(0, (r * r - sumSq) / sumSq);
  const sign = large !== clockwise ? 1 : -1;
  const co = sign * Math.sqrt(ratio);
  const cxp = co * y1p;
  const cyp = -co * x1p;
  const cx = cxp + (x1 + x2) / 2;
  const cy = cyp + (y1 + y2) / 2;
  const startDeg = (Math.atan2(y1 - cy, x1 - cx) * 180) / Math.PI;
  const endDeg = (Math.atan2(y2 - cy, x2 - cx) * 180) / Math.PI;
  let sweepDeg = endDeg - startDeg;
  if (clockwise && sweepDeg < 0) sweepDeg += 360;
  if (!clockwise && sweepDeg > 0) sweepDeg -= 360;
  return { center: [cx, cy], radius: r, startDeg, sweepDeg };
}

function cubicPoint(p0: Point, p1: Point, p2: Point, p3: Point, t: number): Point {
  const mt = 1 - t;
  const a = mt * mt * mt;
  const b = 3 * mt * mt * t;
  const c = 3 * mt * t * t;
  const d = t * t * t;
  return [a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0], a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1]];
}

/** Exact bbox of a cubic bezier via derivative roots (per-axis quadratic). */
export function cubicBounds(p0: Point, p1: Point, p2: Point, p3: Point): Bounds {
  const ts = [0, 1];
  for (const axis of [0, 1] as const) {
    const a0 = p0[axis];
    const a1 = p1[axis];
    const a2 = p2[axis];
    const a3 = p3[axis];
    const d0 = a1 - a0;
    const d1 = a2 - a1;
    const d2 = a3 - a2;
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
  return boundsOf(ts.map((t) => cubicPoint(p0, p1, p2, p3, t)));
}

/** Sample a cubic bezier into a polyline with chords approximately no longer than `maxChord`. */
export function sampleCubic(p0: Point, p1: Point, p2: Point, p3: Point, maxChord: number): Point[] {
  const polyLen = dist(p0, p1) + dist(p1, p2) + dist(p2, p3);
  const steps = Math.max(1, Math.ceil(polyLen / maxChord));
  const pts: Point[] = [];
  for (let i = 0; i <= steps; i++) pts.push(cubicPoint(p0, p1, p2, p3, i / steps));
  return pts;
}

function quadPoint(p0: Point, p1: Point, p2: Point, t: number): Point {
  const mt = 1 - t;
  const a = mt * mt;
  const b = 2 * mt * t;
  const c = t * t;
  return [a * p0[0] + b * p1[0] + c * p2[0], a * p0[1] + b * p1[1] + c * p2[1]];
}

/** Exact bbox of a quadratic bezier via its single derivative root per axis. */
export function quadBounds(p0: Point, p1: Point, p2: Point): Bounds {
  const ts = [0, 1];
  for (const axis of [0, 1] as const) {
    const a0 = p0[axis];
    const a1 = p1[axis];
    const a2 = p2[axis];
    const denom = a0 - 2 * a1 + a2;
    if (Math.abs(denom) > 1e-12) {
      const t = (a0 - a1) / denom;
      if (t > 0 && t < 1) ts.push(t);
    }
  }
  return boundsOf(ts.map((t) => quadPoint(p0, p1, p2, t)));
}

/** Sample a quadratic bezier into a polyline with chords approximately no longer than `maxChord`. */
export function sampleQuad(p0: Point, p1: Point, p2: Point, maxChord: number): Point[] {
  const polyLen = dist(p0, p1) + dist(p1, p2);
  const steps = Math.max(1, Math.ceil(polyLen / maxChord));
  const pts: Point[] = [];
  for (let i = 0; i <= steps; i++) pts.push(quadPoint(p0, p1, p2, i / steps));
  return pts;
}
