import { parseArgs } from 'node:util';
import { reviseIcon } from '@iconforge/agent';
import type { Defect } from '@iconforge/agent';
import { parseIconSpec } from '@iconforge/schema';
import { join } from 'node:path';
import { makeCheck, requireProvider, writeGenerateOutcome, type ProviderFactory } from '../generate-common.ts';
import { CliError, EXIT, readJsonFile, resolveProfile, writeOutFile } from '../util.ts';

export const REVISE_HELP = `Usage: icon revise <spec.json> --defects <defects.json|"free text"> --brief "..." --out <dir> [options]

Revise an existing IconSpec against a list of defects.

Options:
  --defects <path|text>  Path to a defects.json array, or free text describing the problem (required)
  --brief "<text>"       Original brief, used as context (required)
  --out <dir>            Output directory (required)
  --profile <id|path>    Profile (default: spec's own "profile" field)
  --model <id>           OpenRouter model id
  --help                 Show this help

Reads OPENROUTER_API_KEY from the environment. Exit 2 if missing.
`;

async function loadDefects(defectsArg: string): Promise<Defect[]> {
  if (defectsArg.endsWith('.json')) {
    const input = await readJsonFile(defectsArg);
    if (!Array.isArray(input)) {
      throw new CliError(`"${defectsArg}" must contain a JSON array of defects`, EXIT.USAGE);
    }
    return input as Defect[];
  }
  return [{ shapeId: 'unknown', observation: defectsArg, change: 'address the observation above' }];
}

export async function runRevise(argv: string[], deps: { providerFactory?: ProviderFactory } = {}): Promise<number> {
  const { values, positionals } = parseArgs({
    args: argv,
    options: {
      defects: { type: 'string' },
      brief: { type: 'string' },
      out: { type: 'string' },
      profile: { type: 'string' },
      model: { type: 'string' },
      help: { type: 'boolean', default: false },
    },
    allowPositionals: true,
  });

  if (values.help) {
    process.stdout.write(REVISE_HELP);
    return EXIT.OK;
  }
  const specPath = positionals[0];
  if (specPath === undefined) throw new CliError('revise requires <spec.json>', EXIT.USAGE);
  if (values.defects === undefined) throw new CliError('revise requires --defects <path|text>', EXIT.USAGE);
  if (values.brief === undefined) throw new CliError('revise requires --brief "..."', EXIT.USAGE);
  if (values.out === undefined) throw new CliError('revise requires --out <dir>', EXIT.USAGE);

  const input = await readJsonFile(specPath);
  const parsedSpec = parseIconSpec(input);
  if (!parsedSpec.ok) {
    throw new CliError(`"${specPath}" is not a valid IconSpec`, EXIT.VALIDATION);
  }

  const profile = await resolveProfile(values.profile, parsedSpec.value.profile);
  let provider;
  try {
    provider = requireProvider(values.model, deps.providerFactory);
  } catch (err) {
    if (err instanceof CliError) {
      process.stderr.write(`${err.message}\n`);
      return err.exitCode;
    }
    throw err;
  }
  const defects = await loadDefects(values.defects);

  const revised = await reviseIcon(parsedSpec.value, values.brief, defects, {
    profile,
    provider,
    check: makeCheck(profile),
  });

  const outcome = {
    spec: revised.spec,
    history: [{ round: 1, spec: revised.spec, diagnostics: revised.diagnostics, defects, usage: revised.usage }],
    stopReason: revised.spec !== undefined ? ('accepted' as const) : ('failed' as const),
    totalUsage: revised.usage,
  };

  await writeOutFile(join(values.out, 'defects.json'), `${JSON.stringify(defects, null, 2)}\n`);
  return writeGenerateOutcome(outcome, profile, values.out, values.brief);
}
