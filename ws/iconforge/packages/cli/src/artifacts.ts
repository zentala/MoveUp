import { join } from 'node:path';
import type { Diagnostic, IconSpec, StyleProfile } from '@iconforge/schema';
import { SCHEMA_VERSION } from '@iconforge/schema';
import { summarize } from '@iconforge/quality';
import { renderPng, renderSvg } from '@iconforge/renderer';
import { sha256Hex, writeOutFile } from './util.ts';

export type RenderReport = {
  profile: string;
  schemaVersion: number;
  diagnostics: Diagnostic[];
  summary: ReturnType<typeof summarize>;
  svgSha256: string;
};

/** Writes icon.svg, spec.json, four PNG previews and report.json into `outDir`. Returns the report. */
export async function writeIconArtifacts(
  spec: IconSpec,
  profile: StyleProfile,
  diagnostics: Diagnostic[],
  outDir: string,
): Promise<RenderReport> {
  const svg = renderSvg(spec, profile);
  const png24Light = renderPng(svg, { size: 24, background: 'light' });
  const png24Dark = renderPng(svg, { size: 24, background: 'dark' });
  const png512Light = renderPng(svg, { size: 512, background: 'light' });
  const png512Dark = renderPng(svg, { size: 512, background: 'dark' });

  const report: RenderReport = {
    profile: profile.id,
    schemaVersion: SCHEMA_VERSION,
    diagnostics,
    summary: summarize(diagnostics),
    svgSha256: sha256Hex(svg),
  };

  await writeOutFile(join(outDir, 'icon.svg'), svg);
  await writeOutFile(join(outDir, 'spec.json'), `${JSON.stringify(spec, null, 2)}\n`);
  await writeOutFile(join(outDir, 'preview-24-light.png'), png24Light);
  await writeOutFile(join(outDir, 'preview-24-dark.png'), png24Dark);
  await writeOutFile(join(outDir, 'preview-512-light.png'), png512Light);
  await writeOutFile(join(outDir, 'preview-512-dark.png'), png512Dark);
  await writeOutFile(join(outDir, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);

  return report;
}
