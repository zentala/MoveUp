import type { IconSpec, Shape, StyleProfile } from '@iconforge/schema';
import { Resvg } from '@resvg/resvg-js';

// ---------------------------------------------------------------------------
// Number / attribute formatting — one function, fixed precision, no `-0`.
// ---------------------------------------------------------------------------

function formatNumber(value: number, precision: number): string {
  const rounded = Number(value.toFixed(precision));
  const normalized = rounded === 0 ? 0 : rounded; // strips -0
  let text = normalized.toFixed(precision);
  if (precision > 0) {
    text = text.replace(/0+$/, '').replace(/\.$/, '');
  }
  return text === '' || text === '-' ? '0' : text;
}

function escapeXml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

// ---------------------------------------------------------------------------
// Safe element builder — allowlisted tags/attrs only.
// ---------------------------------------------------------------------------

const ALLOWED_TAGS = new Set([
  'svg',
  'g',
  'line',
  'polyline',
  'polygon',
  'circle',
  'ellipse',
  'rect',
  'path',
]);

const ALLOWED_ATTRS = new Set([
  'xmlns',
  'width',
  'height',
  'viewBox',
  'fill',
  'stroke',
  'stroke-width',
  'stroke-linecap',
  'stroke-linejoin',
  'transform',
  'data-id',
  // shape-specific geometry attrs
  'x1',
  'y1',
  'x2',
  'y2',
  'points',
  'cx',
  'cy',
  'r',
  'rx',
  'ry',
  'x',
  'y',
  'd',
]);

type Attrs = Record<string, string>;

/** Renders one element with attributes in a fixed, caller-provided order. Self-closing or with children. */
function element(tag: string, attrs: Attrs, orderedKeys: readonly string[], children?: string[]): string {
  if (!ALLOWED_TAGS.has(tag)) {
    throw new Error(`iconforge/renderer: tag "${tag}" is not allowlisted`);
  }
  const parts: string[] = [];
  for (const key of orderedKeys) {
    const value = attrs[key];
    if (value === undefined) continue;
    if (!ALLOWED_ATTRS.has(key)) {
      throw new Error(`iconforge/renderer: attribute "${key}" is not allowlisted`);
    }
    parts.push(`${key}="${escapeXml(value)}"`);
  }
  const attrString = parts.length > 0 ? ` ${parts.join(' ')}` : '';
  if (children === undefined) {
    return `<${tag}${attrString}/>`;
  }
  return `<${tag}${attrString}>${children.join('')}</${tag}>`;
}

// ---------------------------------------------------------------------------
// Shape compilation
// ---------------------------------------------------------------------------

type RenderOptions = { size?: number; ids?: boolean };

function idAttr(shape: Shape, opts: RenderOptions): Attrs {
  return opts.ids === true ? { 'data-id': shape.id } : {};
}

function compilePoints(points: readonly (readonly [number, number])[], precision: number): string {
  return points.map(([x, y]) => `${formatNumber(x, precision)},${formatNumber(y, precision)}`).join(' ');
}

function compileArcPath(
  center: readonly [number, number],
  radius: number,
  startDeg: number,
  sweepDeg: number,
  precision: number,
): { isFullCircle: true } | { isFullCircle: false; d: string } {
  if (Math.abs(sweepDeg) === 360) {
    return { isFullCircle: true };
  }
  const [cx, cy] = center;
  const startRad = (startDeg * Math.PI) / 180;
  const endRad = ((startDeg + sweepDeg) * Math.PI) / 180;
  const startX = cx + radius * Math.cos(startRad);
  const startY = cy + radius * Math.sin(startRad);
  const endX = cx + radius * Math.cos(endRad);
  const endY = cy + radius * Math.sin(endRad);
  const largeArcFlag = Math.abs(sweepDeg) > 180 ? 1 : 0;
  const sweepFlag = sweepDeg > 0 ? 1 : 0;
  const fn = (n: number): string => formatNumber(n, precision);
  const d =
    `M ${fn(startX)} ${fn(startY)} ` +
    `A ${fn(radius)} ${fn(radius)} 0 ${largeArcFlag} ${sweepFlag} ${fn(endX)} ${fn(endY)}`;
  return { isFullCircle: false, d };
}

function compileShape(shape: Shape, precision: number, opts: RenderOptions): string {
  const fn = (n: number): string => formatNumber(n, precision);

  switch (shape.type) {
    case 'line': {
      const attrs: Attrs = {
        x1: fn(shape.from[0]),
        y1: fn(shape.from[1]),
        x2: fn(shape.to[0]),
        y2: fn(shape.to[1]),
        ...idAttr(shape, opts),
      };
      return element('line', attrs, ['x1', 'y1', 'x2', 'y2', 'data-id']);
    }

    case 'polyline': {
      const tag = shape.closed === true ? 'polygon' : 'polyline';
      const attrs: Attrs = {
        points: compilePoints(shape.points, precision),
        ...idAttr(shape, opts),
      };
      return element(tag, attrs, ['points', 'data-id']);
    }

    case 'circle': {
      const attrs: Attrs = {
        cx: fn(shape.center[0]),
        cy: fn(shape.center[1]),
        r: fn(shape.radius),
        ...idAttr(shape, opts),
      };
      return element('circle', attrs, ['cx', 'cy', 'r', 'data-id']);
    }

    case 'ellipse': {
      const attrs: Attrs = {
        cx: fn(shape.center[0]),
        cy: fn(shape.center[1]),
        rx: fn(shape.rx),
        ry: fn(shape.ry),
        ...idAttr(shape, opts),
      };
      return element('ellipse', attrs, ['cx', 'cy', 'rx', 'ry', 'data-id']);
    }

    case 'rect': {
      const attrs: Attrs = {
        x: fn(shape.origin[0]),
        y: fn(shape.origin[1]),
        width: fn(shape.width),
        height: fn(shape.height),
        ...idAttr(shape, opts),
      };
      return element('rect', attrs, ['x', 'y', 'width', 'height', 'data-id']);
    }

    case 'roundedRect': {
      const attrs: Attrs = {
        x: fn(shape.origin[0]),
        y: fn(shape.origin[1]),
        width: fn(shape.width),
        height: fn(shape.height),
        rx: fn(shape.radius),
        ry: fn(shape.radius),
        ...idAttr(shape, opts),
      };
      return element('rect', attrs, ['x', 'y', 'width', 'height', 'rx', 'ry', 'data-id']);
    }

    case 'arc': {
      const compiled = compileArcPath(shape.center, shape.radius, shape.startDeg, shape.sweepDeg, precision);
      if (compiled.isFullCircle) {
        const attrs: Attrs = {
          cx: fn(shape.center[0]),
          cy: fn(shape.center[1]),
          r: fn(shape.radius),
          ...idAttr(shape, opts),
        };
        return element('circle', attrs, ['cx', 'cy', 'r', 'data-id']);
      }
      const attrs: Attrs = { d: compiled.d, ...idAttr(shape, opts) };
      return element('path', attrs, ['d', 'data-id']);
    }

    case 'curve': {
      const d =
        `M ${fn(shape.from[0])} ${fn(shape.from[1])} ` +
        `C ${fn(shape.control1[0])} ${fn(shape.control1[1])} ` +
        `${fn(shape.control2[0])} ${fn(shape.control2[1])} ` +
        `${fn(shape.to[0])} ${fn(shape.to[1])}`;
      const attrs: Attrs = { d, ...idAttr(shape, opts) };
      return element('path', attrs, ['d', 'data-id']);
    }

    case 'group': {
      const children = shape.children.map((child) => compileShape(child, precision, opts));
      const transformParts: string[] = [];
      if (shape.translate !== undefined) {
        transformParts.push(`translate(${fn(shape.translate[0])} ${fn(shape.translate[1])})`);
      }
      if (shape.rotationDeg !== undefined) {
        transformParts.push(`rotate(${fn(shape.rotationDeg)})`);
      }
      const attrs: Attrs = { ...idAttr(shape, opts) };
      if (transformParts.length > 0) {
        attrs.transform = transformParts.join(' ');
      }
      return element('g', attrs, ['transform', 'data-id'], children);
    }
  }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export function renderSvg(spec: IconSpec, profile: StyleProfile, opts: RenderOptions = {}): string {
  const [vbX, vbY, vbWidth, vbHeight] = profile.viewBox;
  const width = opts.size ?? vbWidth;
  const height = opts.size ?? vbHeight;
  const precision = profile.precision;

  const body = spec.shapes.map((shape) => compileShape(shape, precision, opts)).join('');

  const svgAttrs: Attrs = {
    xmlns: 'http://www.w3.org/2000/svg',
    width: formatNumber(width, precision),
    height: formatNumber(height, precision),
    viewBox: `${formatNumber(vbX, precision)} ${formatNumber(vbY, precision)} ${formatNumber(vbWidth, precision)} ${formatNumber(vbHeight, precision)}`,
    fill: profile.fill,
    stroke: profile.stroke,
    'stroke-width': formatNumber(profile.strokeWidth, precision),
    'stroke-linecap': profile.strokeLinecap,
    'stroke-linejoin': profile.strokeLinejoin,
  };
  const svg =
    element(
      'svg',
      svgAttrs,
      ['xmlns', 'width', 'height', 'viewBox', 'fill', 'stroke', 'stroke-width', 'stroke-linecap', 'stroke-linejoin'],
      [body],
    ) + '\n';

  assertSafeSvg(svg);
  return svg;
}

const FORBIDDEN_PATTERN =
  /<script|foreignObject|<image|href|url\(|style|\son[a-z]+=/i;

/** Throws if the SVG contains anything outside the small allowlisted surface. */
export function assertSafeSvg(svg: string): void {
  if (FORBIDDEN_PATTERN.test(svg)) {
    throw new Error('iconforge/renderer: unsafe content detected in generated SVG');
  }
  const tagPattern = /<\/?([a-zA-Z][a-zA-Z0-9]*)/g;
  let match: RegExpExecArray | null;
  while ((match = tagPattern.exec(svg)) !== null) {
    const tag = match[1] as string;
    if (!ALLOWED_TAGS.has(tag)) {
      throw new Error(`iconforge/renderer: unsafe tag "<${tag}>" in generated SVG`);
    }
  }
}

export type Background = 'light' | 'dark' | 'transparent';

const BACKGROUND_COLOR: Record<Exclude<Background, 'transparent'>, string> = {
  light: '#ffffff',
  dark: '#111111',
};

const STROKE_COLOR_FOR_BACKGROUND: Record<Exclude<Background, 'transparent'>, string> = {
  light: '#111111',
  dark: '#eeeeee',
};

export function renderPng(
  svg: string,
  opts: { size: number; background: Background; color?: string },
): Buffer {
  const strokeColor =
    opts.color ?? (opts.background === 'transparent' ? '#111111' : STROKE_COLOR_FOR_BACKGROUND[opts.background]);
  const rasterizable = svg.replace(/currentColor/g, strokeColor);

  const resvg = new Resvg(rasterizable, {
    fitTo: { mode: 'width', value: opts.size },
    background: opts.background === 'transparent' ? undefined : BACKGROUND_COLOR[opts.background],
  });
  return resvg.render().asPng();
}
