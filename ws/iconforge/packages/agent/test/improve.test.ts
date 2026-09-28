import { describe, expect, it } from 'vitest';
import type { Diagnostic, IconSpec } from '@iconforge/schema';
import { improveIcon } from '../src/improve.ts';
import { createMockProvider } from '../src/mock.ts';
import { profile, validSpec, fakeRender, noErrorsCheck } from './fixtures.ts';

function json(value: unknown): string {
  return JSON.stringify(value);
}

const revisedSpec: IconSpec = {
  ...validSpec,
  shapes: [...validSpec.shapes, { id: 'extra', type: 'circle', center: [12, 12], radius: 1 }],
};

describe('improveIcon', () => {
  it('accepts immediately when the reviewer reports no defects', async () => {
    const reviewer = createMockProvider([json({ defects: [] })]);
    const reviser = createMockProvider([]);
    const result = await improveIcon({
      spec: validSpec,
      brief: 'a desk',
      profile,
      reviewer,
      reviser,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('accepted');
    expect(result.spec).toBe(validSpec);
    expect(reviewer.calls).toHaveLength(1);
    expect(reviser.calls).toHaveLength(0);
  });

  it('revises once then accepts on the second review', async () => {
    const defects = [{ shapeId: 'top', observation: 'too thin', change: 'widen it' }];
    const reviewer = createMockProvider([json({ defects }), json({ defects: [] })]);
    const reviser = createMockProvider([json(revisedSpec)]);
    const result = await improveIcon({
      spec: validSpec,
      brief: 'a desk',
      profile,
      reviewer,
      reviser,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('accepted');
    expect(result.spec?.shapes).toHaveLength(4);
    expect(reviewer.calls).toHaveLength(2);
    expect(reviser.calls).toHaveLength(1);
  });

  it('never accepts a revision that introduces a geometry error; keeps the last valid spec', async () => {
    const defects = [{ shapeId: 'top', observation: 'too thin', change: 'widen it' }];
    const reviewer = createMockProvider([json({ defects }), json({ defects: [] })]);
    // Reviser returns the same broken spec on both the initial attempt and the one retry.
    const reviser = createMockProvider([json(revisedSpec), json(revisedSpec)]);
    const geometryError: Diagnostic = { severity: 'error', code: 'geometry.bad', message: 'bad', path: 'shapes' };
    const check = (spec: IconSpec): Diagnostic[] => (spec.shapes.length > 3 ? [geometryError] : []);

    const result = await improveIcon({
      spec: validSpec,
      brief: 'a desk',
      profile,
      reviewer,
      reviser,
      render: fakeRender,
      check,
      budget: { maxRounds: 3 },
    });

    expect(result.spec).toBe(validSpec);
    expect(result.stopReason).toBe('accepted');
    expect(reviser.calls).toHaveLength(2); // one explicit retry inside askForSpec
    // The failed-revision round is recorded, but the spec is still the last valid one.
    const revisionRound = result.history.find((h) => h.defects?.length === 1 && h.round === 2);
    expect(revisionRound?.spec).toBe(validSpec);
  });

  it('stops at max-rounds when defects never clear', async () => {
    const defects = [{ shapeId: 'top', observation: 'still off', change: 'fix it' }];
    const reviewer = createMockProvider([json({ defects })]);
    const reviser = createMockProvider([json(revisedSpec)]);
    const result = await improveIcon({
      spec: validSpec,
      brief: 'a desk',
      profile,
      reviewer,
      reviser,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 2 },
    });
    expect(result.stopReason).toBe('max-rounds');
    expect(result.history.length).toBeGreaterThanOrEqual(2);
  });

  it('stops on budget once the cost ceiling is reached before the next reviewer call', async () => {
    const reviewer = createMockProvider([
      (): string => json({ defects: [{ shapeId: 'top', observation: 'x', change: 'y' }] }),
    ]);
    // Inflate usage via a custom provider wrapping the mock cost.
    const costlyReviewer = {
      ...reviewer,
      async complete(req: Parameters<typeof reviewer.complete>[0]) {
        const res = await reviewer.complete(req);
        return { ...res, usage: { ...res.usage, costUsd: 10 } };
      },
    };
    const reviser = createMockProvider([json(revisedSpec)]);
    const result = await improveIcon({
      spec: validSpec,
      brief: 'a desk',
      profile,
      reviewer: costlyReviewer,
      reviser,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3, maxCostUsd: 5 },
    });
    expect(result.stopReason).toBe('budget');
    expect(reviser.calls).toHaveLength(0);
  });
});
