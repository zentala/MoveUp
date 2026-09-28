import { parseArgs } from 'node:util';
import { join } from 'node:path';
import { collectSheetEntries, expandInputs, type SheetEntry } from '../sheet-collect.ts';
import { buildSheetHtml } from '../sheet-html.ts';
import { renderSheetPng } from '../sheet-png.ts';
import { CliError, EXIT, writeOutFile } from '../util.ts';

export const SHEET_HELP = `Usage: icon sheet <dir|files...> --out <dir> [--profile id|path]

Build a comparison sheet (sheet.html + sheet.png) for a set of icon specs.
Invalid icons are listed as rejected with their first diagnostic, not skipped.

Options:
  --out <dir>           Output directory (required)
  --profile <id|path>   Profile override (default: each spec's own "profile" field)
  --help                Show this help
`;

export async function runSheet(argv: string[]): Promise<number> {
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
    process.stdout.write(SHEET_HELP);
    return EXIT.OK;
  }
  if (positionals.length === 0) {
    throw new CliError('sheet requires <dir|files...>', EXIT.USAGE);
  }
  if (values.out === undefined) {
    throw new CliError('sheet requires --out <dir>', EXIT.USAGE);
  }

  const paths = await expandInputs(positionals);
  const entries: SheetEntry[] = await collectSheetEntries(paths, values.profile);
  const valid = entries.filter((e): e is Extract<SheetEntry, { ok: true }> => e.ok);
  const rejected = entries.filter((e): e is Extract<SheetEntry, { ok: false }> => !e.ok);

  const html = buildSheetHtml(entries);
  const png = renderSheetPng(valid);

  await writeOutFile(join(values.out, 'sheet.html'), html);
  await writeOutFile(join(values.out, 'sheet.png'), png);

  process.stdout.write(`wrote ${values.out} (${valid.length} icons, ${rejected.length} rejected)\n`);
  for (const r of rejected) {
    process.stdout.write(`rejected ${r.path}: ${r.error.split('\n')[0]}\n`);
  }

  return EXIT.OK;
}
