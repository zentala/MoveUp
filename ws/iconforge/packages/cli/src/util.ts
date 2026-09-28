import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { dirname, resolve, isAbsolute } from 'node:path';
import type { Diagnostic } from '@iconforge/schema';
import { BUILTIN_PROFILES, parseProfile, type StyleProfile } from '@iconforge/schema';

/** CLI-level exit codes: 0 ok, 1 validation errors, 2 usage/config error, 3 provider/budget failure. */
export const EXIT = { OK: 0, VALIDATION: 1, USAGE: 2, PROVIDER: 3 } as const;

export class CliError extends Error {
  readonly exitCode: number;
  constructor(message: string, exitCode: number) {
    super(message);
    this.name = 'CliError';
    this.exitCode = exitCode;
  }
}

export async function readJsonFile(path: string): Promise<unknown> {
  let text: string;
  try {
    text = await readFile(path, 'utf8');
  } catch (err) {
    throw new CliError(`cannot read "${path}": ${(err as Error).message}`, EXIT.USAGE);
  }
  try {
    return JSON.parse(text);
  } catch (err) {
    throw new CliError(`"${path}" is not valid JSON: ${(err as Error).message}`, EXIT.USAGE);
  }
}

/**
 * Resolve `--profile` to a StyleProfile.
 * - undefined -> look up `specProfileId` in BUILTIN_PROFILES.
 * - looks like a path (contains `/`, `\`, or ends with `.json`) -> read + parseProfile.
 * - otherwise -> builtin id lookup.
 */
export async function resolveProfile(
  profileArg: string | undefined,
  specProfileId: string | undefined,
): Promise<StyleProfile> {
  const candidate = profileArg ?? specProfileId;
  if (candidate === undefined) {
    throw new CliError('no profile specified and spec has no "profile" field', EXIT.USAGE);
  }
  const looksLikePath = /[/\\]/.test(candidate) || candidate.endsWith('.json');
  if (looksLikePath) {
    const input = await readJsonFile(candidate);
    const parsed = parseProfile(input);
    if (!parsed.ok) {
      throw new CliError(`invalid profile file "${candidate}": ${formatDiagnostics(parsed.diagnostics)}`, EXIT.USAGE);
    }
    return parsed.value;
  }
  const builtin = BUILTIN_PROFILES[candidate];
  if (builtin === undefined) {
    throw new CliError(
      `unknown profile "${candidate}" (builtin ids: ${Object.keys(BUILTIN_PROFILES).join(', ')})`,
      EXIT.USAGE,
    );
  }
  return builtin;
}

export function formatDiagnostic(path: string, d: Diagnostic): string {
  const shape = d.shapeId !== undefined ? ` [${d.shapeId}]` : '';
  return `${path}${shape} ${d.severity} ${d.code}: ${d.message}`;
}

export function formatDiagnostics(diagnostics: readonly Diagnostic[], path = ''): string {
  return diagnostics.map((d) => formatDiagnostic(path, d)).join('\n');
}

export async function ensureDir(dir: string): Promise<void> {
  await mkdir(dir, { recursive: true });
}

export async function writeOutFile(path: string, data: string | Buffer): Promise<void> {
  await ensureDir(dirname(path));
  await writeFile(path, data);
}

export function sha256Hex(data: string | Buffer): string {
  return createHash('sha256').update(data).digest('hex');
}

export function resolvePath(path: string): string {
  return isAbsolute(path) ? path : resolve(process.cwd(), path);
}
