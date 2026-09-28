import type { CompleteRequest, CompleteResult, LlmProvider } from './provider.ts';

export type MockResponder = string | ((req: CompleteRequest, callIndex: number) => string);

/**
 * Deterministic in-memory provider for tests. `responses[i]` answers the i-th
 * call; once exhausted, the last response repeats. Every call (request +
 * result) is recorded on `.calls` for assertions.
 */
export interface MockProvider extends LlmProvider {
  calls: Array<{ req: CompleteRequest; result: CompleteResult }>;
}

export function createMockProvider(responses: MockResponder[], opts?: { model?: string }): MockProvider {
  const calls: Array<{ req: CompleteRequest; result: CompleteResult }> = [];
  return {
    id: 'mock',
    model: opts?.model ?? 'mock-model',
    calls,
    async complete(req: CompleteRequest): Promise<CompleteResult> {
      const index = calls.length;
      const responder = responses[Math.min(index, responses.length - 1)];
      if (responder === undefined) {
        throw new Error('createMockProvider: no responses configured');
      }
      const text = typeof responder === 'function' ? responder(req, index) : responder;
      const result: CompleteResult = {
        text,
        usage: { inputTokens: 0, outputTokens: 0, costUsd: 0 },
      };
      calls.push({ req, result });
      return result;
    },
  };
}
