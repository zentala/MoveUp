import { describe, expect, it } from 'vitest';
import { fileURLToPath } from 'node:url';
import { runValidateFile } from './validate.ts';

const DESK = fileURLToPath(new URL('../../../../examples/icons/moveup/desk.json', import.meta.url));
const BAD = fileURLToPath(new URL('../../test/fixtures/bad.json', import.meta.url));

describe('runValidateFile', () => {
  it('accepts a valid spec', async () => {
    const result = await runValidateFile(DESK, undefined);
    expect(result.ok).toBe(true);
    expect(result.diagnostics).toHaveLength(0);
  });

  it('rejects an out-of-range spec with a located diagnostic', async () => {
    const result = await runValidateFile(BAD, undefined);
    expect(result.ok).toBe(false);
    expect(result.diagnostics.length).toBeGreaterThan(0);
    const first = result.diagnostics[0];
    expect(first?.shapeId).toBe('oob');
    expect(first?.severity).toBe('error');
  });
});
