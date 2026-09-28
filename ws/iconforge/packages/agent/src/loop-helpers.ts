import type { IconSpec, Diagnostic } from '@iconforge/schema';
import { parseIconSpec } from '@iconforge/schema';
import type { ChatMessage, LlmProvider, Usage } from './provider.ts';
import { extractJson } from './json-extract.ts';
import { retryWithDiagnosticsPrompt, type Defect } from './prompts.ts';

export const ICON_SPEC_JSON_SCHEMA_NAME = 'IconSpecV1';

/**
 * Output token caps. Without them OpenRouter reserves the model's default
 * (65 536 for Claude Sonnet 5) against the key's credit and answers 402 on a
 * low balance. A 64-shape IconSpec is ~6k tokens; a review of <=3 defects ~600.
 */
export const SPEC_MAX_TOKENS = 8000;
export const REVIEW_MAX_TOKENS = 1500;

export function emptyUsage(): Usage {
  return { inputTokens: 0, outputTokens: 0, costUsd: 0 };
}

export function addUsage(a: Usage, b: Usage): Usage {
  return {
    inputTokens: a.inputTokens + b.inputTokens,
    outputTokens: a.outputTokens + b.outputTokens,
    costUsd: (a.costUsd ?? 0) + (b.costUsd ?? 0),
  };
}

function onlyErrors(diagnostics: Diagnostic[]): Diagnostic[] {
  return diagnostics.filter((d) => d.severity === 'error');
}

/**
 * Ask a provider for an IconSpec, validate it with `parseIconSpec` + `check`,
 * and — on failure — perform exactly ONE explicit retry with the diagnostics
 * attached, per `docs/spec/ARCHITECTURE.md`. Shared by the planner and
 * reviser steps.
 */
export async function askForSpec(
  provider: LlmProvider,
  system: string,
  initialMessages: ChatMessage[],
  check: (spec: IconSpec) => Diagnostic[],
  signal: AbortSignal | undefined,
): Promise<{ spec: IconSpec | undefined; diagnostics: Diagnostic[]; usage: Usage }> {
  let usage = emptyUsage();
  const messages = [...initialMessages];

  for (let attempt = 0; attempt < 2; attempt += 1) {
    // No provider-enforced JSON schema: strict modes reject the recursive IconSpec
    // schema, and the acyclic unrolled one exceeds their grammar size limit
    // (OpenRouter -> Anthropic/Google/Azure, 2026-09-28). The prompt carries the
    // type description; parseIconSpec + one retry enforce the contract.
    const result = await provider.complete({ system, messages, maxTokens: SPEC_MAX_TOKENS, signal });
    usage = addUsage(usage, result.usage);

    const extracted = extractJson(result.text);
    if (!extracted.ok) {
      if (attempt === 0) {
        pushRetry(messages, result.text, extracted.diagnostics);
        continue;
      }
      return { spec: undefined, diagnostics: extracted.diagnostics, usage };
    }

    const parsed = parseIconSpec(extracted.value);
    if (!parsed.ok) {
      if (attempt === 0) {
        pushRetry(messages, result.text, parsed.diagnostics);
        continue;
      }
      return { spec: undefined, diagnostics: parsed.diagnostics, usage };
    }

    const geometryDiagnostics = check(parsed.value);
    const geometryErrors = onlyErrors(geometryDiagnostics);
    if (geometryErrors.length > 0) {
      if (attempt === 0) {
        pushRetry(messages, result.text, geometryErrors);
        continue;
      }
      return { spec: undefined, diagnostics: geometryDiagnostics, usage };
    }

    return { spec: parsed.value, diagnostics: geometryDiagnostics, usage };
  }

  // Unreachable: the loop above always returns within its two iterations.
  return { spec: undefined, diagnostics: [], usage };
}

function pushRetry(messages: ChatMessage[], assistantText: string, diagnostics: Diagnostic[]): void {
  messages.push({ role: 'assistant', content: assistantText });
  messages.push({ role: 'user', content: retryWithDiagnosticsPrompt({ diagnostics }) });
}

export function budgetExceeded(maxCostUsd: number | undefined, spentUsd: number): boolean {
  return maxCostUsd !== undefined && spentUsd >= maxCostUsd;
}

export function timedOut(timeoutMs: number | undefined, startedAtMs: number): boolean {
  return timeoutMs !== undefined && Date.now() - startedAtMs >= timeoutMs;
}

/** Ids of shapes at any depth, used to check that unchanged shapes keep their id after a revision. */
function collectShapeIds(spec: IconSpec): Set<string> {
  const ids = new Set<string>();
  const walk = (shapes: IconSpec['shapes']): void => {
    for (const shape of shapes) {
      ids.add(shape.id);
      if (shape.type === 'group') walk(shape.children);
    }
  };
  walk(spec.shapes);
  return ids;
}

/** Warns when the reviser dropped the id of a shape that was not listed as a defect. */
export function unchangedShapesPreservedWarning(
  before: IconSpec,
  after: IconSpec,
  defects: Defect[],
): Diagnostic | undefined {
  const touchedIds = new Set(defects.map((d) => d.shapeId));
  const beforeIds = collectShapeIds(before);
  const afterIds = collectShapeIds(after);
  const dropped = [...beforeIds].filter((id) => !touchedIds.has(id) && !afterIds.has(id));
  if (dropped.length === 0) return undefined;
  return {
    severity: 'warning',
    code: 'agent.shape-id-not-preserved',
    message: `reviser dropped ids of shapes not listed as defects: ${dropped.join(', ')}`,
    path: 'shapes',
  };
}

export function makeTimeoutSignal(timeoutMs: number | undefined): AbortSignal | undefined {
  if (timeoutMs === undefined) return undefined;
  return AbortSignal.timeout(timeoutMs);
}
