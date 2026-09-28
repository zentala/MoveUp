import { parseArgs } from 'node:util';
import { join, basename, extname, dirname } from 'node:path';
import { improveIcon, type Complexity, type GenerateResult } from '@iconforge/agent';
import { parseIconSpec } from '@iconforge/schema';
import { renderSvg, renderPng } from '@iconforge/renderer';
import { expandInputs } from '../sheet-collect.ts';
import { buildImproveSheetHtml, renderImproveSheetPng, type ImproveSheetEntry } from '../improve-sheet.ts';
import { makeCheck, makeRender, requireProvider, type ProviderFactory } from '../generate-common.ts';
import { CliError, EXIT, readJsonFile, resolveProfile, writeOutFile } from '../util.ts';

export const IMPROVE_HELP = `Usage: icon improve <spec.json|dir...> --out <dir> [options]

Run the review -> revise loop against EXISTING IconSpec files (no planner
round). Writes before/after renders, per-round specs and a comparison sheet.

Options:
  --out <dir>              Output directory (required)
  --brief "<text>"         Brief for every icon (default: derived from each icon's
                            name, or looked up in a "briefs.json" file next to the
                            specs: { "<name>": "<brief>" })
  --rounds <1..3>           Review/revise rounds (default: 3, hard cap 3)
  --model <id>              OpenRouter model id for the reviser (default: agent's DEFAULT_MODEL)
  --reviewer-model <id>     OpenRouter model id for the reviewer (default: same as --model)
  --max-cost <usd>          Abort once spend reaches this
  --complexity <auto|simple|detailed>  Accepted for parity with "generate"; the
                            review/revise loop has no planner round so it has no
                            effect on shape count here.
  --help                    Show this help

Reads OPENROUTER_API_KEY from the environment. Exit 2 if missing.
`;

const COMPLEXITY_VALUES: readonly Complexity[] = ['auto', 'simple', 'detailed'];

async function loadBriefsMap(specPath: string): Promise<Record<string, string>> {
  const briefsPath = join(dirname(specPath), 'briefs.json');
  try {
    const input = await readJsonFile(briefsPath);
    if (input !== null && typeof input === 'object' && !Array.isArray(input)) {
      return input as Record<string, string>;
    }
    return {};
  } catch {
    return {};
  }
}

function historyOf(result: GenerateResult): unknown {
  return {
    stopReason: result.stopReason,
    totalUsage: result.totalUsage,
    rounds: result.history.map((h) => ({
      round: h.round,
      diagnostics: h.diagnostics,
      defects: h.defects,
      usage: h.usage,
    })),
  };
}

export async function runImprove(argv: string[], deps: { providerFactory?: ProviderFactory } = {}): Promise<number> {
  const { values, positionals } = parseArgs({
    args: argv,
    options: {
      out: { type: 'string' },
      brief: { type: 'string' },
      rounds: { type: 'string' },
      model: { type: 'string' },
      'reviewer-model': { type: 'string' },
      'max-cost': { type: 'string' },
      complexity: { type: 'string' },
      help: { type: 'boolean', default: false },
    },
    allowPositionals: true,
  });

  if (values.help) {
    process.stdout.write(IMPROVE_HELP);
    return EXIT.OK;
  }
  if (positionals.length === 0) {
    throw new CliError('improve requires <spec.json|dir...>', EXIT.USAGE);
  }
  if (values.out === undefined) {
    throw new CliError('improve requires --out <dir>', EXIT.USAGE);
  }
  if (values.complexity !== undefined && !COMPLEXITY_VALUES.includes(values.complexity as Complexity)) {
    throw new CliError(`--complexity must be one of ${COMPLEXITY_VALUES.join(', ')}`, EXIT.USAGE);
  }

  let reviserProvider;
  let reviewerProvider;
  try {
    reviserProvider = requireProvider(values.model, deps.providerFactory);
    reviewerProvider = requireProvider(values['reviewer-model'] ?? values.model, deps.providerFactory);
  } catch (err) {
    if (err instanceof CliError) {
      process.stderr.write(`${err.message}\n`);
      return err.exitCode;
    }
    throw err;
  }

  const maxRounds = values.rounds !== undefined ? Number(values.rounds) : undefined;
  const maxCostUsd = values['max-cost'] !== undefined ? Number(values['max-cost']) : undefined;

  const paths = await expandInputs(positionals);
  const briefsCache = new Map<string, Record<string, string>>();
  const sheetEntries: ImproveSheetEntry[] = [];
  let failures = 0;

  for (const specPath of paths) {
    const name = basename(specPath, extname(specPath));
    const outDir = join(values.out, name);
    let input: unknown;
    try {
      input = await readJsonFile(specPath);
    } catch (err) {
      process.stderr.write(`${name}: ${(err as Error).message}\n`);
      failures += 1;
      continue;
    }
    const parsed = parseIconSpec(input);
    if (!parsed.ok) {
      process.stderr.write(`${name}: not a valid IconSpec\n`);
      failures += 1;
      continue;
    }

    const profile = await resolveProfile(undefined, parsed.value.profile);
    const briefDir = dirname(specPath);
    if (!briefsCache.has(briefDir)) briefsCache.set(briefDir, await loadBriefsMap(specPath));
    const brief = values.brief ?? briefsCache.get(briefDir)?.[name] ?? name.replace(/-/g, ' ');

    const beforeSpec = parsed.value;
    const beforeSvg = renderSvg(beforeSpec, profile);
    await writeOutFile(join(outDir, 'before.svg'), beforeSvg);
    await writeOutFile(join(outDir, 'before-512-light.png'), renderPng(beforeSvg, { size: 512, background: 'light' }));
    await writeOutFile(join(outDir, 'before-24-light.png'), renderPng(beforeSvg, { size: 24, background: 'light' }));

    // One icon's provider failure must not abort the batch or suppress the sheet.
    let result: Awaited<ReturnType<typeof improveIcon>>;
    try {
      result = await improveIcon({
        spec: beforeSpec,
        brief,
        profile,
        reviewer: reviewerProvider,
        reviser: reviserProvider,
        render: makeRender(profile),
        check: makeCheck(profile),
        budget: { maxRounds, maxCostUsd },
      });
    } catch (err) {
      const message = (err as Error).message;
      process.stderr.write(`${name}: failed — ${message}\n`);
      await writeOutFile(join(outDir, 'history.json'), `${JSON.stringify({ error: message }, null, 2)}\n`);
      failures += 1;
      continue;
    }

    for (const entry of result.history) {
      if (entry.spec !== undefined) {
        await writeOutFile(join(outDir, 'rounds', `${entry.round}.json`), `${JSON.stringify(entry.spec, null, 2)}\n`);
      }
    }

    const afterSpec = result.spec ?? beforeSpec;
    const afterSvg = renderSvg(afterSpec, profile);
    await writeOutFile(join(outDir, 'after.svg'), afterSvg);
    await writeOutFile(join(outDir, 'after-512-light.png'), renderPng(afterSvg, { size: 512, background: 'light' }));
    await writeOutFile(join(outDir, 'after-24-light.png'), renderPng(afterSvg, { size: 24, background: 'light' }));
    await writeOutFile(join(outDir, 'after.json'), `${JSON.stringify(afterSpec, null, 2)}\n`);
    await writeOutFile(join(outDir, 'history.json'), `${JSON.stringify(historyOf(result), null, 2)}\n`);

    sheetEntries.push({
      name,
      beforeSvg,
      afterSvg,
      viewBox: profile.viewBox,
      rounds: result.history.map((h) => ({ round: h.round, defects: h.defects ?? [], diagnostics: h.diagnostics })),
      stopReason: result.stopReason,
    });

    const cost = (result.totalUsage.costUsd ?? 0).toFixed(4);
    process.stdout.write(`${name}: ${result.history.length} round(s), ${result.stopReason}, $${cost}\n`);
    if (result.stopReason !== 'accepted' && result.stopReason !== 'max-rounds') failures += 1;
  }

  await writeOutFile(join(values.out, 'improve-sheet.html'), buildImproveSheetHtml(sheetEntries));
  await writeOutFile(join(values.out, 'improve-sheet.png'), renderImproveSheetPng(sheetEntries));

  process.stdout.write(`wrote ${values.out} (${sheetEntries.length} icons, ${failures} failed)\n`);
  return failures === 0 ? EXIT.OK : EXIT.PROVIDER;
}
