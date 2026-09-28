import { validateIcon } from '@iconforge/quality';
import type { Diagnostic } from '@iconforge/schema';
import { parseArgs } from 'node:util';
import { CliError, EXIT, formatDiagnostic, readJsonFile, resolveProfile } from '../util.ts';

export const VALIDATE_HELP = `Usage: icon validate <spec.json...> [--profile id|path] [--json]

Validate one or more IconSpec files against a style profile.

Options:
  --profile <id|path>  Profile to validate against (default: spec's own "profile" field)
  --json                Print diagnostics as JSON instead of text
  --help                Show this help
`;

export type ValidateFileResult = { path: string; ok: boolean; diagnostics: Diagnostic[] };

export async function runValidateFile(path: string, profileArg: string | undefined): Promise<ValidateFileResult> {
  const input = await readJsonFile(path);
  const specProfileId =
    input !== null && typeof input === 'object' && 'profile' in input && typeof (input as { profile: unknown }).profile === 'string'
      ? (input as { profile: string }).profile
      : undefined;
  const profile = await resolveProfile(profileArg, specProfileId);
  const result = validateIcon(input, profile);
  return { path, ok: result.ok, diagnostics: result.diagnostics };
}

export async function runValidate(argv: string[]): Promise<number> {
  const { values, positionals } = parseArgs({
    args: argv,
    options: {
      profile: { type: 'string' },
      json: { type: 'boolean', default: false },
      help: { type: 'boolean', default: false },
    },
    allowPositionals: true,
  });

  if (values.help) {
    process.stdout.write(VALIDATE_HELP);
    return EXIT.OK;
  }
  if (positionals.length === 0) {
    throw new CliError('validate requires at least one <spec.json>', EXIT.USAGE);
  }

  const results: ValidateFileResult[] = [];
  for (const path of positionals) {
    results.push(await runValidateFile(path, values.profile));
  }

  if (values.json) {
    process.stdout.write(`${JSON.stringify(results, null, 2)}\n`);
  } else {
    for (const result of results) {
      if (result.diagnostics.length === 0) {
        process.stdout.write(`${result.path} ok\n`);
      } else {
        for (const d of result.diagnostics) {
          process.stdout.write(`${formatDiagnostic(result.path, d)}\n`);
        }
      }
    }
  }

  const allOk = results.every((r) => r.ok);
  return allOk ? EXIT.OK : EXIT.VALIDATION;
}
