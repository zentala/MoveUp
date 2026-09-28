import type { z } from 'zod';

export type Severity = 'error' | 'warning';

/** One finding. `path` is slash-separated, e.g. `shapes/2/children/0/radius`. */
export type Diagnostic = {
  severity: Severity;
  code: string;
  message: string;
  path: string;
  shapeId?: string;
};

export type ParseResult<T> =
  | { ok: true; value: T; diagnostics: Diagnostic[] }
  | { ok: false; diagnostics: Diagnostic[] };

export function zodToDiagnostics(error: z.ZodError, input: unknown): Diagnostic[] {
  return error.issues.map((issue) => {
    const diagnostic: Diagnostic = {
      severity: 'error',
      code: `schema.${issue.code}`,
      message: issue.message,
      path: issue.path.map(String).join('/'),
    };
    const shapeId = findShapeId(input, issue.path);
    if (shapeId !== undefined) diagnostic.shapeId = shapeId;
    return diagnostic;
  });
}

/** Walk the input along the issue path and return the id of the innermost shape containing it. */
function findShapeId(input: unknown, path: readonly PropertyKey[]): string | undefined {
  let node: unknown = input;
  let found: string | undefined;
  for (const key of path) {
    if (node === null || typeof node !== 'object') break;
    node = (node as Record<PropertyKey, unknown>)[key];
    if (node && typeof node === 'object' && 'type' in node && 'id' in node) {
      const nodeId = (node as { id: unknown }).id;
      if (typeof nodeId === 'string') found = nodeId;
    }
  }
  return found;
}
