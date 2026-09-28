import type { IconSpec, StyleProfile, Diagnostic } from '@iconforge/schema';
import type { LlmProvider, Usage } from './provider.ts';
import { extractJson } from './json-extract.ts';
import { reviewerPrompt, ReviewerOutputSchema, type Defect } from './prompts.ts';
import { askForSpec, addUsage, emptyUsage, budgetExceeded, timedOut, makeTimeoutSignal, REVIEW_MAX_TOKENS } from './loop-helpers.ts';
import { reviserPrompt } from './prompts.ts';
import type { RenderResult, GenerateResult, LoopEvent, LoopHistoryEntry, StopReason, GenerateBudget } from './loop.ts';
import { HARD_MAX_ROUNDS } from './loop.ts';

export type ImproveIconOptions = {
  spec: IconSpec;
  brief: string;
  profile: StyleProfile;
  reviewer: LlmProvider;
  reviser: LlmProvider;
  render: (spec: IconSpec) => RenderResult;
  check: (spec: IconSpec) => Diagnostic[];
  budget: GenerateBudget;
  onEvent?: (event: LoopEvent) => void;
};

/**
 * Review -> revise loop for an EXISTING IconSpec (the CLI `improve` command).
 * Unlike `generateIcon`, there is no planner round: `opts.spec` is round 0.
 * A revision that fails geometry validation (even after the one explicit
 * retry `askForSpec` performs) is never accepted — the previous valid spec
 * is kept and the loop proceeds to the next round.
 */
export async function improveIcon(opts: ImproveIconOptions): Promise<GenerateResult> {
  const startedAtMs = Date.now();
  const maxRounds = Math.min(opts.budget.maxRounds ?? HARD_MAX_ROUNDS, HARD_MAX_ROUNDS);
  const history: LoopHistoryEntry[] = [];
  let totalUsage = emptyUsage();
  let currentSpec = opts.spec;

  const emit = (event: LoopEvent): void => opts.onEvent?.(event);
  const signal = makeTimeoutSignal(opts.budget.timeoutMs);

  const stop = (reason: StopReason): GenerateResult => {
    emit({ type: 'stop', reason });
    return { spec: currentSpec, history, stopReason: reason, totalUsage };
  };

  for (let round = 1; round <= maxRounds; round += 1) {
    if (timedOut(opts.budget.timeoutMs, startedAtMs)) return stop('timeout');
    if (budgetExceeded(opts.budget.maxCostUsd, totalUsage.costUsd ?? 0)) return stop('budget');

    const review = await runReview(opts, currentSpec, round, signal, emit);
    totalUsage = addUsage(totalUsage, review.usage);
    history.push(review.historyEntry);

    if (review.defects.length === 0) return stop('accepted');

    if (timedOut(opts.budget.timeoutMs, startedAtMs)) return stop('timeout');
    if (budgetExceeded(opts.budget.maxCostUsd, totalUsage.costUsd ?? 0)) return stop('budget');

    emit({ type: 'reviser-start', round });
    const revised = await askForSpec(
      opts.reviser,
      'You are an IconForge reviser. Follow the instructions in the user message exactly.',
      [
        {
          role: 'user',
          content: reviserPrompt({ brief: opts.brief, profile: opts.profile, spec: currentSpec, defects: review.defects }),
        },
      ],
      opts.check,
      signal,
    );
    totalUsage = addUsage(totalUsage, revised.usage);
    emit({ type: 'reviser-result', round, ok: revised.spec !== undefined, diagnostics: revised.diagnostics });

    if (revised.spec === undefined) {
      // Revision introduced geometry errors even after the retry: keep the last valid spec.
      history.push({ round: round + 1, spec: currentSpec, diagnostics: revised.diagnostics, defects: review.defects, usage: revised.usage });
      continue;
    }

    history.push({ round: round + 1, spec: revised.spec, diagnostics: revised.diagnostics, defects: review.defects, usage: revised.usage });
    currentSpec = revised.spec;
  }

  return stop('max-rounds');
}

async function runReview(
  opts: ImproveIconOptions,
  currentSpec: IconSpec,
  round: number,
  signal: AbortSignal | undefined,
  emit: (event: LoopEvent) => void,
): Promise<{ defects: Defect[]; usage: Usage; historyEntry: LoopHistoryEntry }> {
  const rendered = opts.render(currentSpec);
  const geometryDiagnostics = opts.check(currentSpec);
  emit({ type: 'reviewer-start', round });
  const reviewerResult = await opts.reviewer.complete({
    maxTokens: REVIEW_MAX_TOKENS,
    system: 'You are an IconForge reviewer. Follow the instructions in the user message exactly.',
    messages: [
      {
        role: 'user',
        content: [
          {
            type: 'text',
            text: reviewerPrompt({ brief: opts.brief, profile: opts.profile, diagnostics: geometryDiagnostics }),
          },
          { type: 'image', pngBase64: rendered.png24.toString('base64') },
          { type: 'image', pngBase64: rendered.png512.toString('base64') },
        ],
      },
    ],
    signal,
  });

  const extracted = extractJson(reviewerResult.text);
  const parsed = extracted.ok ? ReviewerOutputSchema.safeParse(extracted.value) : undefined;
  const defects: Defect[] = parsed?.success ? parsed.data.defects : [];
  const reviewDiagnostics: Diagnostic[] =
    !extracted.ok || !parsed?.success
      ? [
          {
            severity: 'warning',
            code: 'agent.reviewer-output-invalid',
            message: 'reviewer output could not be parsed; treated as no defects',
            path: '',
          },
        ]
      : geometryDiagnostics;

  emit({ type: 'reviewer-result', round, defects });

  return {
    defects,
    usage: reviewerResult.usage,
    historyEntry: { round, spec: currentSpec, diagnostics: reviewDiagnostics, defects, usage: reviewerResult.usage },
  };
}
