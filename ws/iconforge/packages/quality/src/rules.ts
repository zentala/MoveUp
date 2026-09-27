import type { Diagnostic, IconSpec, Point, Shape, StyleProfile } from '@iconforge/schema';
import { shapeBounds, type Bounds } from './bounds.ts';

interface Node {
  shape: Shape;
  path: string;
  /** Number of ancestor group shapes (0 for top-level entries in spec.shapes). */
  ancestorGroupCount: number;
}

function collectAll(shapes: Shape[]): Node[] {
  const out: Node[] = [];
  const walk = (list: Shape[], prefix: string, ancestorGroupCount: number): void => {
    list.forEach((shape, i) => {
      const path = `${prefix}/${i}`;
      out.push({ shape, path, ancestorGroupCount });
      if (shape.type === 'group') walk(shape.children, `${path}/children`, ancestorGroupCount + 1);
    });
  };
  walk(shapes, 'shapes', 0);
  return out;
}

function err(code: string, message: string, path: string, shapeId?: string): Diagnostic {
  const d: Diagnostic = { severity: 'error', code, message, path };
  if (shapeId !== undefined) d.shapeId = shapeId;
  return d;
}

function warn(code: string, message: string, path: string, shapeId?: string): Diagnostic {
  const d: Diagnostic = { severity: 'warning', code, message, path };
  if (shapeId !== undefined) d.shapeId = shapeId;
  return d;
}

/** Raw coordinate points (as written), used for the coordMin/coordMax range check. */
function rawPoints(shape: Shape): Point[] {
  switch (shape.type) {
    case 'line':
      return [shape.from, shape.to];
    case 'polyline':
      return [...shape.points];
    case 'circle':
    case 'ellipse':
      return [shape.center];
    case 'rect':
    case 'roundedRect':
      return [shape.origin];
    case 'arc':
      return [shape.center];
    case 'curve':
      return [shape.from, shape.control1, shape.control2, shape.to];
    case 'group':
      return shape.translate ? [shape.translate] : [];
  }
}

/** Numeric values checked against the grid (endpoints/centers/origins/sizes). Curve control points are never included. */
function gridCheckValues(shape: Shape): number[] {
  switch (shape.type) {
    case 'line':
      return [...shape.from, ...shape.to];
    case 'polyline':
      return shape.points.flatMap((p) => [...p]);
    case 'circle':
      return [...shape.center, shape.radius];
    case 'ellipse':
      return [...shape.center, shape.rx, shape.ry];
    case 'rect':
      return [...shape.origin, shape.width, shape.height];
    case 'roundedRect':
      return [...shape.origin, shape.width, shape.height, shape.radius];
    case 'arc':
      return [...shape.center, shape.radius];
    case 'curve':
      return [...shape.from, ...shape.to];
    case 'group':
      return shape.translate ? [...shape.translate] : [];
  }
}

function isOnGrid(v: number, grid: number, tolerance: number): boolean {
  const nearest = Math.round(v / grid) * grid;
  return Math.abs(nearest - v) <= tolerance;
}

/** Distance between two axis-aligned rects; 0 when overlapping or touching. */
function rectGap(a: Bounds, b: Bounds): number {
  const dx = Math.max(0, Math.max(a.minX, b.minX) - Math.min(a.maxX, b.maxX));
  const dy = Math.max(0, Math.max(a.minY, b.minY) - Math.min(a.maxY, b.maxY));
  return Math.sqrt(dx * dx + dy * dy);
}

function expand(b: Bounds, by: number): Bounds {
  return { minX: b.minX - by, minY: b.minY - by, maxX: b.maxX + by, maxY: b.maxY + by };
}

const EPS = 1e-9;

/** Geometry and profile-conformance rules over a parsed IconSpec. */
export function checkGeometry(spec: IconSpec, profile: StyleProfile): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];
  const nodes = collectAll(spec.shapes);
  const halfStroke = profile.strokeWidth / 2;
  const [vbX, vbY, vbW, vbH] = profile.viewBox;

  // profile.mismatch
  if (spec.profile !== profile.id) {
    diagnostics.push(
      err('profile.mismatch', `spec profile "${spec.profile}" does not match "${profile.id}"`, 'profile'),
    );
  }

  // limits.max-shapes (count all leaf + group nodes)
  if (nodes.length > profile.limits.maxShapes) {
    diagnostics.push(
      err(
        'limits.max-shapes',
        `${nodes.length} shapes exceed the profile limit of ${profile.limits.maxShapes}`,
        'shapes',
      ),
    );
  }

  for (const node of nodes) {
    const { shape, path } = node;

    // limits.group-depth
    if (shape.type === 'group') {
      const level = node.ancestorGroupCount + 1;
      if (level > profile.limits.maxGroupDepth) {
        diagnostics.push(
          err(
            'limits.group-depth',
            `group nesting depth ${level} exceeds the profile limit of ${profile.limits.maxGroupDepth}`,
            path,
            shape.id,
          ),
        );
      }
    }

    // limits.coord-range
    const outOfRange = rawPoints(shape).some(
      ([x, y]) =>
        x < profile.limits.coordMin ||
        x > profile.limits.coordMax ||
        y < profile.limits.coordMin ||
        y > profile.limits.coordMax,
    );
    if (outOfRange) {
      diagnostics.push(
        err(
          'limits.coord-range',
          `coordinate outside [${profile.limits.coordMin}, ${profile.limits.coordMax}]`,
          path,
          shape.id,
        ),
      );
    }

    // profile.shape-not-allowed
    if (!profile.allowedShapes.includes(shape.type)) {
      diagnostics.push(err('profile.shape-not-allowed', `shape type "${shape.type}" is not allowed`, path, shape.id));
    }

    // grid.off-grid
    const offGrid = gridCheckValues(shape).some((v) => !isOnGrid(v, profile.grid, 1e-6));
    if (offGrid) {
      diagnostics.push(
        warn('grid.off-grid', `not aligned to the ${profile.grid} grid`, path, shape.id),
      );
    }

    // rounded.radius-too-large
    if (shape.type === 'roundedRect' && shape.radius > Math.min(shape.width, shape.height) / 2) {
      diagnostics.push(
        warn('rounded.radius-too-large', `radius ${shape.radius} exceeds min(width, height) / 2`, path, shape.id),
      );
    }

    // detail.tiny-shape
    if (shape.type !== 'group') {
      const b = shapeBounds(shape);
      const w = b.maxX - b.minX;
      const h = b.maxY - b.minY;
      if (w < profile.strokeWidth && h < profile.strokeWidth) {
        diagnostics.push(
          warn('detail.tiny-shape', `bbox ${w.toFixed(3)}x${h.toFixed(3)} is smaller than the stroke width`, path, shape.id),
        );
      }
    }
  }

  // bounds.outside-viewbox / bounds.margin — top-level shapes only
  const topLevel = nodes.filter((n) => n.ancestorGroupCount === 0);
  const topLevelBounds = new Map<Node, Bounds>();
  for (const node of topLevel) {
    const raw = shapeBounds(node.shape);
    const expanded = expand(raw, halfStroke);
    topLevelBounds.set(node, expanded);

    const outsideViewbox =
      expanded.minX < vbX - EPS ||
      expanded.minY < vbY - EPS ||
      expanded.maxX > vbX + vbW + EPS ||
      expanded.maxY > vbY + vbH + EPS;

    if (outsideViewbox) {
      diagnostics.push(
        err('bounds.outside-viewbox', 'stroke-expanded bounds fall outside the viewBox', node.path, node.shape.id),
      );
      continue;
    }

    const insideMargin =
      expanded.minX < vbX + profile.margin - EPS ||
      expanded.minY < vbY + profile.margin - EPS ||
      expanded.maxX > vbX + vbW - profile.margin + EPS ||
      expanded.maxY > vbY + vbH - profile.margin + EPS;
    if (insideMargin) {
      diagnostics.push(
        warn('bounds.margin', 'stroke-expanded bounds enter the optical margin', node.path, node.shape.id),
      );
    }
  }

  // gap.too-close — pairs of top-level shapes
  for (let i = 0; i < topLevel.length; i++) {
    const nodeA = topLevel[i];
    if (nodeA === undefined) continue;
    for (let j = i + 1; j < topLevel.length; j++) {
      const nodeB = topLevel[j];
      if (nodeB === undefined) continue;
      const a = topLevelBounds.get(nodeA);
      const b = topLevelBounds.get(nodeB);
      if (a === undefined || b === undefined) continue;
      const gap = rectGap(a, b);
      if (gap > EPS && gap < profile.minGap) {
        diagnostics.push(
          warn(
            'gap.too-close',
            `gap ${gap.toFixed(3)} to "${nodeB.shape.id}" is below the minimum ${profile.minGap}`,
            nodeA.path,
            nodeA.shape.id,
          ),
        );
      }
    }
  }

  return diagnostics;
}
