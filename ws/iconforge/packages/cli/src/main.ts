#!/usr/bin/env node
import { pathToFileURL } from 'node:url';
import { runValidate } from './commands/validate.ts';
import { runRender } from './commands/render.ts';
import { runSheet } from './commands/sheet.ts';
import { runGenerate } from './commands/generate.ts';
import { runRevise } from './commands/revise.ts';
import { CliError, EXIT } from './util.ts';

const TOP_HELP = `Usage: icon <command> [options]

Commands:
  validate   Validate IconSpec files against a profile
  render     Render one IconSpec to SVG + PNG previews
  sheet      Build a comparison sheet for a set of icons
  generate   Generate an IconSpec from a text brief via an LLM loop
  revise     Revise an IconSpec against reported defects

Run "icon <command> --help" for command-specific options.
`;

const COMMANDS: Record<string, (argv: string[]) => Promise<number>> = {
  validate: runValidate,
  render: runRender,
  sheet: runSheet,
  generate: runGenerate,
  revise: runRevise,
};

/** Run only when this file is the process entrypoint (not when imported by tests). */
export function isEntrypoint(): boolean {
  return process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href;
}

export async function main(argv: string[]): Promise<number> {
  const [command, ...rest] = argv;
  if (command === undefined || command === '--help' || command === '-h') {
    process.stdout.write(TOP_HELP);
    return EXIT.OK;
  }
  const handler = COMMANDS[command];
  if (handler === undefined) {
    process.stderr.write(`unknown command "${command}"\n\n${TOP_HELP}`);
    return EXIT.USAGE;
  }
  try {
    return await handler(rest);
  } catch (err) {
    if (err instanceof CliError) {
      process.stderr.write(`${err.message}\n`);
      return err.exitCode;
    }
    process.stderr.write(`${(err as Error).stack ?? (err as Error).message}\n`);
    return EXIT.PROVIDER;
  }
}


if (isEntrypoint()) {
  main(process.argv.slice(2)).then((code) => {
    process.exitCode = code;
  });
}
