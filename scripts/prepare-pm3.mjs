// Opt-in file preparation only. No registration, process launch or machine changes.
import { readFileSync, realpathSync, statSync, writeFileSync, existsSync } from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { fileURLToPath } from 'node:url';
import { createConfiguration, absolutePath, DEFAULT_READINESS_PORT } from './pm3-config.mjs';
import { loadPm3Source, validateConfiguration } from './pm3-source-validation.mjs';

export function prepare(options) {
  const installRoot = realpathSync(absolutePath(options['install-root'], '--install-root'));
  const outputDirectory = realpathSync(absolutePath(options['output-dir'], '--output-dir'));
  const input = absolutePath(options['ordered-candidates'], '--ordered-candidates');
  const portText = options.port ?? String(DEFAULT_READINESS_PORT);
  if (!/^\d+$/.test(portText)) throw new Error('--port must be an integer from 1 to 65535');
  const versions = readFileSync(input, 'utf8').trim().split(/\r?\n/);
  const documents = createConfiguration({ installRoot, outputDirectory, versions, port: Number(portText) });
  for (const candidate of documents.candidates.candidates) {
    if (!statSync(candidate.argv[0]).isFile()) throw new Error(`Candidate executable is missing: ${candidate.argv[0]}`);
    candidate.argv[0] = realpathSync(candidate.argv[0]);
  }
  const api = loadPm3Source(options['pm3-source'] ?? process.env.PM3_SOURCE);
  validateConfiguration(api, documents);
  const files = [path.join(outputDirectory, 'candidates.json'), path.join(outputDirectory, 'pm3.yaml')];
  if (files.some(existsSync)) throw new Error('Output files already exist. Use a new isolated output directory; existing services are never overwritten.');
  // JSON is valid YAML; avoid another serializer and preserve scalar types exactly.
  writeFileSync(files[0], JSON.stringify(documents.candidates, null, 2) + '\n', { flag: 'wx' });
  writeFileSync(files[1], JSON.stringify(documents.config, null, 2) + '\n', { flag: 'wx' });
  return { files, sourceHashes: api.hashes, validation: 'schema-passed',
    integration: 'pending-pm3-owner-acceptance', registration: 'not-performed' };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const { values } = parseArgs({ options: {
      'install-root': { type: 'string' }, 'output-dir': { type: 'string' },
      'ordered-candidates': { type: 'string' }, 'pm3-source': { type: 'string' },
      port: { type: 'string' }, help: { type: 'boolean' },
    } });
    if (values.help) {
      console.log('node scripts/prepare-pm3.mjs --install-root ABS --ordered-candidates ABS --output-dir ABS [--pm3-source ABS] [--port 3391]\nOutput directory must exist and be empty of pm3.yaml/candidates.json. Preparation only; no service is registered.');
    } else console.log(JSON.stringify(prepare(values), null, 2));
  } catch (error) {
    console.error(`PM3 preparation refused: ${error.message}`);
    process.exitCode = 1;
  }
}
