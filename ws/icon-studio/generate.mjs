import { fileURLToPath } from 'node:url';
import { dirname } from 'node:path';
import { referenceLine } from './reference-line.mjs';
import { families } from './manifest.mjs';
import { writeIconSet } from './generator-io.mjs';

const root = dirname(fileURLToPath(import.meta.url));

const hbDesk = '<path d="M3.5 8h17M5.5 8v11M18.5 8v11M4.5 19h2M17.5 19h2"/>';
const hbMarks = {
  neutral: '<path d="M12 12v3"/>',
  sitting: '<path d="M12 11v5m-2-2 2 2 2-2"/>',
  standing: '<path d="M12 16v-5m-2 2 2-2 2 2"/>',
  'time-to-move': '<path d="M9.5 13.5h5m-2-2 2 2-2 2"/>',
  paused: '<path d="M10.5 11.5v4M13.5 11.5v4"/>',
  disconnected: '<path d="m9.5 11.5 5 5m0-5-5 5"/>',
  'goal-reached': '<path d="m9.5 13.5 1.8 1.8 3.5-3.5"/>',
};

function healthyBalance(state) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" role="img" aria-label="${state}">\n  <title>${state}</title>\n  ${hbDesk}\n  ${hbMarks[state]}\n</svg>\n`;
}

// This proposal starts from the live MoveUp tray's white desk and status dot,
// then substitutes a tiny geometric mark where a dot cannot name the state.
const moveupDesk = '<path d="M2 5h20v3H2zM4 8h2v9H4zM18 8h2v9h-2z" fill="#F0EBE0" stroke="#403830" stroke-width="0.8"/>';
const moveupMarks = {
  neutral: '<rect x="10" y="13" width="4" height="4" fill="#706858"/>',
  sitting: '<path d="M10 13h4v4h-4z" fill="#65A30D"/>',
  standing: '<path d="M12 12l4 5H8z" fill="#D97706"/>',
  'time-to-move': '<path d="M10 12h4v3h-4zM10 16h4v2h-4z" fill="#B91C1C"/>',
  paused: '<path d="M10 12h1.5v6H10zM12.5 12H14v6h-1.5z" fill="#706858"/>',
  disconnected: '<path d="M9 12h6v6H9z" fill="#4A7C9E"/><path d="m9 18 6-6" stroke="#141210" stroke-width="1.5"/>',
  'goal-reached': '<path d="M8.5 12h7v6h-7z" fill="#D97706"/><path d="m10 15 1.3 1.3 2.7-2.7" fill="none" stroke="#141210" stroke-width="1.5" stroke-linecap="square"/>',
};

function moveup(state) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" role="img" aria-label="${state}">\n  <title>${state}</title>\n  ${moveupDesk}\n  ${moveupMarks[state]}\n</svg>\n`;
}

await writeIconSet(root, families, {
  'healthy-balance': healthyBalance,
  moveup,
  'reference-line': referenceLine,
});
