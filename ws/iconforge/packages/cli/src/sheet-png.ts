import { Resvg } from '@resvg/resvg-js';
import { renderSvg } from '@iconforge/renderer';
import type { SheetEntry } from './sheet-collect.ts';

const SIZES = [24, 48, 96] as const;
const CELL = 120;
const PANEL_WIDTH = CELL * SIZES.length;
const GAP = 20;
const LABEL_HEIGHT = 24;
const ROW_HEIGHT = LABEL_HEIGHT + CELL;

function innerBody(svg: string): string {
  const match = /<svg[^>]*>([\s\S]*)<\/svg>/.exec(svg);
  return match?.[1] ?? '';
}

function panelCells(body: string, viewBox: readonly [number, number, number, number], color: string): string {
  const [vbX, vbY, vbW, vbH] = viewBox;
  return SIZES.map((size, i) => {
    const x = i * CELL + (CELL - size) / 2;
    const y = (CELL - size) / 2;
    return `<svg x="${x}" y="${y}" width="${size}" height="${size}" viewBox="${vbX} ${vbY} ${vbW} ${vbH}" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" color="${color}">${body}</svg>`;
  }).join('');
}

/** Builds one composed SVG document containing every icon at 3 sizes on light + dark panels. */
export function buildSheetSvg(entries: Array<Extract<SheetEntry, { ok: true }>>, withLabels: boolean): string {
  const width = GAP * 3 + PANEL_WIDTH * 2;
  const height = GAP * (entries.length + 1) + ROW_HEIGHT * entries.length;

  const rows = entries.map((entry, row) => {
    const svg = renderSvg(entry.spec, entry.profile);
    const body = innerBody(svg);
    const y0 = GAP + row * (ROW_HEIGHT + GAP);
    const lightX = GAP;
    const darkX = GAP * 2 + PANEL_WIDTH;
    const panelY = y0 + LABEL_HEIGHT;
    const label = withLabels
      ? `<text x="${lightX}" y="${y0 + 16}" font-family="sans-serif" font-size="14" fill="#333">${entry.name}</text>`
      : '';
    return `
${label}
<rect x="${lightX}" y="${panelY}" width="${PANEL_WIDTH}" height="${CELL}" fill="#ffffff"/>
<g transform="translate(${lightX},${panelY})">${panelCells(body, entry.profile.viewBox, '#111111')}</g>
<rect x="${darkX}" y="${panelY}" width="${PANEL_WIDTH}" height="${CELL}" fill="#111111"/>
<g transform="translate(${darkX},${panelY})">${panelCells(body, entry.profile.viewBox, '#eeeeee')}</g>`;
  });

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
<rect x="0" y="0" width="${width}" height="${height}" fill="#fafafa"/>
${rows.join('\n')}
</svg>
`;
}

/** Rasterize the sheet. Tries system-font labels first; falls back to unlabeled cells if font loading fails. */
export function renderSheetPng(entries: Array<Extract<SheetEntry, { ok: true }>>): Buffer {
  if (entries.length === 0) {
    const empty = `<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>`;
    return new Resvg(empty).render().asPng();
  }
  try {
    const svg = buildSheetSvg(entries, true);
    const resvg = new Resvg(svg, { font: { loadSystemFonts: true } });
    return resvg.render().asPng();
  } catch {
    const svg = buildSheetSvg(entries, false);
    const resvg = new Resvg(svg);
    return resvg.render().asPng();
  }
}
