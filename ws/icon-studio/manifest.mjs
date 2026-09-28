import { referenceLineNames } from './reference-line.mjs';

const trayStates = [
  'neutral', 'sitting', 'standing', 'time-to-move',
  'paused', 'disconnected', 'goal-reached',
];

// The generator and file:// preview use this same inventory. The generator
// writes preview-manifest.js as a classic script for local browser viewing.
export const families = [
  { id: 'healthy-balance', title: 'Healthy Balance · rounded tray line', names: trayStates, sizes: [16, 24, 32], themes: ['light', 'dark'] },
  { id: 'moveup', title: 'MoveUp · instrument tray marker', names: trayStates, sizes: [16, 24, 32], themes: ['light', 'dark'] },
  { id: 'reference-line', title: 'Reference line · app icon studies', names: referenceLineNames, sizes: [24, 36, 48], themes: ['reference'] },
];
