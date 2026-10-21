// Opt-in file preparation. No service registration or application launch.
import * as fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { fileURLToPath } from 'node:url';
import { createConfiguration, absolutePath, DEFAULT_READINESS_PORT } from './pm3-config.mjs';
import { validateOffline } from './pm3-validation.mjs';

export function prepare(options, { io = fs, execute } = {}) {
  for (const retired of ['candidates-file', 'ordered-candidates', 'pm3-source', 'run-directory']) {
    if (options[retired] !== undefined) throw new Error('--' + retired + ' is retired; use --executable with ordinary PM3');
  }
  const installRoot = fs.realpathSync(absolutePath(options['install-root'], '--install-root'));
  const outputDirectory = fs.realpathSync(absolutePath(options['output-dir'], '--output-dir'));
  const executable = fs.realpathSync(absolutePath(options.executable, '--executable'));
  if (!fs.statSync(installRoot).isDirectory() || !fs.statSync(outputDirectory).isDirectory()) {
    throw new Error('Install root and output directory must be directories');
  }
  if (!fs.statSync(executable).isFile()) throw new Error('--executable must name an existing application file');
  const portText = options.port ?? String(DEFAULT_READINESS_PORT);
  if (!/^\d+$/.test(portText)) throw new Error('--port must be an integer from 1 to 65535');
  const nodeExecutable = options['pm3-node'];
  const pm3Home = options['pm3-home'];
  if ((nodeExecutable !== undefined) !== (pm3Home !== undefined)) {
    throw new Error('Optional PM3 lint requires both --pm3-node ABS and --pm3-home ABS');
  }
  const destination = path.join(outputDirectory, 'pm3.yaml');
  const { config } = createConfiguration({ installRoot, executable, port: Number(portText),
    configFile: destination, nodeExecutable, pm3Home });
  if (fs.existsSync(destination)) throw new Error('Output pm3.yaml already exists; use a new isolated output directory');
  const temporaryDirectory = io.mkdtempSync(path.join(outputDirectory, '.moveup-pm3-'));
  const temporaryFile = path.join(temporaryDirectory, 'pm3.yaml');
  let validation = { status: 'not-checked', reason: 'Optional installed PM3 lint was not selected' };
  const cleanupWarnings = [];
  let failed = false;
  let primaryError;
  try {
    const fd = io.openSync(temporaryFile, 'wx');
    try {
      io.writeFileSync(fd, JSON.stringify(config, null, 2) + '\n');
      io.fsyncSync(fd);
    } finally { io.closeSync(fd); }
    if (nodeExecutable !== undefined) {
      validation = validateOffline({ nodeExecutable, pm3Home, configFile: temporaryFile,
        isolatedHome: path.join(temporaryDirectory, 'lint-home') }, execute);
    }
    // Hard-link publication is atomic and cannot overwrite a competing file.
    io.linkSync(temporaryFile, destination);
  } catch (error) {
    failed = true;
    primaryError = error;
  } finally {
    for (const [operation, target] of [['unlinkSync', temporaryFile], ['rmdirSync', temporaryDirectory]]) {
      try { io[operation](target); }
      catch (error) {
        if (error.code !== 'ENOENT') cleanupWarnings.push({ operation, path: target,
          code: error.code ?? null, message: error.message });
      }
    }
  }
  if (failed) {
    try { primaryError.cleanupWarnings = cleanupWarnings; } catch { /* Non-extensible error. */ }
    throw primaryError;
  }
  return { publication: 'published', cleanupWarnings, files: [destination], validation,
    registration: 'not-performed', runtime: 'not-checked', readyToInstall: false };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const { values } = parseArgs({ options: {
      'install-root': { type: 'string' }, 'output-dir': { type: 'string' },
      executable: { type: 'string' }, 'pm3-node': { type: 'string' }, 'pm3-home': { type: 'string' },
      port: { type: 'string' }, help: { type: 'boolean' },
    } });
    if (values.help) {
      console.log('node scripts/prepare-pm3.mjs --install-root ABS --executable ABS --output-dir ABS [--port 3391] [--pm3-node ABS --pm3-home ABS]\nWrites ordinary cmd/autorestart configuration only. Optional public PM3 lint uses an isolated home. No registration, launch or runtime verification.');
    } else console.log(JSON.stringify(prepare(values), null, 2));
  } catch (error) {
    console.error('PM3 preparation failed: ' + error.message);
    for (const warning of error.cleanupWarnings ?? []) console.error('PM3 cleanup warning: ' + JSON.stringify(warning));
    process.exitCode = 1;
  }
}
