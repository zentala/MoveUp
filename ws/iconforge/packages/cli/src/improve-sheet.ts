import { Resvg } from '@resvg/resvg-js';
import type { Defect } from '@iconforge/agent';
import type { Diagnostic } from '@iconforge/schema';

export type ImproveSheetRound = { round: number; defects: Defect[]; diagnostics: Diagnostic[] };

export type ImproveSheetEntry = {
  name: string;
  beforeSvg: string;
  afterSvg: string;
  viewBox: readonly [number, number, number, number];
  rounds: ImproveSheetRound[];
  stopReason: string;
};

const SIZES = [24, 48, 96] as const;
const CELL = 120;
const GROUP_WIDTH = CELL * SIZES.length;
const GAP = 20;
const LABEL_HEIGHT = 24;
const ROW_HEIGHT = LABEL_HEIGHT + CELL;

function escapeHtml(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function innerBody(svg: string): string {
  const match = /<svg[^>]*>([\s\S]*)<\/svg>/.exec(svg);
  return match?.[1] ?? '';
}

function sizeCells(body: string, viewBox: readonly [number, number, number, number]): string {
  const [vbX, vbY, vbW, vbH] = viewBox;
  return SIZES.map((size, i) => {
    const x = i * CELL + (CELL - size) / 2;
    const y = (CELL - size) / 2;
    return `<svg x="${x}" y="${y}" width="${size}" height="${size}" viewBox="${vbX} ${vbY} ${vbW} ${vbH}" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" color="#111111">${body}</svg>`;
  }).join('');
}

/** Builds one composed SVG: one row per icon, before-group then after-group, each at 24/48/96px. */
export function buildImproveSheetSvg(entries: ImproveSheetEntry[], withLabels: boolean): string {
  const width = GAP * 3 + GROUP_WIDTH * 2;
  const height = GAP * (entries.length + 1) + ROW_HEIGHT * entries.length;

  const rows = entries.map((entry, row) => {
    const beforeBody = innerBody(entry.beforeSvg);
    const afterBody = innerBody(entry.afterSvg);
    const y0 = GAP + row * (ROW_HEIGHT + GAP);
    const beforeX = GAP;
    const afterX = GAP * 2 + GROUP_WIDTH;
    const groupY = y0 + LABEL_HEIGHT;
    const label = withLabels
      ? `<text x="${beforeX}" y="${y0 + 16}" font-family="sans-serif" font-size="14" fill="#333">${entry.name} (${entry.stopReason})</text>` +
        `<text x="${afterX}" y="${y0 + 16}" font-family="sans-serif" font-size="14" fill="#333">after</text>`
      : '';
    return `
${label}
<rect x="${beforeX}" y="${groupY}" width="${GROUP_WIDTH}" height="${CELL}" fill="#ffffff"/>
<g transform="translate(${beforeX},${groupY})">${sizeCells(beforeBody, entry.viewBox)}</g>
<rect x="${afterX}" y="${groupY}" width="${GROUP_WIDTH}" height="${CELL}" fill="#ffffff"/>
<g transform="translate(${afterX},${groupY})">${sizeCells(afterBody, entry.viewBox)}</g>`;
  });

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
<rect x="0" y="0" width="${width}" height="${height}" fill="#fafafa"/>
${rows.join('\n')}
</svg>
`;
}

/** Rasterize the before/after sheet. Falls back to unlabeled cells if system font loading fails. */
export function renderImproveSheetPng(entries: ImproveSheetEntry[]): Buffer {
  if (entries.length === 0) {
    const empty = `<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>`;
    return new Resvg(empty).render().asPng();
  }
  try {
    const svg = buildImproveSheetSvg(entries, true);
    const resvg = new Resvg(svg, { font: { loadSystemFonts: true } });
    return resvg.render().asPng();
  } catch {
    const svg = buildImproveSheetSvg(entries, false);
    const resvg = new Resvg(svg);
    return resvg.render().asPng();
  }
}

function renderRoundLi(r: ImproveSheetRound): string {
  const defects = r.defects
    .map((d) => `<li><code>${escapeHtml(d.shapeId)}</code>: ${escapeHtml(d.observation)} → ${escapeHtml(d.change)}</li>`)
    .join('');
  return `<li>round ${r.round}: ${r.defects.length} defect${r.defects.length === 1 ? '' : 's'}${defects ? `<ul>${defects}</ul>` : ''}</li>`;
}

function renderIconRow(entry: ImproveSheetEntry): string {
  const beforeCells = SIZES.map(
    (size) => `<div class="cell" style="width:${size}px;height:${size}px">${entry.beforeSvg}</div>`,
  ).join('');
  const afterCells = SIZES.map(
    (size) => `<div class="cell" style="width:${size}px;height:${size}px">${entry.afterSvg}</div>`,
  ).join('');
  const rounds = entry.rounds.map(renderRoundLi).join('');
  return `
<section class="icon-row">
  <h3>${escapeHtml(entry.name)} <span class="badge">${escapeHtml(entry.stopReason)}</span></h3>
  <div class="panels">
    <div class="panel"><span class="panel-label">before</span>${beforeCells}</div>
    <div class="panel"><span class="panel-label">after</span>${afterCells}</div>
  </div>
  <ol class="rounds">${rounds}</ol>
</section>`;
}

const STYLE = `
body { font-family: system-ui, sans-serif; margin: 24px; background: #fafafa; }
.icon-row { margin-bottom: 32px; }
.icon-row h3 { margin: 0 0 8px; }
.badge { font-size: 12px; padding: 2px 8px; border-radius: 10px; margin-left: 8px; background: #e7f1ff; color: #084298; }
.panels { display: flex; gap: 24px; }
.panel { display: flex; gap: 8px; align-items: center; padding: 12px; border-radius: 8px; background: #ffffff; color: #111111; border: 1px solid #ddd; }
.panel-label { font-size: 12px; color: #666; width: 48px; }
.cell { display: flex; align-items: center; justify-content: center; }
.cell svg { width: 100%; height: 100%; }
.rounds { font-size: 13px; color: #333; }
.rounds ul { margin: 4px 0; }
`;

/** Builds a self-contained before/after comparison sheet with the per-round defect list. */
export function buildImproveSheetHtml(entries: ImproveSheetEntry[]): string {
  const rows = entries.map(renderIconRow).join('\n');
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>IconForge improve sheet</title>
<style>${STYLE}</style>
</head>
<body>
<h1>IconForge improve sheet</h1>
<p>${entries.length} icon${entries.length === 1 ? '' : 's'}</p>
${rows}
</body>
</html>
`;
}
