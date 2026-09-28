import { parseArgs } from 'node:util';
import { generateIcon } from '@iconforge/agent';
import { parseIconSpec } from '@iconforge/schema';
import { makeCheck, makeRender, requireProvider, writeGenerateOutcome, type ProviderFactory } from '../generate-common.ts';
import { CliError, EXIT, readJsonFile, resolveProfile } from '../util.ts';

export const GENERATE_HELP = `Usage: icon generate "<brief>" --out <dir> [options]

Generate an IconSpec from a text brief via an LLM planner/reviewer/reviser loop.

Options:
  --out <dir>            Output directory (required)
  --profile <id|path>    Profile (default: outline-24-v1)
  --examples <dir>       Directory of example IconSpec json files
  --model <id>           OpenRouter model id (default: agent's DEFAULT_MODEL)
  --max-rounds <1..3>    Review/revise rounds (default: 3, hard cap 3)
  --max-cost <usd>       Abort once spend reaches this
  --timeout-ms <ms>      Abort once wall time reaches this
  --help                 Show this help

Reads OPENROUTER_API_KEY from the environment. Exit 2 if missing.
`;

async function loadExamples(dir: string | undefined) {
  if (dir === undefined) return undefined;
  const { readdir } = await import('node:fs/promises');
  const { join, extname } = await import('node:path');
  const files = (await readdir(dir)).filter((f) => extname(f) === '.json');
  const specs = [];
  for (const file of files) {
    const input = await readJsonFile(join(dir, file));
    const parsed = parseIconSpec(input);
    if (parsed.ok) specs.push(parsed.value);
  }
  return specs;
}

export async function runGenerate(argv: string[], deps: { providerFactory?: ProviderFactory } = {}): Promise<number> {
  const { values, positionals } = parseArgs({
    args: argv,
    options: {
      out: { type: 'string' },
      profile: { type: 'string' },
      examples: { type: 'string' },
      model: { type: 'string' },
      'max-rounds': { type: 'string' },
      'max-cost': { type: 'string' },
      'timeout-ms': { type: 'string' },
      help: { type: 'boolean', default: false },
    },
    allowPositionals: true,
  });

  if (values.help) {
    process.stdout.write(GENERATE_HELP);
    return EXIT.OK;
  }
  const brief = positionals[0];
  if (brief === undefined) {
    throw new CliError('generate requires "<brief>"', EXIT.USAGE);
  }
  if (values.out === undefined) {
    throw new CliError('generate requires --out <dir>', EXIT.USAGE);
  }

  const profile = await resolveProfile(values.profile, 'outline-24-v1');
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
  const examples = await loadExamples(values.examples);

  const result = await generateIcon({
    brief,
    profile,
    examples,
    provider,
    reviewer: provider,
    render: makeRender(profile),
    check: makeCheck(profile),
    budget: {
      maxRounds: values['max-rounds'] !== undefined ? Number(values['max-rounds']) : undefined,
      maxCostUsd: values['max-cost'] !== undefined ? Number(values['max-cost']) : undefined,
      timeoutMs: values['timeout-ms'] !== undefined ? Number(values['timeout-ms']) : undefined,
    },
  });

  return writeGenerateOutcome(result, profile, values.out, brief);
}
