import { describe, expect, it } from 'vitest';
import { OUTLINE_24_V1, type IconSpec } from '@iconforge/schema';
import { checkJoins } from './joins.ts';

const profile = OUTLINE_24_V1; // strokeWidth 1.5

function codesOf(diags: ReturnType<typeof checkJoins>): string[] {
  return diags.map((d) => d.code);
}

function spec(overrides: Partial<IconSpec>): IconSpec {
  return {
    version: 1,
    name: 'test-icon',
    profile: profile.id,
    shapes: [],
    ...overrides,
  };
}

describe('checkJoins — join.near-miss', () => {
  it('a sun ray stopping 0.4 short of the circle warns', () => {
    // circle center (12,12) r=4 -> right edge at x=16. Ray inner end at x=16.4: 0.4 short.
    const s = spec({
      shapes: [
        { id: 'sun', type: 'circle', center: [12, 12], radius: 4 },
        { id: 'ray-right', type: 'line', from: [16.4, 12], to: [20, 12] },
      ],
    });
    const diags = checkJoins(s, profile);
    expect(codesOf(diags)).toContain('join.near-miss');
    const finding = diags.find((d) => d.code === 'join.near-miss');
    expect(finding?.message).toContain('0.40');
    expect(finding?.shapeId).toBe('ray-right');
  });

  it('a ray touching the circle exactly is a clean join — no warning', () => {
    const s = spec({
      shapes: [
        { id: 'sun', type: 'circle', center: [12, 12], radius: 4 },
        { id: 'ray-right', type: 'line', from: [16, 12], to: [20, 12] },
      ],
    });
    const diags = checkJoins(s, profile);
    expect(codesOf(diags)).not.toContain('join.near-miss');
    expect(codesOf(diags)).not.toContain('join.overshoot');
  });

  it('a gap wider than strokeWidth is not reported (that is gap.too-close territory instead)', () => {
    const s = spec({
      shapes: [
        { id: 'sun', type: 'circle', center: [12, 12], radius: 4 },
        { id: 'ray-right', type: 'line', from: [18, 12], to: [20, 12] }, // 2 short, > strokeWidth 1.5
      ],
    });
    const diags = checkJoins(s, profile);
    expect(codesOf(diags)).not.toContain('join.near-miss');
  });
});

describe('checkJoins — join.overshoot', () => {
  it('a leg running 0.5 past a desktop line warns', () => {
    // desktop line horizontal at y=10, x in [4,20]. Leg crosses it at (10,10) then
    // continues to (10,10.5) — 0.5 past the crossing.
    const s = spec({
      shapes: [
        { id: 'desktop', type: 'line', from: [4, 10], to: [20, 10] },
        { id: 'leg', type: 'line', from: [10, 4], to: [10, 10.5] },
      ],
    });
    const diags = checkJoins(s, profile);
    expect(codesOf(diags)).toContain('join.overshoot');
    const finding = diags.find((d) => d.code === 'join.overshoot');
    expect(finding?.message).toContain('0.50');
    expect(finding?.shapeId).toBe('leg');
  });

  it('a leg ending exactly on the crossing is a clean join — no warning', () => {
    const s = spec({
      shapes: [
        { id: 'desktop', type: 'line', from: [4, 10], to: [20, 10] },
        { id: 'leg', type: 'line', from: [10, 4], to: [10, 10] },
      ],
    });
    const diags = checkJoins(s, profile);
    expect(codesOf(diags)).not.toContain('join.overshoot');
    expect(codesOf(diags)).not.toContain('join.near-miss');
  });
});

describe('checkJoins — same-shape pairs are never compared', () => {
  it('a path heart whose two cubics share start/end points reports nothing about itself', () => {
    const s = spec({
      shapes: [
        {
          id: 'heart',
          type: 'path',
          start: [12, 20],
          segments: [
            { kind: 'cubic', control1: [4, 14], control2: [4, 7], to: [12, 7] },
            { kind: 'cubic', control1: [20, 7], control2: [20, 14], to: [12, 20] },
          ],
          closed: true,
        },
      ],
    });
    // closed path -> not even an "open end" candidate, but this also guards the
    // same-shapeId skip for any future open self-touching path.
    expect(checkJoins(s, profile)).toEqual([]);
  });
});

describe('checkJoins — rotated group ends are checked in world coordinates', () => {
  it('a leg inside a 90deg-rotated group is measured against its transformed position', () => {
    // Local child line (0,0)->(4,0), group translate=[12,12] rotationDeg=90 (cw, y-down)
    // maps it to world (12,12)->(12,16): a "leg" pointing straight down from the center.
    const s = spec({
      shapes: [
        {
          id: 'g',
          type: 'group',
          translate: [12, 12],
          rotationDeg: 90,
          children: [{ id: 'leg', type: 'line', from: [0, 0], to: [4, 0] }],
        },
        // stops 0.4 short of the leg's transformed end at (12,16)
        { id: 'foot', type: 'line', from: [8, 16.4], to: [16, 16.4] },
      ],
    });
    const diags = checkJoins(s, profile);
    expect(codesOf(diags)).toContain('join.near-miss');
    const finding = diags.find((d) => d.code === 'join.near-miss');
    expect(finding?.shapeId).toBe('leg');
    expect(finding?.message).toContain('0.40');
  });
});
