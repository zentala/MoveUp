import { describe, expect, it, afterEach } from 'vitest';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createMockProvider } from '@iconforge/agent';
import { runRevise } from './revise.ts';
import { EXIT } from '../util.ts';

const VALID_SPEC = {
  version: 1,
  name: 'gen-icon',
  profile: 'outline-24-v1',
  shapes: [{ id: 's', type: 'circle', center: [12, 12], radius: 4 }],
};
const REVISED_SPEC = {
  ...VALID_SPEC,
  shapes: [{ id: 's', type: 'circle', center: [12, 12], radius: 5 }],
};

const originalKey = process.env.OPENROUTER_API_KEY;
afterEach(() => {
  if (originalKey === undefined) delete process.env.OPENROUTER_API_KEY;
  else process.env.OPENROUTER_API_KEY = originalKey;
});

describe('runRevise', () => {
  it('revises a spec against free-text defects and writes the outcome, no secrets', async () => {
    process.env.OPENROUTER_API_KEY = 'sk-test-secret-value';
    const mock = createMockProvider([JSON.stringify(REVISED_SPEC)]);
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-revise-'));
    try {
      const specPath = join(dir, 'spec.json');
      await writeFile(specPath, JSON.stringify(VALID_SPEC));
      const out = join(dir, 'out');

      const code = await runRevise(
        [specPath, '--defects', 'circle is too small', '--brief', 'a small round icon', '--out', out],
        { providerFactory: () => mock },
      );
      expect(code).toBe(EXIT.OK);

      const defectsText = await readFile(join(out, 'defects.json'), 'utf8');
      expect(JSON.parse(defectsText)).toMatchObject([{ observation: 'circle is too small' }]);

      const specText = await readFile(join(out, 'spec.json'), 'utf8');
      expect(JSON.parse(specText)).toMatchObject({ name: 'gen-icon' });

      const historyText = await readFile(join(out, 'history.json'), 'utf8');
      expect(historyText).not.toContain('sk-test-secret-value');
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('reads a defects.json array file', async () => {
    process.env.OPENROUTER_API_KEY = 'sk-test-secret-value';
    const mock = createMockProvider([JSON.stringify(REVISED_SPEC)]);
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-revise-'));
    try {
      const specPath = join(dir, 'spec.json');
      await writeFile(specPath, JSON.stringify(VALID_SPEC));
      const defectsPath = join(dir, 'defects.json');
      await writeFile(defectsPath, JSON.stringify([{ shapeId: 's', observation: 'small', change: 'grow it' }]));
      const out = join(dir, 'out');

      const code = await runRevise([specPath, '--defects', defectsPath, '--brief', 'a small round icon', '--out', out], {
        providerFactory: () => mock,
      });
      expect(code).toBe(EXIT.OK);

      const defectsText = await readFile(join(out, 'defects.json'), 'utf8');
      expect(JSON.parse(defectsText)).toEqual([{ shapeId: 's', observation: 'small', change: 'grow it' }]);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('exits 2 when OPENROUTER_API_KEY is missing', async () => {
    delete process.env.OPENROUTER_API_KEY;
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-revise-nokey-'));
    try {
      const specPath = join(dir, 'spec.json');
      await writeFile(specPath, JSON.stringify(VALID_SPEC));
      const out = join(dir, 'out');
      const code = await runRevise([specPath, '--defects', 'x', '--brief', 'b', '--out', out]);
      expect(code).toBe(EXIT.USAGE);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('exits USAGE when the spec file does not validate', async () => {
    process.env.OPENROUTER_API_KEY = 'sk-test-secret-value';
    const mock = createMockProvider([JSON.stringify(REVISED_SPEC)]);
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-revise-badspec-'));
    try {
      const specPath = join(dir, 'spec.json');
      await writeFile(specPath, JSON.stringify({ version: 1, name: 'bad', profile: 'outline-24-v1', shapes: [] }));
      const out = join(dir, 'out');
      await expect(
        runRevise([specPath, '--defects', 'x', '--brief', 'b', '--out', out], { providerFactory: () => mock }),
      ).rejects.toThrow();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
