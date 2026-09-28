import type { Diagnostic } from '@iconforge/schema';

/**
 * Extract a JSON value from raw model text. Tolerates a single ```json ... ```
 * (or bare ``` ... ```) fence around the payload; never evaluates the text,
 * only `JSON.parse`s it.
 */
export function extractJson(text: string): { ok: true; value: unknown } | { ok: false; diagnostics: Diagnostic[] } {
  const candidate = stripFence(text.trim());
  try {
    return { ok: true, value: JSON.parse(candidate) };
  } catch (err) {
    return {
      ok: false,
      diagnostics: [
        {
          severity: 'error',
          code: 'agent.invalid-json',
          message: `model output is not valid JSON: ${err instanceof Error ? err.message : String(err)}`,
          path: '',
        },
      ],
    };
  }
}

const FENCE_RE = /^```(?:json)?\s*\n?([\s\S]*?)\n?```$/;

function stripFence(text: string): string {
  const match = FENCE_RE.exec(text);
  return match?.[1] !== undefined ? match[1].trim() : text;
}
