import { IconSpecSchema, type IconSpec, type Shape } from './iconSpec.ts';
import { StyleProfileSchema, type StyleProfile } from './profile.ts';
import { zodToDiagnostics, type Diagnostic, type ParseResult } from './diagnostics.ts';

/** Parse and structurally check an IconSpec. Never repairs coordinates silently. */
export function parseIconSpec(input: unknown): ParseResult<IconSpec> {
  const result = IconSpecSchema.safeParse(input);
  if (!result.success) return { ok: false, diagnostics: zodToDiagnostics(result.error, input) };
  const diagnostics = checkUniqueIds(result.data.shapes);
  if (diagnostics.length > 0) return { ok: false, diagnostics };
  return { ok: true, value: result.data, diagnostics: [] };
}

export function parseProfile(input: unknown): ParseResult<StyleProfile> {
  const result = StyleProfileSchema.safeParse(input);
  if (!result.success) return { ok: false, diagnostics: zodToDiagnostics(result.error, input) };
  return { ok: true, value: result.data, diagnostics: [] };
}

function checkUniqueIds(shapes: Shape[]): Diagnostic[] {
  const seen = new Map<string, string>();
  const out: Diagnostic[] = [];
  const walk = (list: Shape[], prefix: string): void => {
    list.forEach((shape, i) => {
      const path = `${prefix}/${i}`;
      const first = seen.get(shape.id);
      if (first !== undefined) {
        out.push({
          severity: 'error',
          code: 'spec.duplicate-id',
          message: `duplicate id "${shape.id}" (first at ${first})`,
          path,
          shapeId: shape.id,
        });
      } else {
        seen.set(shape.id, path);
      }
      if (shape.type === 'group') walk(shape.children, `${path}/children`);
    });
  };
  walk(shapes, 'shapes');
  return out;
}
