import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { OUTLINE_24_V1, parseIconSpec, type IconSpec, type StyleProfile } from '@iconforge/schema';
import { checkGeometry } from './rules.ts';
import { summarize, validateIcon } from './index.ts';

const profile = OUTLINE_24_V1;

function codesOf(diags: ReturnType<typeof checkGeometry>): string[] {
  return diags.map((d) => d.code);
}

function spec(overrides: Partial<IconSpec>): IconSpec {
  return {
    version: 1,
    name: 'test-icon',
    profile: profile.id,
    shapes: [{ id: 'a', type: 'line', from: [4, 4], to: [20, 20] }],
    ...overrides,
  };
}

describe('checkGeometry — desk example', () => {
  it('the shipped desk.json passes with zero errors', () => {
    const deskPath = fileURLToPath(
      new URL('../../../examples/icons/moveup/desk.json', import.meta.url),
    );
    const raw = JSON.parse(readFileSync(deskPath, 'utf8'));
    const parsed = parseIconSpec(raw);
    expect(parsed.ok).toBe(true);
    if (!parsed.ok) return;
    const diags = checkGeometry(parsed.value, profile);
    const summary = summarize(diags);
    expect(summary.errors).toBe(0);
  });
});

describe('checkGeometry — error rules', () => {
  it('limits.max-shapes', () => {
    const tiny: StyleProfile = { ...profile, limits: { ...profile.limits, maxShapes: 1 } };
    const s = spec({ shapes: [...spec({}).shapes, { id: 'b', type: 'line', from: [1, 1], to: [2, 2] }] });
    expect(codesOf(checkGeometry(s, tiny))).toContain('limits.max-shapes');
  });

  it('limits.group-depth', () => {
    const shallow: StyleProfile = { ...profile, limits: { ...profile.limits, maxGroupDepth: 1 } };
    const s = spec({
      shapes: [
        {
          id: 'g1',
          type: 'group',
          children: [
            {
              id: 'g2',
              type: 'group',
              children: [{ id: 'c', type: 'line', from: [1, 1], to: [2, 2] }],
            },
          ],
        },
      ],
    });
    expect(codesOf(checkGeometry(s, shallow))).toContain('limits.group-depth');
  });

  it('limits.coord-range', () => {
    const s = spec({ shapes: [{ id: 'a', type: 'line', from: [4, 4], to: [1000, 4] }] });
    expect(codesOf(checkGeometry(s, profile))).toContain('limits.coord-range');
  });

  it('profile.shape-not-allowed', () => {
    const noArcs: StyleProfile = {
      ...profile,
      allowedShapes: profile.allowedShapes.filter((t) => t !== 'arc'),
    };
    const s = spec({ shapes: [{ id: 'a', type: 'arc', center: [12, 12], radius: 5, startDeg: 0, sweepDeg: 90 }] });
    expect(codesOf(checkGeometry(s, noArcs))).toContain('profile.shape-not-allowed');
  });

  it('profile.mismatch', () => {
    const s = spec({ profile: 'other-profile' });
    expect(codesOf(checkGeometry(s, profile))).toContain('profile.mismatch');
  });

  describe('bounds.outside-viewbox — stroke edge case', () => {
    // strokeWidth 1.5 -> half stroke 0.75; viewBox starts at x=0.
    it('a vertical line at x=0.5 is outside (half-stroke crosses 0)', () => {
      const s = spec({ shapes: [{ id: 'a', type: 'line', from: [0.5, 4], to: [0.5, 20] }] });
      expect(codesOf(checkGeometry(s, profile))).toContain('bounds.outside-viewbox');
    });

    it('a vertical line at x=0.75 sits exactly on the edge and is allowed', () => {
      const s = spec({ shapes: [{ id: 'a', type: 'line', from: [0.75, 4], to: [0.75, 20] }] });
      expect(codesOf(checkGeometry(s, profile))).not.toContain('bounds.outside-viewbox');
    });
  });
});

describe('checkGeometry — warning rules', () => {
  it('bounds.margin', () => {
    // margin=2; a line 1 unit from the edge (plus half-stroke 0.75) enters the margin
    // but stays inside the viewBox.
    const s = spec({ shapes: [{ id: 'a', type: 'line', from: [1, 4], to: [1, 20] }] });
    expect(codesOf(checkGeometry(s, profile))).toContain('bounds.margin');
  });

  it('grid.off-grid on endpoints, but never on curve control points', () => {
    const offGrid = spec({ shapes: [{ id: 'a', type: 'line', from: [4.13, 4], to: [20, 20] }] });
    expect(codesOf(checkGeometry(offGrid, profile))).toContain('grid.off-grid');

    const curveOffControl = spec({
      shapes: [
        {
          id: 'a',
          type: 'curve',
          from: [4, 4],
          control1: [4.123, 8.456], // off-grid control point: must never warn
          control2: [16.789, 4.321],
          to: [20, 20],
        },
      ],
    });
    expect(codesOf(checkGeometry(curveOffControl, profile))).not.toContain('grid.off-grid');
  });

  it('rounded.radius-too-large', () => {
    const s = spec({ shapes: [{ id: 'a', type: 'roundedRect', origin: [4, 4], width: 4, height: 4, radius: 3 }] });
    expect(codesOf(checkGeometry(s, profile))).toContain('rounded.radius-too-large');
  });

  it('gap.too-close between two nearby but non-overlapping top-level shapes', () => {
    // minGap = 1; strokeWidth 1.5 -> half stroke 0.75 each side.
    const s = spec({
      shapes: [
        { id: 'a', type: 'rect', origin: [4, 4], width: 2, height: 2 },
        { id: 'b', type: 'rect', origin: [8, 4], width: 2, height: 2 },
      ],
    });
    expect(codesOf(checkGeometry(s, profile))).toContain('gap.too-close');
  });

  it('overlapping shapes never produce gap.too-close', () => {
    const s = spec({
      shapes: [
        { id: 'a', type: 'rect', origin: [4, 4], width: 4, height: 4 },
        { id: 'b', type: 'rect', origin: [6, 6], width: 4, height: 4 },
      ],
    });
    expect(codesOf(checkGeometry(s, profile))).not.toContain('gap.too-close');
  });

  it('detail.tiny-shape', () => {
    const s = spec({ shapes: [{ id: 'a', type: 'circle', center: [12, 12], radius: 0.3 }] });
    expect(codesOf(checkGeometry(s, profile))).toContain('detail.tiny-shape');
  });
});

describe('checkGeometry — grouped rotated shape', () => {
  it('a rotated group is checked against the viewBox using its conservative bbox', () => {
    const s = spec({
      shapes: [
        {
          id: 'g',
          type: 'group',
          translate: [12, 12],
          rotationDeg: 45,
          children: [{ id: 'c', type: 'rect', origin: [-2, -2], width: 4, height: 4 }],
        },
      ],
    });
    const diags = checkGeometry(s, profile);
    // centered, small, rotated square stays well inside the viewBox
    expect(codesOf(diags)).not.toContain('bounds.outside-viewbox');
  });
});

describe('validateIcon', () => {
  it('combines schema parsing with geometry checks', () => {
    const result = validateIcon(spec({}), profile);
    expect(result.ok).toBe(true);
  });

  it('reports ok=false when the input fails schema parsing', () => {
    const result = validateIcon({ not: 'an icon' }, profile);
    expect(result.ok).toBe(false);
  });

  it('reports ok=false when geometry has errors', () => {
    const result = validateIcon(spec({ profile: 'wrong' }), profile);
    expect(result.ok).toBe(false);
  });
});
