import { describe, expect, it } from 'vitest';
import type { Diagnostic, IconSpec } from '@iconforge/schema';
import { generateIcon, reviseIcon, HARD_MAX_ROUNDS } from '../src/loop.ts';
import { createMockProvider } from '../src/mock.ts';
import { profile, validSpec, invalidSpec, fakeRender, noErrorsCheck } from './fixtures.ts';

function json(value: unknown): string {
  return JSON.stringify(value);
}

describe('generateIcon happy path', () => {
  it('accepts the planner output when there is no reviewer', async () => {
    const provider = createMockProvider([json(validSpec)]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('accepted');
    expect(result.spec?.name).toBe('desk');
    expect(provider.calls).toHaveLength(1);
  });

  it('accepts immediately when the reviewer reports no defects', async () => {
    const provider = createMockProvider([json(validSpec)]);
    const reviewer = createMockProvider([json({ defects: [] })]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      reviewer,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('accepted');
    expect(reviewer.calls).toHaveLength(1);
  });
});

describe('planner retry', () => {
  it('recovers from invalid JSON on the explicit retry', async () => {
    const provider = createMockProvider(['not json at all', json(validSpec)]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('accepted');
    expect(provider.calls).toHaveLength(2);
    expect(result.spec?.name).toBe('desk');
  });

  it('tolerates a ```json fence on the retry response', async () => {
    const fenced = '```json\n' + json(validSpec) + '\n```';
    const provider = createMockProvider(['garbage', fenced]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('accepted');
  });

  it('fails after two schema-invalid attempts, with diagnostics', async () => {
    const provider = createMockProvider([json(invalidSpec), json(invalidSpec)]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('failed');
    expect(result.spec).toBeUndefined();
    expect(provider.calls).toHaveLength(2);
    expect(result.history[0]?.diagnostics.length).toBeGreaterThan(0);
  });
});

describe('reviewer -> reviser cycle', () => {
  it('calls the reviser when the reviewer reports defects, then accepts', async () => {
    const revised: IconSpec = {
      ...validSpec,
      shapes: [
        { id: 'top', type: 'roundedRect', origin: [3, 9], width: 18, height: 2, radius: 0.5 },
        { id: 'left-leg', type: 'line', from: [6, 11], to: [6, 19] },
        { id: 'right-leg', type: 'line', from: [18, 11], to: [18, 20] },
      ],
    };
    const provider = createMockProvider([json(validSpec), json(revised)]);
    const reviewer = createMockProvider([
      json({ defects: [{ shapeId: 'left-leg', observation: 'leg too long', change: 'shorten to y=19' }] }),
      json({ defects: [] }),
    ]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      reviewer,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('accepted');
    expect(provider.calls).toHaveLength(2); // planner + reviser
    expect(reviewer.calls).toHaveLength(2);
    const revisedEntry = result.history.find((h) => h.defects && h.defects.length > 0);
    expect(revisedEntry).toBeDefined();
  });

  it('warns in history when the reviser drops an id not listed as a defect', async () => {
    const droppedIdSpec: IconSpec = {
      ...validSpec,
      shapes: [
        { id: 'top', type: 'roundedRect', origin: [3, 9], width: 18, height: 2, radius: 0.5 },
        { id: 'left-leg', type: 'line', from: [6, 11], to: [6, 19] },
        // right-leg silently dropped even though it was not a listed defect
      ],
    };
    const provider = createMockProvider([json(validSpec), json(droppedIdSpec)]);
    const reviewer = createMockProvider([
      json({ defects: [{ shapeId: 'left-leg', observation: 'leg too long', change: 'shorten' }] }),
      json({ defects: [] }),
    ]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      reviewer,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    const warned = result.history.some((h) =>
      h.diagnostics.some((d) => d.code === 'agent.shape-id-not-preserved'),
    );
    expect(warned).toBe(true);
  });

  it('stops at max-rounds when the reviewer keeps finding defects', async () => {
    const provider = createMockProvider([json(validSpec), json(validSpec), json(validSpec), json(validSpec)]);
    const alwaysDefect = () =>
      json({ defects: [{ shapeId: 'left-leg', observation: 'still off', change: 'adjust' }] });
    const reviewer = createMockProvider([alwaysDefect, alwaysDefect, alwaysDefect, alwaysDefect]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      reviewer,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('max-rounds');
  });

  it('clamps maxRounds to the hard cap of 3 even if a larger value is requested', async () => {
    const provider = createMockProvider(Array(10).fill(json(validSpec)));
    const alwaysDefect = () => json({ defects: [{ shapeId: 'left-leg', observation: 'x', change: 'y' }] });
    const reviewer = createMockProvider(Array(10).fill(null).map(() => alwaysDefect));
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      reviewer,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 999 },
    });
    expect(result.stopReason).toBe('max-rounds');
    expect(reviewer.calls.length).toBeLessThanOrEqual(HARD_MAX_ROUNDS);
  });
});

describe('budget', () => {
  it('stops with reason budget once maxCostUsd is exhausted', async () => {
    const provider = createMockProvider([json(validSpec), json(validSpec), json(validSpec)]);
    const alwaysDefect = () => json({ defects: [{ shapeId: 'left-leg', observation: 'x', change: 'y' }] });
    const reviewer = createMockProvider(
      [alwaysDefect, alwaysDefect, alwaysDefect],
      { model: 'mock-reviewer' },
    );
    // Force a cost on the reviewer's usage by wrapping complete().
    const originalComplete = reviewer.complete.bind(reviewer);
    reviewer.complete = async (req) => {
      const res = await originalComplete(req);
      return { ...res, usage: { ...res.usage, costUsd: 1 } };
    };
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      reviewer,
      render: fakeRender,
      check: noErrorsCheck,
      budget: { maxRounds: 3, maxCostUsd: 0.5 },
    });
    expect(result.stopReason).toBe('budget');
  });
});

describe('reviseIcon standalone', () => {
  it('produces a revised spec from a defect list', async () => {
    const provider = createMockProvider([json(validSpec)]);
    const result = await reviseIcon(validSpec, 'a desk', [{ shapeId: 'left-leg', observation: 'x', change: 'y' }], {
      profile,
      provider,
      check: noErrorsCheck,
    });
    expect(result.spec?.name).toBe('desk');
    expect(provider.calls).toHaveLength(1);
  });
});

describe('quality check errors trigger the same one-retry path', () => {
  it('fails after two check-level errors', async () => {
    const failingCheck = (): Diagnostic[] => [
      { severity: 'error', code: 'quality.out-of-bounds', message: 'shape out of bounds', path: 'shapes/0' },
    ];
    const provider = createMockProvider([json(validSpec), json(validSpec)]);
    const result = await generateIcon({
      brief: 'a desk',
      profile,
      provider,
      render: fakeRender,
      check: failingCheck,
      budget: { maxRounds: 3 },
    });
    expect(result.stopReason).toBe('failed');
    expect(provider.calls).toHaveLength(2);
  });
});
