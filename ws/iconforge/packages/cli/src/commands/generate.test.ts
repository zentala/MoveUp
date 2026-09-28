import { describe, expect, it, afterEach } from 'vitest';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createMockProvider } from '@iconforge/agent';
import { runGenerate } from './generate.ts';
import { EXIT } from '../util.ts';

const VALID_SPEC = {
  version: 1,
  name: 'gen-icon',
  profile: 'outline-24-v1',
  shapes: [{ id: 's', type: 'circle', center: [12, 12], radius: 4 }],
};

const originalKey = process.env.OPENROUTER_API_KEY;
afterEach(() => {
  if (originalKey === undefined) delete process.env.OPENROUTER_API_KEY;
  else process.env.OPENROUTER_API_KEY = originalKey;
});

describe('runGenerate', () => {
  it('writes history.json without secrets using an injected mock provider', async () => {
    process.env.OPENROUTER_API_KEY = 'sk-test-secret-value';
    const mock = createMockProvider([JSON.stringify(VALID_SPEC), JSON.stringify({ defects: [] })]);
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-generate-'));
    try {
      const out = join(dir, 'out');
      const code = await runGenerate(['a small round icon', '--out', out], {
        providerFactory: () => mock,
      });
      expect(code).toBe(EXIT.OK);

      const historyText = await readFile(join(out, 'history.json'), 'utf8');
      expect(historyText).not.toContain('sk-test-secret-value');
      expect(historyText).not.toContain('OPENROUTER_API_KEY');
      const history = JSON.parse(historyText) as { stopReason: string };
      expect(history.stopReason).toBe('accepted');

      const specText = await readFile(join(out, 'spec.json'), 'utf8');
      expect(JSON.parse(specText)).toMatchObject({ name: 'gen-icon' });

      const briefText = await readFile(join(out, 'brief.txt'), 'utf8');
      expect(briefText).toContain('a small round icon');
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('exits 2 when OPENROUTER_API_KEY is missing', async () => {
    delete process.env.OPENROUTER_API_KEY;
    const dir = await mkdtemp(join(tmpdir(), 'iconforge-generate-nokey-'));
    try {
      const out = join(dir, 'out');
      const code = await runGenerate(['a brief', '--out', out]);
      expect(code).toBe(EXIT.USAGE);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
