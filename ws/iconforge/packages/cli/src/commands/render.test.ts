import { describe, expect, it } from 'vitest';
import { fileURLToPath } from 'node:url';
import { mkdtemp, readFile, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { runRender } from './render.ts';
import { EXIT } from '../util.ts';

const DESK = fileURLToPath(new URL('../../../../examples/icons/moveup/desk.json', import.meta.url));
const BAD = fileURLToPath(new URL('../../test/fixtures/bad.json', import.meta.url));

describe('runRender', () => {
  it('writes all expected files and is byte-identical across runs', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-render-'));
    try {
      const outA = join(dir, 'a');
      const outB = join(dir, 'b');
      const codeA = await runRender([DESK, '--out', outA]);
      const codeB = await runRender([DESK, '--out', outB]);
      expect(codeA).toBe(EXIT.OK);
      expect(codeB).toBe(EXIT.OK);

      const files = (await readdir(outA)).sort();
      expect(files).toEqual(
        ['icon.svg', 'preview-24-dark.png', 'preview-24-light.png', 'preview-512-dark.png', 'preview-512-light.png', 'report.json', 'spec.json'].sort(),
      );

      const svgA = await readFile(join(outA, 'icon.svg'), 'utf8');
      const svgB = await readFile(join(outB, 'icon.svg'), 'utf8');
      expect(svgA).toBe(svgB);
      expect(svgA).toContain('currentColor');
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('on validation errors writes nothing and exits 1', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-render-bad-'));
    try {
      const out = join(dir, 'out');
      const code = await runRender([BAD, '--out', out]);
      expect(code).toBe(EXIT.VALIDATION);
      await expect(readdir(out)).rejects.toThrow();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
