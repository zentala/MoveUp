import { describe, expect, it } from 'vitest';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { runSheet } from './sheet.ts';
import { EXIT } from '../util.ts';

const VALID_A = {
  version: 1,
  name: 'a-icon',
  profile: 'outline-24-v1',
  shapes: [{ id: 's', type: 'circle', center: [12, 12], radius: 4 }],
};
const VALID_B = {
  version: 1,
  name: 'b-icon',
  profile: 'outline-24-v1',
  shapes: [{ id: 's', type: 'rect', origin: [4, 4], width: 16, height: 16 }],
};
const INVALID = { version: 1, name: 'c-icon', profile: 'outline-24-v1', shapes: [] };

describe('runSheet', () => {
  it('lists a rejected icon and renders the valid ones', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-sheet-'));
    try {
      const inDir = join(dir, 'in');
      await import('node:fs/promises').then((fs) => fs.mkdir(inDir, { recursive: true }));
      await writeFile(join(inDir, 'a-icon.json'), JSON.stringify(VALID_A));
      await writeFile(join(inDir, 'b-icon.json'), JSON.stringify(VALID_B));
      await writeFile(join(inDir, 'c-icon.json'), JSON.stringify(INVALID));

      const outDir = join(dir, 'out');
      const code = await runSheet([inDir, '--out', outDir]);
      expect(code).toBe(EXIT.OK);

      const html = await readFile(join(outDir, 'sheet.html'), 'utf8');
      expect(html).toContain('a-icon');
      expect(html).toContain('b-icon');
      expect(html).toContain('Rejected');
      expect(html).toContain('c-icon');

      const png = await readFile(join(outDir, 'sheet.png'));
      expect(png.length).toBeGreaterThan(0);
      expect(png.subarray(0, 8).toString('hex')).toBe('89504e470d0a1a0a'); // PNG signature
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
