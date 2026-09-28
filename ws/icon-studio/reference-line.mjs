/**
 * Hand-authored vector studies after the three June 8, 2026 Healthy Balance
 * reference boards. These are new drawings, not automated raster traces.
 *
 * Paths use a 48-unit grid so an agent can edit the geometry directly. The
 * source files are SVG-ready: the studio generator writes each one verbatim.
 */

const ink = '#F5F7FA';
const blue = '#4DA3FF';
const green = '#7ED957';
const amber = '#FFB84D';
const red = '#FF6868';

const desk = `
  <rect x="8" y="22" width="32" height="3" rx="1"/>
  <path d="M12 25v15m-4 0h8M36 25v15m-4 0h8M10 27h4m20 0h4"/>`;
const shortDesk = `
  <rect x="5" y="22" width="22" height="3" rx="1"/>
  <path d="M9 25v15m-4 0h8M23 25v15m-4 0h8"/>`;
const seatedPerson = `
  <circle cx="35" cy="12" r="3"/>
  <path d="M34 16c-2 1-3 3-3 6v6l-3 5m5-5 7 1v6h-7m-5-2h-3m15 2h3m-3-6v11"/>`;
const standingPerson = `
  <circle cx="36" cy="12" r="3"/>
  <path d="M35 17c-2 0-3 2-3 4v9l-2 10m7-10 2 10m-7-19-3 6m10-6 3 6"/>`;

const drawings = {
  desk,
  raise: `${desk}<path d="M24 17V6m-5 5 5-5 5 5"/>`,
  lower: `${desk}<path d="M24 6v11m-5-5 5 5 5-5"/>`,
  'sit-stand-cycle': `${desk}<path d="M19 17V7m-4 4 4-4 4 4M29 7v10m-4-4 4 4 4-4"/>`,
  'desk-pulse': `${desk}<path d="M15 16c2-3 5-3 7 0s5 3 7 0 5-3 7 0" stroke="${blue}"/>`,
  'smart-nudge': `${desk}<path d="M4 13c-2 3-2 6 0 9m40-9c2 3 2 6 0 9" stroke="${blue}"/><path d="M8 15c-1 2-1 4 0 6m32-6c1 2 1 4 0 6" stroke="${blue}"/>`,
  sitting: `${shortDesk}${seatedPerson}`,
  standing: `${shortDesk}${standingPerson}<path d="M29 15v-8m-3 3 3-3 3 3" stroke="${green}"/>`,
  'time-to-move': `${shortDesk}${seatedPerson}<circle cx="42" cy="7" r="4" stroke="${amber}"/><path d="M42 5v2l1.5 1" stroke="${amber}"/>`,
  away: `${desk}<circle cx="24" cy="13" r="5" stroke="${blue}" stroke-dasharray="2 3"/>`,
  paused: `${desk}<circle cx="24" cy="12" r="7"/><path d="M22 9v6m4-6v6"/>`,
  connected: `${desk}<circle cx="38" cy="36" r="4" stroke="${green}"/><path d="m36.5 36 1 1 2-2" stroke="${green}"/>`,
  disconnected: `${desk}<circle cx="38" cy="36" r="4" stroke="${red}"/><path d="m36.5 34.5 3 3m0-3-3 3" stroke="${red}"/>`,
  'goal-reached': `${desk}<circle cx="24" cy="12" r="7" stroke="${green}"/><path d="m20.5 12 2.5 2.5 4.5-5" stroke="${green}"/>`,
};

export const referenceLineNames = Object.keys(drawings);

export function referenceLine(name) {
  const paths = drawings[name];
  if (!paths) throw new Error(`Unknown reference-line icon: ${name}`);
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none" stroke="${ink}" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" role="img" aria-label="${name}">\n  <title>${name}</title>\n  ${paths.trim()}\n</svg>\n`;
}
