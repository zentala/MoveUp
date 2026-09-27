import { join } from 'node:path';
import type { IconSpec, StyleProfile } from '@iconforge/schema';
import { checkGeometry } from '@iconforge/quality';
import { renderPng, renderSvg } from '@iconforge/renderer';
import { createOpenRouterProvider, DEFAULT_MODEL, type LlmProvider, type GenerateResult } from '@iconforge/agent';
import { writeIconArtifacts } from './artifacts.ts';
import { CliError, EXIT, writeOutFile } from './util.ts';

export type ProviderFactory = (opts: { apiKey: string; model?: string }) => LlmProvider;

const defaultProviderFactory: ProviderFactory = (opts) => createOpenRouterProvider(opts);

/** Reads OPENROUTER_API_KEY and builds a provider. Never prints the key. Exit code 2 when missing. */
export function requireProvider(model: string | undefined, factory: ProviderFactory = defaultProviderFactory): LlmProvider {
  const apiKey = process.env.OPENROUTER_API_KEY;
  if (apiKey === undefined || apiKey.length === 0) {
    throw new CliError('OPENROUTER_API_KEY is not set', EXIT.USAGE);
  }
  return factory({ apiKey, model: model ?? DEFAULT_MODEL });
}

export function makeRender(profile: StyleProfile) {
  return (spec: IconSpec) => {
    const svg = renderSvg(spec, profile);
    const png24 = renderPng(svg, { size: 24, background: 'light' });
    const png512 = renderPng(svg, { size: 512, background: 'light' });
    return { svg, png24, png512 };
  };
}

export function makeCheck(profile: StyleProfile) {
  return (spec: IconSpec) => checkGeometry(spec, profile);
}

/** Redacts anything key-shaped out of a history record before it is written to disk. */
function historyOf(result: GenerateResult): unknown {
  return {
    stopReason: result.stopReason,
    totalUsage: result.totalUsage,
    rounds: result.history.map((h) => ({
      round: h.round,
      spec: h.spec,
      diagnostics: h.diagnostics,
      defects: h.defects,
      usage: h.usage,
    })),
  };
}

/** Writes render artifacts (if a spec was produced) plus history.json and brief.txt. */
export async function writeGenerateOutcome(
  result: GenerateResult,
  profile: StyleProfile,
  outDir: string,
  brief: string,
): Promise<number> {
  await writeOutFile(join(outDir, 'brief.txt'), `${brief}\n`);
  await writeOutFile(join(outDir, 'history.json'), `${JSON.stringify(historyOf(result), null, 2)}\n`);

  if (result.spec === undefined) {
    process.stderr.write(`generate failed: ${result.stopReason}\n`);
    return result.stopReason === 'budget' || result.stopReason === 'timeout' ? EXIT.PROVIDER : EXIT.VALIDATION;
  }

  const finalDiagnostics = checkGeometry(result.spec, profile);
  await writeIconArtifacts(result.spec, profile, finalDiagnostics, outDir);
  process.stdout.write(`wrote ${outDir} (${result.stopReason})\n`);
  return result.stopReason === 'accepted' || result.stopReason === 'max-rounds' ? EXIT.OK : EXIT.PROVIDER;
}
