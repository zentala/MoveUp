import type { Diagnostic, IconSpec, Point, StyleProfile } from '@iconforge/schema';
import { flatten, type FlattenedContour } from './flatten.ts';

/** Distance at or below which an end is a clean join — worth no warning. */
const CLEAN_JOIN = 0.05;

function warn(code: string, message: string, path: string, shapeId: string): Diagnostic {
  return { severity: 'warning', code, message, path, shapeId };
}

function pointToSegmentDistance(p: Point, a: Point, b: Point): number {
  const abx = b[0] - a[0];
  const aby = b[1] - a[1];
  const len2 = abx * abx + aby * aby;
  if (len2 < 1e-12) return Math.hypot(p[0] - a[0], p[1] - a[1]);
  let t = ((p[0] - a[0]) * abx + (p[1] - a[1]) * aby) / len2;
  t = Math.max(0, Math.min(1, t));
  const projX = a[0] + t * abx;
  const projY = a[1] + t * aby;
  return Math.hypot(p[0] - projX, p[1] - projY);
}

/** Minimum distance from `p` to any segment of `contour`'s polyline. */
function distanceToContour(p: Point, contour: FlattenedContour): number {
  const pts = contour.points;
  const n = pts.length;
  const segCount = contour.closed ? n : n - 1;
  let min = Infinity;
  for (let i = 0; i < segCount; i++) {
    const a = pts[i];
    const b = pts[(i + 1) % n];
    if (a === undefined || b === undefined) continue;
    const d = pointToSegmentDistance(p, a, b);
    if (d < min) min = d;
  }
  return min;
}

/** Intersection of segment a1->a2 with b1->b2, if any, with the parametric position on a. */
function segmentIntersection(
  a1: Point,
  a2: Point,
  b1: Point,
  b2: Point,
): { tOnA: number } | undefined {
  const rX = a2[0] - a1[0];
  const rY = a2[1] - a1[1];
  const sX = b2[0] - b1[0];
  const sY = b2[1] - b1[1];
  const denom = rX * sY - rY * sX;
  if (Math.abs(denom) < 1e-12) return undefined;
  const qpX = b1[0] - a1[0];
  const qpY = b1[1] - a1[1];
  const t = (qpX * sY - qpY * sX) / denom;
  const u = (qpX * rY - qpY * rX) / denom;
  if (t < 0 || t > 1 || u < 0 || u > 1) return undefined;
  return { tOnA: t };
}

/**
 * If the approach segment `a -> e` (the last segment leading into open end `e`) crosses
 * `other`'s centerline and then keeps going for a length in (0.05, strokeWidth] before
 * reaching `e`, return that overshoot distance. Picks the crossing nearest to `e` when
 * several segments of `other` are crossed.
 */
function findOvershoot(a: Point, e: Point, other: FlattenedContour, strokeWidth: number): number | undefined {
  const pts = other.points;
  const n = pts.length;
  const segCount = other.closed ? n : n - 1;
  let bestT = -1;
  for (let i = 0; i < segCount; i++) {
    const b1 = pts[i];
    const b2 = pts[(i + 1) % n];
    if (b1 === undefined || b2 === undefined) continue;
    const hit = segmentIntersection(a, e, b1, b2);
    if (hit && hit.tOnA > 1e-6 && hit.tOnA < 1 - 1e-6 && hit.tOnA > bestT) bestT = hit.tOnA;
  }
  if (bestT < 0) return undefined;
  const segLen = Math.hypot(e[0] - a[0], e[1] - a[1]);
  const d = (1 - bestT) * segLen;
  return d > CLEAN_JOIN && d <= strokeWidth ? d : undefined;
}

/**
 * `join.near-miss` / `join.overshoot`: with `stroke-linecap: round`, an open end that
 * stops short of or runs past another shape's centerline leaves a visible notch or bump
 * (half-disc of radius strokeWidth/2 at the gap). Ends within CLEAN_JOIN of another
 * centerline are clean joins. Pairs of ends belonging to the same `path`/shape are
 * never compared against each other.
 */
export function checkJoins(spec: IconSpec, profile: StyleProfile): Diagnostic[] {
  const contours = flatten(spec);
  const diagnostics: Diagnostic[] = [];
  const strokeWidth = profile.strokeWidth;

  for (const contour of contours) {
    if (contour.closed) continue;
    const pts = contour.points;
    if (pts.length < 2) continue;
    const last = pts.length - 1;
    const ends: Array<{ point: Point; approachFrom: Point }> = [
      { point: pts[0] as Point, approachFrom: pts[1] as Point },
      { point: pts[last] as Point, approachFrom: pts[last - 1] as Point },
    ];

    for (const end of ends) {
      let overshot = false;
      for (const other of contours) {
        if (other.shapeId === contour.shapeId) continue;
        const d = findOvershoot(end.approachFrom, end.point, other, strokeWidth);
        if (d !== undefined) {
          diagnostics.push(
            warn(
              'join.overshoot',
              `end of ${contour.shapeId} runs ${d.toFixed(2)} past ${other.shapeId}`,
              contour.path,
              contour.shapeId,
            ),
          );
          overshot = true;
          break;
        }
      }
      if (overshot) continue;

      let minDist = Infinity;
      let nearestId: string | undefined;
      for (const other of contours) {
        if (other.shapeId === contour.shapeId) continue;
        const d = distanceToContour(end.point, other);
        if (d < minDist) {
          minDist = d;
          nearestId = other.shapeId;
        }
      }
      if (nearestId !== undefined && minDist > CLEAN_JOIN && minDist <= strokeWidth) {
        diagnostics.push(
          warn(
            'join.near-miss',
            `end of ${contour.shapeId} stops ${minDist.toFixed(2)} short of ${nearestId} — make it meet exactly or separate by ≥ minGap`,
            contour.path,
            contour.shapeId,
          ),
        );
      }
    }
  }

  return diagnostics;
}
