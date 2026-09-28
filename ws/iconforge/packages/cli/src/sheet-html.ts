import { renderSvg } from '@iconforge/renderer';
import { summarize } from '@iconforge/quality';
import type { SheetEntry } from './sheet-collect.ts';

const SIZES = [24, 48, 96] as const;

function escapeHtml(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function renderIconCell(svg: string, size: number, colorClass: string): string {
  return `<div class="cell ${colorClass}" style="width:${size}px;height:${size}px">${svg}</div>`;
}

function renderIconRow(entry: Extract<SheetEntry, { ok: true }>): string {
  const svg = renderSvg(entry.spec, entry.profile);
  const summary = summarize(entry.diagnostics);
  const badge =
    summary.warnings > 0 ? `<span class="badge warn">${summary.warnings} warning${summary.warnings === 1 ? '' : 's'}</span>` : '';
  const lightCells = SIZES.map((size) => renderIconCell(svg, size, 'light')).join('');
  const darkCells = SIZES.map((size) => renderIconCell(svg, size, 'dark')).join('');
  return `
<section class="icon-row">
  <h3>${escapeHtml(entry.name)} ${badge}</h3>
  <div class="panels">
    <div class="panel panel-light">${lightCells}</div>
    <div class="panel panel-dark">${darkCells}</div>
  </div>
</section>`;
}

function renderRejectedRow(entry: Extract<SheetEntry, { ok: false }>): string {
  return `<li><strong>${escapeHtml(entry.name)}</strong> (${escapeHtml(entry.path)}): ${escapeHtml(entry.error.split('\n')[0] ?? entry.error)}</li>`;
}

const STYLE = `
body { font-family: system-ui, sans-serif; margin: 24px; background: #fafafa; }
.icon-row { margin-bottom: 32px; }
.icon-row h3 { margin: 0 0 8px; }
.badge { font-size: 12px; padding: 2px 8px; border-radius: 10px; margin-left: 8px; }
.badge.warn { background: #fff3cd; color: #664d03; }
.panels { display: flex; gap: 16px; }
.panel { display: flex; gap: 8px; align-items: center; padding: 12px; border-radius: 8px; }
.panel-light { background: #ffffff; color: #111111; border: 1px solid #ddd; }
.panel-dark { background: #111111; color: #eeeeee; }
.cell { display: flex; align-items: center; justify-content: center; }
.cell svg { width: 100%; height: 100%; }
.rejected { color: #842029; }
`;

/** Builds a self-contained comparison sheet: inline SVGs, no external assets, no scripts. */
export function buildSheetHtml(entries: SheetEntry[]): string {
  const valid = entries.filter((e): e is Extract<SheetEntry, { ok: true }> => e.ok);
  const rejected = entries.filter((e): e is Extract<SheetEntry, { ok: false }> => !e.ok);

  const rows = valid.map(renderIconRow).join('\n');
  const rejectedSection =
    rejected.length > 0
      ? `<section class="rejected"><h2>Rejected (${rejected.length})</h2><ul>${rejected.map(renderRejectedRow).join('')}</ul></section>`
      : '';

  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>IconForge sheet</title>
<style>${STYLE}</style>
</head>
<body>
<h1>IconForge sheet</h1>
<p>${valid.length} icon${valid.length === 1 ? '' : 's'} — ${rejected.length} rejected</p>
${rows}
${rejectedSection}
</body>
</html>
`;
}
