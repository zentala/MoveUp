import type { IconSpec } from '@iconforge/schema';
import { OUTLINE_24_V1 } from '@iconforge/schema';

export const profile = OUTLINE_24_V1;

export const validSpec: IconSpec = {
  version: 1,
  name: 'desk',
  profile: 'outline-24-v1',
  shapes: [
    { id: 'top', type: 'roundedRect', origin: [3, 9], width: 18, height: 2, radius: 0.5 },
    { id: 'left-leg', type: 'line', from: [6, 11], to: [6, 20] },
    { id: 'right-leg', type: 'line', from: [18, 11], to: [18, 20] },
  ],
};

export const invalidSpec = {
  version: 1,
  name: 'bad',
  profile: 'outline-24-v1',
  shapes: [{ id: 'x', type: 'circle', center: [0, 0], radius: -1 }],
};

export function fakeRender(): { svg: string; png24: Buffer; png512: Buffer } {
  return { svg: '<svg/>', png24: Buffer.from('24'), png512: Buffer.from('512') };
}

export function noErrorsCheck(): [] {
  return [];
}
