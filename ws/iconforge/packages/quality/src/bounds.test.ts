import { describe, expect, it } from 'vitest';
import type { ArcShape, CurveShape, GroupShape } from '@iconforge/schema';
import { shapeBounds } from './bounds.ts';

describe('shapeBounds', () => {
  it('line', () => {
    expect(shapeBounds({ id: 'a', type: 'line', from: [1, 2], to: [5, 6] })).toEqual({
      minX: 1,
      minY: 2,
      maxX: 5,
      maxY: 6,
    });
  });

  it('circle', () => {
    expect(shapeBounds({ id: 'a', type: 'circle', center: [10, 10], radius: 3 })).toEqual({
      minX: 7,
      minY: 7,
      maxX: 13,
      maxY: 13,
    });
  });

  it('rect and roundedRect share the same box', () => {
    const rect = shapeBounds({ id: 'a', type: 'rect', origin: [2, 3], width: 4, height: 5 });
    const rounded = shapeBounds({ id: 'b', type: 'roundedRect', origin: [2, 3], width: 4, height: 5, radius: 1 });
    expect(rect).toEqual({ minX: 2, minY: 3, maxX: 6, maxY: 8 });
    expect(rounded).toEqual(rect);
  });

  describe('arc', () => {
    it('0 to 90 sweep includes the 90deg extreme', () => {
      const arc: ArcShape = { id: 'a', type: 'arc', center: [0, 0], radius: 10, startDeg: 0, sweepDeg: 90 };
      const b = shapeBounds(arc);
      expect(b.minX).toBeCloseTo(0);
      expect(b.maxX).toBeCloseTo(10);
      expect(b.minY).toBeCloseTo(0);
      expect(b.maxY).toBeCloseTo(10);
    });

    it('45 to 315 sweep (270deg) crosses 90, 180, 270 but not 0', () => {
      const arc: ArcShape = { id: 'a', type: 'arc', center: [0, 0], radius: 10, startDeg: 45, sweepDeg: 270 };
      const b = shapeBounds(arc);
      expect(b.minX).toBeCloseTo(-10); // 180deg
      expect(b.maxX).toBeCloseTo(10 * Math.cos((45 * Math.PI) / 180), 5); // start point, 0deg not in sweep
      expect(b.minY).toBeCloseTo(-10); // 270deg
      expect(b.maxY).toBeCloseTo(10); // 90deg
    });

    it('negative sweep goes the other direction', () => {
      const arc: ArcShape = { id: 'a', type: 'arc', center: [0, 0], radius: 10, startDeg: 90, sweepDeg: -90 };
      const b = shapeBounds(arc);
      // sweeps from 90 back to 0: covers [0,90], includes both endpoints, no other cardinal crossed
      expect(b.minX).toBeCloseTo(0);
      expect(b.maxX).toBeCloseTo(10);
      expect(b.minY).toBeCloseTo(0);
      expect(b.maxY).toBeCloseTo(10);
    });
  });

  it('curve: exact cubic bezier extrema', () => {
    // A symmetric S-curve whose control points overshoot the endpoints' y-range.
    const curve: CurveShape = {
      id: 'a',
      type: 'curve',
      from: [0, 0],
      control1: [0, 10],
      control2: [10, -10],
      to: [10, 0],
    };
    const b = shapeBounds(curve);
    // Endpoints alone would give y in [0,0]; true extrema must exceed that range.
    expect(b.minY).toBeLessThan(0);
    expect(b.maxY).toBeGreaterThan(0);
    expect(b.minX).toBeCloseTo(0, 1);
    expect(b.maxX).toBeCloseTo(10, 1);
  });

  it('group: translate then rotate around the group origin, conservative on rotation', () => {
    const group: GroupShape = {
      id: 'g',
      type: 'group',
      translate: [10, 10],
      rotationDeg: 90,
      children: [{ id: 'c', type: 'rect', origin: [0, 0], width: 4, height: 2 }],
    };
    const b = shapeBounds(group);
    // Rotating the axis-aligned rect's corners by 90deg around its own translated
    // origin swaps width/height in the resulting AABB.
    expect(b.maxX - b.minX).toBeCloseTo(2, 5);
    expect(b.maxY - b.minY).toBeCloseTo(4, 5);
  });
});

describe('shapeBounds — path', () => {
  it('path heart: two cubics from a shared apex back to a shared bottom point', () => {
    // Left lobe: apex(12,7) -> bulge up-left -> bottom(12,20). Right lobe mirrors it.
    const heart: import('@iconforge/schema').PathShape = {
      id: 'heart',
      type: 'path',
      start: [12, 20],
      segments: [
        { kind: 'cubic', control1: [4, 14], control2: [4, 7], to: [12, 7] },
        { kind: 'cubic', control1: [20, 7], control2: [20, 14], to: [12, 20] },
      ],
      closed: true,
    };
    const b = shapeBounds(heart);
    // Both cubics bow outward past the endpoints' own x=12, toward their control points.
    expect(b.minX).toBeLessThan(12);
    expect(b.maxX).toBeGreaterThan(12);
    expect(b.minY).toBeCloseTo(7, 1);
    expect(b.maxY).toBeCloseTo(20, 1);
  });

  describe('path arc segment — both flag combinations', () => {
    // Chord (0,0)->(10,0), radius 10: a 60deg "small" arc bulging one way, and its
    // complementary 300deg "large" arc bulging the other way, for each sweep direction.
    const start: [number, number] = [0, 0];
    const to: [number, number] = [10, 0];
    const makePath = (large: boolean, clockwise: boolean): import('@iconforge/schema').PathShape => ({
      id: 'a',
      type: 'path',
      start,
      segments: [{ kind: 'arc', to, radius: 10, large, clockwise }],
    });

    it('both endpoints stay exactly on every combination\'s bbox chord', () => {
      for (const large of [false, true]) {
        for (const clockwise of [false, true]) {
          const b = shapeBounds(makePath(large, clockwise));
          expect(b.minX).toBeLessThanOrEqual(0 + 1e-6);
          expect(b.maxX).toBeGreaterThanOrEqual(10 - 1e-6);
        }
      }
    });

    it('small arc (large=false) bulges less than its complementary large arc', () => {
      const small = shapeBounds(makePath(false, true));
      const large = shapeBounds(makePath(true, true));
      const smallBulge = Math.max(Math.abs(small.minY), Math.abs(small.maxY));
      const largeBulge = Math.max(Math.abs(large.minY), Math.abs(large.maxY));
      expect(smallBulge).toBeLessThan(largeBulge);
    });

    it('flipping clockwise (large fixed) bulges to the opposite side of the chord', () => {
      const cw = shapeBounds(makePath(false, true));
      const ccw = shapeBounds(makePath(false, false));
      // one bulges above the chord (y<0), the other below (y>0); never both on the same side
      expect(cw.minY < 0 && cw.maxY <= 1e-9).toBe(true);
      expect(ccw.maxY > 0 && ccw.minY >= -1e-9).toBe(true);
    });
  });
});
