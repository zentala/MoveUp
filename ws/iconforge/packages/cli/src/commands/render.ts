import { parseArgs } from 'node:util';
import { validateIcon } from '@iconforge/quality';
import { writeIconArtifacts } from '../artifacts.ts';
import { CliError, EXIT, formatDiagnostic, readJsonFile, resolveProfile } from '../util.ts';

export const RENDER_HELP = `Usage: icon render <spec.json> --out <dir> [--profile id|path]

Render one IconSpec to SVG + PNG previews. On validation errors, prints
diagnostics, writes nothing, and exits 1.

Options:
  --out <dir>           Output directory (required)
  --profile <id|path>   Profile to render against (default: spec's own "profile" field)
  --help                Show this help
`;

export async function runRender(argv: string[]): Promise<number> {
  const { values, positionals } = parseArgs({
    args: argv,
    options: {
      out: { type: 'string' },
      profile: { type: 'string' },
      help: { type: 'boolean', default: false },
    },
    allowPositionals: true,
  });

  if (values.help) {
    process.stdout.write(RENDER_HELP);
    return EXIT.OK;
  }
  const specPath = positionals[0];
  if (specPath === undefined) {
    throw new CliError('render requires exactly one <spec.json>', EXIT.USAGE);
  }
  if (values.out === undefined) {
    throw new CliError('render requires --out <dir>', EXIT.USAGE);
  }

  const input = await readJsonFile(specPath);
  const specProfileId =
    input !== null && typeof input === 'object' && 'profile' in input && typeof (input as { profile: unknown }).profile === 'string'
      ? (input as { profile: string }).profile
      : undefined;
  const profile = await resolveProfile(values.profile, specProfileId);
  const result = validateIcon(input, profile);

  if (!result.ok) {
    for (const d of result.diagnostics) {
      process.stderr.write(`${formatDiagnostic(specPath, d)}\n`);
    }
    return EXIT.VALIDATION;
  }

  await writeIconArtifacts(result.spec, profile, result.diagnostics, values.out);
  process.stdout.write(`wrote ${values.out}\n`);
  return EXIT.OK;
}
