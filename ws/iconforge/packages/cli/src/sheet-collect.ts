import { readdir, stat } from 'node:fs/promises';
import { join, basename, extname } from 'node:path';
import type { Diagnostic, IconSpec, StyleProfile } from '@iconforge/schema';
import { validateIcon } from '@iconforge/quality';
import { formatDiagnostics, readJsonFile, resolveProfile } from './util.ts';

export type SheetEntry =
  | { path: string; name: string; ok: true; spec: IconSpec; profile: StyleProfile; diagnostics: Diagnostic[] }
  | { path: string; name: string; ok: false; error: string };

/** Expand `inputs` (files and/or directories) into a sorted list of `.json` file paths. */
export async function expandInputs(inputs: string[]): Promise<string[]> {
  const out: string[] = [];
  for (const input of inputs) {
    const st = await stat(input).catch(() => undefined);
    if (st?.isDirectory()) {
      const entries = await readdir(input);
      for (const entry of entries) {
        if (extname(entry) === '.json') out.push(join(input, entry));
      }
    } else {
      out.push(input);
    }
  }
  out.sort();
  return out;
}

export async function collectSheetEntries(paths: string[], profileArg: string | undefined): Promise<SheetEntry[]> {
  const entries: SheetEntry[] = [];
  for (const path of paths) {
    const name = basename(path, extname(path));
    try {
      const input = await readJsonFile(path);
      const specProfileId =
        input !== null &&
        typeof input === 'object' &&
        'profile' in input &&
        typeof (input as { profile: unknown }).profile === 'string'
          ? (input as { profile: string }).profile
          : undefined;
      const profile = await resolveProfile(profileArg, specProfileId);
      const result = validateIcon(input, profile);
      if (!result.ok) {
        entries.push({ path, name, ok: false, error: formatDiagnostics(result.diagnostics) || 'validation failed' });
        continue;
      }
      entries.push({ path, name, ok: true, spec: result.spec, profile, diagnostics: result.diagnostics });
    } catch (err) {
      entries.push({ path, name, ok: false, error: (err as Error).message });
    }
  }
  return entries;
}
