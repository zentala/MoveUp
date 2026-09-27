import { describe, expect, it, afterEach } from 'vitest';
import { mkdtemp, readFile, writeFile, mkdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createMockProvider } from '@iconforge/agent';
import { runImprove } from './improve.ts';
import { EXIT } from '../util.ts';

const SPEC_A = {
  version: 1,
  name: 'a-icon',
  profile: 'outline-24-v1',
  shapes: [{ id: 's', type: 'circle', center: [12, 12], radius: 4 }],
};

const originalKey = process.env.OPENROUTER_API_KEY;
afterEach(() => {
  if (originalKey === undefined) delete process.env.OPENROUTER_API_KEY;
  else process.env.OPENROUTER_API_KEY = originalKey;
});

function json(value: unknown): string {
  return JSON.stringify(value);
}

describe('runImprove', () => {
  it('writes before/after artifacts, rounds, history and the sheet for an accepted-immediately icon', async () => {
    process.env.OPENROUTER_API_KEY = 'sk-test-secret-value';
    const mock = createMockProvider([json({ defects: [] })]);
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-improve-'));
    try {
      const inDir = join(dir, 'in');
      await mkdir(inDir, { recursive: true });
      await writeFile(join(inDir, 'a-icon.json'), JSON.stringify(SPEC_A));
      await writeFile(join(inDir, 'briefs.json'), JSON.stringify({ 'a-icon': 'a small circle' }));

      const out = join(dir, 'out');
      const code = await runImprove([join(inDir, 'a-icon.json'), '--out', out], { providerFactory: () => mock });
      expect(code).toBe(EXIT.OK);

      const iconDir = join(out, 'a-icon');
      await readFile(join(iconDir, 'before.svg'), 'utf8');
      await readFile(join(iconDir, 'before-24-light.png'));
      await readFile(join(iconDir, 'before-512-light.png'));
      await readFile(join(iconDir, 'after.svg'), 'utf8');
      await readFile(join(iconDir, 'after-24-light.png'));
      await readFile(join(iconDir, 'after-512-light.png'));

      const afterJson = JSON.parse(await readFile(join(iconDir, 'after.json'), 'utf8'));
      expect(afterJson).toMatchObject({ name: 'a-icon' });

      const historyText = await readFile(join(iconDir, 'history.json'), 'utf8');
      expect(historyText).not.toContain('sk-test-secret-value');
      const history = JSON.parse(historyText) as { stopReason: string };
      expect(history.stopReason).toBe('accepted');

      const sheetHtml = await readFile(join(out, 'improve-sheet.html'), 'utf8');
      expect(sheetHtml).toContain('a-icon');
      await readFile(join(out, 'improve-sheet.png'));

      expect(mock.calls).toHaveLength(1);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('writes rounds/<n>.json when the icon goes through a revision', async () => {
    process.env.OPENROUTER_API_KEY = 'sk-test-secret-value';
    const revised = { ...SPEC_A, shapes: [{ id: 's', type: 'circle', center: [12, 12], radius: 5 }] };
    // Distinct provider per role (reviewer vs. reviser), selected by --model / --reviewer-model,
    // the same way the CLI wires two independent LlmProviders.
    const reviewer = createMockProvider([
      json({ defects: [{ shapeId: 's', observation: 'too small', change: 'grow it' }] }),
      json({ defects: [] }),
    ]);
    const reviser = createMockProvider([json(revised)]);

    const dir = await mkdtemp(join(tmpdir(), 'iconforge-improve-revise-'));
    try {
      const inDir = join(dir, 'in');
      await mkdir(inDir, { recursive: true });
      await writeFile(join(inDir, 'a-icon.json'), JSON.stringify(SPEC_A));

      const out = join(dir, 'out');
      const code = await runImprove(
        [
          join(inDir, 'a-icon.json'),
          '--brief',
          'a circle',
          '--out',
          out,
          '--model',
          'reviser-model',
          '--reviewer-model',
          'reviewer-model',
        ],
        {
          providerFactory: ({ model }) => (model === 'reviewer-model' ? reviewer : reviser),
        },
      );
      expect(code).toBe(EXIT.OK);

      const round2 = await readFile(join(out, 'a-icon', 'rounds', '2.json'), 'utf8');
      expect(JSON.parse(round2)).toMatchObject({ shapes: [{ radius: 5 }] });
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('exits 2 when OPENROUTER_API_KEY is missing', async () => {
    delete process.env.OPENROUTER_API_KEY;
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-improve-nokey-'));
    try {
      const inDir = join(dir, 'in');
      await mkdir(inDir, { recursive: true });
      await writeFile(join(inDir, 'a-icon.json'), JSON.stringify(SPEC_A));
      const out = join(dir, 'out');
      const code = await runImprove([join(inDir, 'a-icon.json'), '--out', out]);
      expect(code).toBe(EXIT.USAGE);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('rejects an invalid --complexity value', async () => {
    process.env.OPENROUTER_API_KEY = 'sk-test-secret-value';
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-improve-badcomplexity-'));
    try {
      const inDir = join(dir, 'in');
      await mkdir(inDir, { recursive: true });
      await writeFile(join(inDir, 'a-icon.json'), JSON.stringify(SPEC_A));
      const out = join(dir, 'out');
      await expect(
        runImprove([join(inDir, 'a-icon.json'), '--out', out, '--complexity', 'bogus'], {
          providerFactory: () => createMockProvider([json({ defects: [] })]),
        }),
      ).rejects.toThrow();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
