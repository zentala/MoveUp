import { parseIconSpec, type Diagnostic, type IconSpec, type StyleProfile } from '@iconforge/schema';

export { shapeBounds, IDENTITY_TRANSFORM, type Bounds, type Transform } from './bounds.ts';
export { checkGeometry } from './rules.ts';
export { flatten, type FlattenedContour } from './flatten.ts';
export { checkJoins } from './joins.ts';

import { checkGeometry } from './rules.ts';

export interface QualitySummary {
  errors: number;
  warnings: number;
  ok: boolean;
}

/** Summarize a diagnostics list: `ok` is true only when there are zero errors. */
export function summarize(diagnostics: readonly Diagnostic[]): QualitySummary {
  let errors = 0;
  let warnings = 0;
  for (const d of diagnostics) {
    if (d.severity === 'error') errors++;
    else warnings++;
  }
  return { errors, warnings, ok: errors === 0 };
}

export type ValidateIconResult =
  | { ok: true; spec: IconSpec; diagnostics: Diagnostic[] }
  | { ok: false; spec?: IconSpec; diagnostics: Diagnostic[] };

/** Parse `input` as an IconSpec and run geometry checks against `profile`. */
export function validateIcon(input: unknown, profile: StyleProfile): ValidateIconResult {
  const parsed = parseIconSpec(input);
  if (!parsed.ok) return { ok: false, diagnostics: parsed.diagnostics };
  const diagnostics = checkGeometry(parsed.value, profile);
  const { ok } = summarize(diagnostics);
  return ok ? { ok: true, spec: parsed.value, diagnostics } : { ok: false, spec: parsed.value, diagnostics };
}
