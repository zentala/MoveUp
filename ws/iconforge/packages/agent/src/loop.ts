import type { IconSpec, StyleProfile, Diagnostic } from '@iconforge/schema';
import type { LlmProvider, Usage } from './provider.ts';
import { extractJson } from './json-extract.ts';
import { plannerPrompt, reviserPrompt, reviewerPrompt, ReviewerOutputSchema, type Defect, type Complexity } from './prompts.ts';
import {
  askForSpec,
  addUsage,
  emptyUsage,
  budgetExceeded,
  timedOut,
  unchangedShapesPreservedWarning,
  makeTimeoutSignal,
  REVIEW_MAX_TOKENS,
} from './loop-helpers.ts';

export const HARD_MAX_ROUNDS = 3;

export type RenderResult = { svg: string; png24: Buffer; png512: Buffer };

export type GenerateBudget = {
  /** Default 3. Always clamped to the hard cap of 3. */
  maxRounds?: number;
  maxCostUsd?: number;
  timeoutMs?: number;
};

export type LoopEvent =
  | { type: 'planner-start'; round: number }
  | { type: 'planner-result'; round: number; ok: boolean; diagnostics: Diagnostic[] }
  | { type: 'reviewer-start'; round: number }
  | { type: 'reviewer-result'; round: number; defects: Defect[] }
  | { type: 'reviser-start'; round: number }
  | { type: 'reviser-result'; round: number; ok: boolean; diagnostics: Diagnostic[] }
  | { type: 'stop'; reason: StopReason };

export type StopReason = 'accepted' | 'max-rounds' | 'budget' | 'timeout' | 'failed';

export type LoopHistoryEntry = {
  round: number;
  spec: IconSpec | undefined;
  diagnostics: Diagnostic[];
  defects?: Defect[];
  usage: Usage;
};

export type GenerateResult = {
  spec: IconSpec | undefined;
  history: LoopHistoryEntry[];
  stopReason: StopReason;
  totalUsage: Usage;
};

export type GenerateIconOptions = {
  brief: string;
  profile: StyleProfile;
  examples?: IconSpec[];
  /** Shape-count budget for the planner. Default 'auto': the model picks the smallest workable count. */
  complexity?: Complexity;
  provider: LlmProvider;
  reviewer?: LlmProvider;
  render: (spec: IconSpec) => RenderResult;
  check: (spec: IconSpec) => Diagnostic[];
  budget: GenerateBudget;
  onEvent?: (event: LoopEvent) => void;
};

/** Full generate loop: planner -> validate (+1 retry) -> optional vision review -> reviser -> repeat. */
export async function generateIcon(opts: GenerateIconOptions): Promise<GenerateResult> {
  const startedAtMs = Date.now();
  const maxRounds = Math.min(opts.budget.maxRounds ?? HARD_MAX_ROUNDS, HARD_MAX_ROUNDS);
  const history: LoopHistoryEntry[] = [];
  let totalUsage = emptyUsage();

  const emit = (event: LoopEvent): void => opts.onEvent?.(event);
  const signal = makeTimeoutSignal(opts.budget.timeoutMs);

  const stop = (reason: StopReason, spec: IconSpec | undefined): GenerateResult => {
    emit({ type: 'stop', reason });
    return { spec, history, stopReason: reason, totalUsage };
  };

  // Round 1: planner.
  emit({ type: 'planner-start', round: 1 });
  const plannerSystem = 'You are an IconForge planner. Follow the instructions in the user message exactly.';
  const plannerResult = await askForSpec(
    opts.provider,
    plannerSystem,
    [
      {
        role: 'user',
        content: plannerPrompt({ brief: opts.brief, profile: opts.profile, examples: opts.examples, complexity: opts.complexity }),
      },
    ],
    opts.check,
    signal,
  );
  totalUsage = addUsage(totalUsage, plannerResult.usage);
  history.push({ round: 1, spec: plannerResult.spec, diagnostics: plannerResult.diagnostics, usage: plannerResult.usage });
  emit({ type: 'planner-result', round: 1, ok: plannerResult.spec !== undefined, diagnostics: plannerResult.diagnostics });

  if (plannerResult.spec === undefined) return stop('failed', undefined);
  let currentSpec = plannerResult.spec;

  if (!opts.reviewer) return stop('accepted', currentSpec);

  for (let round = 1; round <= maxRounds; round += 1) {
    if (timedOut(opts.budget.timeoutMs, startedAtMs)) return stop('timeout', currentSpec);
    if (budgetExceeded(opts.budget.maxCostUsd, totalUsage.costUsd ?? 0)) return stop('budget', currentSpec);

    const reviewOutcome = await runReviewRound(opts, currentSpec, round, signal, emit);
    totalUsage = addUsage(totalUsage, reviewOutcome.usage);
    history.push(reviewOutcome.historyEntry);

    if (reviewOutcome.defects.length === 0) return stop('accepted', currentSpec);

    if (timedOut(opts.budget.timeoutMs, startedAtMs)) return stop('timeout', currentSpec);
    if (budgetExceeded(opts.budget.maxCostUsd, totalUsage.costUsd ?? 0)) return stop('budget', currentSpec);

    emit({ type: 'reviser-start', round });
    const revised = await reviseIcon(currentSpec, opts.brief, reviewOutcome.defects, {
      profile: opts.profile,
      provider: opts.provider,
      check: opts.check,
      signal,
    });
    totalUsage = addUsage(totalUsage, revised.usage);
    emit({ type: 'reviser-result', round, ok: revised.spec !== undefined, diagnostics: revised.diagnostics });

    if (revised.spec === undefined) {
      history.push({ round: round + 1, spec: undefined, diagnostics: revised.diagnostics, usage: revised.usage });
      return stop('failed', currentSpec);
    }

    const preservedWarning = unchangedShapesPreservedWarning(currentSpec, revised.spec, reviewOutcome.defects);
    const revisionDiagnostics = preservedWarning ? [...revised.diagnostics, preservedWarning] : revised.diagnostics;
    history.push({ round: round + 1, spec: revised.spec, diagnostics: revisionDiagnostics, usage: revised.usage });

    currentSpec = revised.spec;
  }

  return stop('max-rounds', currentSpec);
}

async function runReviewRound(
  opts: GenerateIconOptions,
  currentSpec: IconSpec,
  round: number,
  signal: AbortSignal | undefined,
  emit: (event: LoopEvent) => void,
): Promise<{ defects: Defect[]; usage: Usage; historyEntry: LoopHistoryEntry }> {
  const rendered = opts.render(currentSpec);
  const geometryDiagnostics = opts.check(currentSpec);
  emit({ type: 'reviewer-start', round });
  const reviewerSystem = 'You are an IconForge reviewer. Follow the instructions in the user message exactly.';
  // opts.reviewer is guaranteed defined by the caller (checked before entering the round loop).
  const reviewerResult = await opts.reviewer!.complete({
    maxTokens: REVIEW_MAX_TOKENS,
    system: reviewerSystem,
    messages: [
      {
        role: 'user',
        content: [
          {
            type: 'text',
            text: reviewerPrompt({
              brief: opts.brief,
              profile: opts.profile,
              examples: opts.examples,
              diagnostics: geometryDiagnostics,
            }),
          },
          { type: 'image', pngBase64: rendered.png24.toString('base64') },
          { type: 'image', pngBase64: rendered.png512.toString('base64') },
        ],
      },
    ],
    signal,
  });

  const extractedReview = extractJson(reviewerResult.text);
  const reviewParsed = extractedReview.ok ? ReviewerOutputSchema.safeParse(extractedReview.value) : undefined;
  const defects: Defect[] = reviewParsed?.success ? reviewParsed.data.defects : [];
  const reviewDiagnostics: Diagnostic[] =
    !extractedReview.ok || !reviewParsed?.success
      ? [
          {
            severity: 'warning',
            code: 'agent.reviewer-output-invalid',
            message: 'reviewer output could not be parsed; treated as no defects',
            path: '',
          },
        ]
      : [];

  emit({ type: 'reviewer-result', round, defects });

  return {
    defects,
    usage: reviewerResult.usage,
    historyEntry: {
      round: round + 1,
      spec: currentSpec,
      diagnostics: reviewDiagnostics,
      defects,
      usage: reviewerResult.usage,
    },
  };
}

export type ReviseIconOptions = {
  profile: StyleProfile;
  provider: LlmProvider;
  check: (spec: IconSpec) => Diagnostic[];
  signal?: AbortSignal;
};

/** Standalone reviser entry point, used by the CLI `revise` command. */
export async function reviseIcon(
  spec: IconSpec,
  brief: string,
  defects: Defect[],
  opts: ReviseIconOptions,
): Promise<{ spec: IconSpec | undefined; diagnostics: Diagnostic[]; usage: Usage }> {
  const system = 'You are an IconForge reviser. Follow the instructions in the user message exactly.';
  return askForSpec(
    opts.provider,
    system,
    [{ role: 'user', content: reviserPrompt({ brief, profile: opts.profile, spec, defects }) }],
    opts.check,
    opts.signal,
  );
}
