import { describe, expect, it } from 'vitest';
import { OUTLINE_24_V1 } from '@iconforge/schema';
import type { IconSpec } from '@iconforge/schema';
import { assertSafeSvg, renderPng, renderSvg } from './index.ts';

function spec(shapes: IconSpec['shapes']): IconSpec {
  return { version: 1, name: 'test-icon', profile: OUTLINE_24_V1.id, shapes };
}

describe('renderSvg — one primitive each', () => {
  it('line', () => {
    const svg = renderSvg(spec([{ id: 'a', type: 'line', from: [1, 2], to: [3, 4] }]), OUTLINE_24_V1);
    expect(svg).toContain('<line x1="1" y1="2" x2="3" y2="4"/>');
    assertSafeSvg(svg);
  });

  it('polyline (open)', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'polyline', points: [[1, 1], [2, 2], [3, 1]] }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<polyline points="1,1 2,2 3,1"/>');
    assertSafeSvg(svg);
  });

  it('polyline closed becomes polygon, never filled', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'polyline', points: [[1, 1], [2, 2], [3, 1]], closed: true }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<polygon points="1,1 2,2 3,1"/>');
    expect(svg).toContain('fill="none"');
    assertSafeSvg(svg);
  });

  it('circle', () => {
    const svg = renderSvg(spec([{ id: 'a', type: 'circle', center: [5, 5], radius: 2 }]), OUTLINE_24_V1);
    expect(svg).toContain('<circle cx="5" cy="5" r="2"/>');
    assertSafeSvg(svg);
  });

  it('ellipse', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'ellipse', center: [5, 5], rx: 3, ry: 2 }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<ellipse cx="5" cy="5" rx="3" ry="2"/>');
    assertSafeSvg(svg);
  });

  it('rect', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'rect', origin: [1, 1], width: 4, height: 3 }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<rect x="1" y="1" width="4" height="3"/>');
    assertSafeSvg(svg);
  });

  it('roundedRect emits rx/ry without clamping', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'roundedRect', origin: [1, 1], width: 4, height: 3, radius: 5 }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<rect x="1" y="1" width="4" height="3" rx="5" ry="5"/>');
    assertSafeSvg(svg);
  });

  it('curve compiles to a cubic bezier path', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'curve', from: [0, 0], control1: [1, 0], control2: [1, 1], to: [2, 1] }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<path d="M 0 0 C 1 0 1 1 2 1"/>');
    assertSafeSvg(svg);
  });

  it('group without translate/rotation omits transform', () => {
    const svg = renderSvg(
      spec([{ id: 'g1', type: 'group', children: [{ id: 'a', type: 'line', from: [0, 0], to: [1, 1] }] }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<g><line x1="0" y1="0" x2="1" y2="1"/></g>');
    assertSafeSvg(svg);
  });

  it('group with translate and rotation', () => {
    const svg = renderSvg(
      spec([
        {
          id: 'g1',
          type: 'group',
          translate: [2, 3],
          rotationDeg: 45,
          children: [{ id: 'a', type: 'line', from: [0, 0], to: [1, 1] }],
        },
      ]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<g transform="translate(2 3) rotate(45)">');
    assertSafeSvg(svg);
  });
});

describe('renderSvg — arc contract', () => {
  function arcPath(sweepDeg: number): string {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'arc', center: [0, 0], radius: 10, startDeg: 0, sweepDeg }]),
      OUTLINE_24_V1,
    );
    const match = svg.match(/<(path|circle)[^/]*\/>/);
    return match ? match[0] : '';
  }

  it('90deg: small arc, sweep flag 1 (clockwise)', () => {
    const el = arcPath(90);
    expect(el).toMatch(/^<path d="M 10 0 A 10 10 0 0 1 [\d.-]+ [\d.-]+"\/>$/);
  });

  it('-90deg: small arc, sweep flag 0 (counter-clockwise)', () => {
    const el = arcPath(-90);
    expect(el).toMatch(/^<path d="M 10 0 A 10 10 0 0 0 [\d.-]+ [\d.-]+"\/>$/);
  });

  it('270deg: large-arc flag set', () => {
    const el = arcPath(270);
    expect(el).toMatch(/^<path d="M 10 0 A 10 10 0 1 1 [\d.-]+ [\d.-]+"\/>$/);
  });

  it('360deg: emitted as a circle', () => {
    const el = arcPath(360);
    expect(el).toBe('<circle cx="0" cy="0" r="10"/>');
  });

  it('-360deg: also a circle', () => {
    const el = arcPath(-360);
    expect(el).toBe('<circle cx="0" cy="0" r="10"/>');
  });
});

describe('determinism', () => {
  it('renders byte-identical output for the same input', () => {
    const s = spec([
      { id: 'a', type: 'line', from: [1, 2], to: [3, 4] },
      { id: 'b', type: 'circle', center: [5, 5], radius: 2 },
    ]);
    const first = renderSvg(s, OUTLINE_24_V1);
    const second = renderSvg(s, OUTLINE_24_V1);
    expect(first).toBe(second);
  });

  it('is not affected by object key order in the input', () => {
    const shapeA = { type: 'line', to: [3, 4], id: 'a', from: [1, 2] } as IconSpec['shapes'][number];
    const shapeB = { id: 'a', from: [1, 2], type: 'line', to: [3, 4] } as IconSpec['shapes'][number];
    const first = renderSvg(spec([shapeA]), OUTLINE_24_V1);
    const second = renderSvg(spec([shapeB]), OUTLINE_24_V1);
    expect(first).toBe(second);
  });
});

describe('number formatting', () => {
  it('strips trailing zeros and avoids -0', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'line', from: [0, -0], to: [1.5, 2.0] }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('x1="0" y1="0" x2="1.5" y2="2"');
    expect(svg).not.toContain('-0');
  });

  it('respects profile precision', () => {
    const profile = { ...OUTLINE_24_V1, precision: 3 };
    const svg = renderSvg(spec([{ id: 'a', type: 'circle', center: [1, 1], radius: 1.23456 }]), profile);
    expect(svg).toContain('r="1.235"');
  });
});

describe('assertSafeSvg', () => {
  it('rejects a script tag', () => {
    expect(() => assertSafeSvg('<svg><script>alert(1)</script></svg>')).toThrow();
  });

  it('rejects an unknown tag', () => {
    expect(() => assertSafeSvg('<svg><foreignObject></foreignObject></svg>')).toThrow();
  });

  it('accepts a well-formed render', () => {
    const svg = renderSvg(spec([{ id: 'a', type: 'circle', center: [1, 1], radius: 1 }]), OUTLINE_24_V1);
    expect(() => assertSafeSvg(svg)).not.toThrow();
  });
});

describe('renderSvg — path', () => {
  it('line segment', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'path', start: [1, 1], segments: [{ kind: 'line', to: [3, 4] }] }]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<path d="M 1 1 L 3 4"/>');
    assertSafeSvg(svg);
  });

  it('quad segment', () => {
    const svg = renderSvg(
      spec([
        {
          id: 'a',
          type: 'path',
          start: [0, 0],
          segments: [{ kind: 'quad', control: [1, 2], to: [3, 4] }],
        },
      ]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<path d="M 0 0 Q 1 2 3 4"/>');
    assertSafeSvg(svg);
  });

  it('cubic segment', () => {
    const svg = renderSvg(
      spec([
        {
          id: 'a',
          type: 'path',
          start: [0, 0],
          segments: [{ kind: 'cubic', control1: [1, 1], control2: [2, 2], to: [3, 3] }],
        },
      ]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<path d="M 0 0 C 1 1 2 2 3 3"/>');
    assertSafeSvg(svg);
  });

  it('arc segment defaults: large=0, clockwise sweep=1', () => {
    const svg = renderSvg(
      spec([
        {
          id: 'a',
          type: 'path',
          start: [0, 0],
          segments: [{ kind: 'arc', to: [5, 5], radius: 3 }],
        },
      ]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<path d="M 0 0 A 3 3 0 0 1 5 5"/>');
    assertSafeSvg(svg);
  });

  it('arc segment: large=true, clockwise=false', () => {
    const svg = renderSvg(
      spec([
        {
          id: 'a',
          type: 'path',
          start: [0, 0],
          segments: [{ kind: 'arc', to: [5, 5], radius: 3, large: true, clockwise: false }],
        },
      ]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<path d="M 0 0 A 3 3 0 1 0 5 5"/>');
    assertSafeSvg(svg);
  });

  it('open path has no trailing Z', () => {
    const svg = renderSvg(
      spec([{ id: 'a', type: 'path', start: [0, 0], segments: [{ kind: 'line', to: [1, 1] }] }]),
      OUTLINE_24_V1,
    );
    expect(svg).not.toContain('Z');
  });

  it('closed path adds trailing Z', () => {
    const svg = renderSvg(
      spec([
        {
          id: 'a',
          type: 'path',
          start: [0, 0],
          segments: [{ kind: 'line', to: [1, 1] }],
          closed: true,
        },
      ]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('<path d="M 0 0 L 1 1 Z"/>');
    assertSafeSvg(svg);
  });

  it('never fills — path relies on profile fill="none"', () => {
    const svg = renderSvg(
      spec([
        {
          id: 'a',
          type: 'path',
          start: [0, 0],
          segments: [{ kind: 'line', to: [1, 1] }],
          closed: true,
        },
      ]),
      OUTLINE_24_V1,
    );
    expect(svg).toContain('fill="none"');
  });

  it('is deterministic across repeated calls', () => {
    const s = spec([
      {
        id: 'a',
        type: 'path',
        start: [0, 0],
        segments: [
          { kind: 'line', to: [1, 1] },
          { kind: 'quad', control: [2, 0], to: [3, 1] },
          { kind: 'cubic', control1: [4, 0], control2: [5, 2], to: [6, 1] },
          { kind: 'arc', to: [7, 1], radius: 2 },
        ],
      },
    ]);
    const first = renderSvg(s, OUTLINE_24_V1);
    const second = renderSvg(s, OUTLINE_24_V1);
    expect(first).toBe(second);
  });

  it('heart shape made of two cubics: closed, safe, and rasterizable', () => {
    const heart = spec([
      {
        id: 'heart',
        type: 'path',
        start: [12, 20],
        segments: [
          { kind: 'cubic', control1: [2, 10], control2: [2, 2], to: [12, 8] },
          { kind: 'cubic', control1: [22, 2], control2: [22, 10], to: [12, 20] },
        ],
        closed: true,
      },
    ]);
    const svg = renderSvg(heart, OUTLINE_24_V1);
    expect(svg).toContain('<path d="M 12 20 C 2 10 2 2 12 8 C 22 2 22 10 12 20 Z"/>');
    assertSafeSvg(svg);
    const png = renderPng(svg, { size: 24, background: 'light' });
    expect(png.subarray(0, 8)).toEqual(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]));
  });
});

describe('renderPng', () => {
  const svg = renderSvg(spec([{ id: 'a', type: 'circle', center: [12, 12], radius: 8 }]), OUTLINE_24_V1);
  const PNG_MAGIC = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

  it('renders a PNG at size 24', () => {
    const png = renderPng(svg, { size: 24, background: 'light' });
    expect(png.subarray(0, 8)).toEqual(PNG_MAGIC);
  });

  it('renders a PNG at size 512', () => {
    const png = renderPng(svg, { size: 512, background: 'dark' });
    expect(png.subarray(0, 8)).toEqual(PNG_MAGIC);
  });

  it('supports a transparent background', () => {
    const png = renderPng(svg, { size: 64, background: 'transparent' });
    expect(png.subarray(0, 8)).toEqual(PNG_MAGIC);
  });
});
