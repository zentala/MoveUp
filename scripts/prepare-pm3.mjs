// Opt-in file preparation. No service registration or application launch.
import * as fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { fileURLToPath } from 'node:url';
import { createConfiguration, absolutePath, DEFAULT_READINESS_PORT } from './pm3-config.mjs';
import { validateOffline } from './pm3-validation.mjs';

export function prepare(options, { io = fs, execute } = {}) {
  const installRoot = fs.realpathSync(absolutePath(options['install-root'], '--install-root'));
  const outputDirectory = fs.realpathSync(absolutePath(options['output-dir'], '--output-dir'));
  // Preserve the app-owned pathname so atomic replacement by the app stays visible.
  const candidatesFile = absolutePath(options['candidates-file'], '--candidates-file');
  if (!fs.statSync(candidatesFile).isFile()) throw new Error('--candidates-file must name the app-owned JSON file');
  JSON.parse(fs.readFileSync(candidatesFile, 'utf8')); // Syntax only; schema belongs to PM3.
  if (!fs.statSync(installRoot).isDirectory()) throw new Error('--install-root must be a directory');
  if (options['ordered-candidates'] !== undefined || options['pm3-source'] !== undefined) {
    throw new Error('Use --candidates-file with the app-owned JSON projection; source loading and candidate generation are no longer supported');
  }
  const portText = options.port ?? String(DEFAULT_READINESS_PORT);
  if (!/^\d+$/.test(portText)) throw new Error('--port must be an integer from 1 to 65535');
  const nodeExecutable = options['pm3-node'];
  const pm3Home = options['pm3-home'];
  if ((nodeExecutable !== undefined) !== (pm3Home !== undefined)) {
    throw new Error('Offline validation requires both --pm3-node ABS and --pm3-home ABS');
  }
  const runDirectory = options['run-directory'];
  const callbackFile = runDirectory === undefined ? undefined : path.join(outputDirectory, 'release-callback.mjs');
  if (runDirectory !== undefined && (!fs.statSync(absolutePath(runDirectory, '--run-directory')).isDirectory()
      || fs.existsSync(callbackFile))) throw new Error('Run directory must exist and callback output must be new');
  const { config } = createConfiguration({ installRoot, candidatesFile, port: Number(portText),
    configFile: path.join(outputDirectory, 'pm3.yaml'),
    ...(runDirectory === undefined ? {} : { nodeExecutable, pm3Home, callbackFile, runDirectory }) });
  const destination = path.join(outputDirectory, 'pm3.yaml');
  if (fs.existsSync(destination)) throw new Error('Output pm3.yaml already exists; use a new isolated output directory');
  const temporaryDirectory = io.mkdtempSync(path.join(outputDirectory, '.moveup-pm3-'));
  const temporaryFile = path.join(temporaryDirectory, 'pm3.yaml');
  let validation = { status: 'pending-not-validated',
    reason: 'No offline PM3 validator selected. Supply --pm3-node ABS and --pm3-home ABS after owner acceptance.' };
  const cleanupWarnings = [];
  let failed = false;
  let primaryError;
  let callbackPublished = false;
  try {
    const fd = io.openSync(temporaryFile, 'wx');
    try {
      io.writeFileSync(fd, JSON.stringify(config, null, 2) + '\n');
      io.fsyncSync(fd);
    } finally { io.closeSync(fd); }
    if (nodeExecutable !== undefined) {
      validation = validateOffline({ nodeExecutable, pm3Home, candidatesFile, configFile: temporaryFile }, execute);
    }
    if (callbackFile !== undefined) {
      io.copyFileSync(fileURLToPath(new URL('./release-callback.mjs', import.meta.url)),
        path.join(temporaryDirectory, 'release-callback.mjs'));
      io.linkSync(path.join(temporaryDirectory, 'release-callback.mjs'), callbackFile);
      callbackPublished = true;
    }
    // Hard-link publication is atomic and fails if any competing output exists.
    // rename would overwrite on POSIX; writing destination directly exposes partial bytes.
    io.linkSync(temporaryFile, destination);
  } catch (error) {
    failed = true;
    primaryError = error;
    if (callbackPublished) {
      try { io.unlinkSync(callbackFile); }
      catch (cleanup) { cleanupWarnings.push({ operation: 'unlinkSync', path: callbackFile,
        code: cleanup.code ?? null, message: cleanup.message }); }
    }
  } finally {
    // This invocation owns the newly created directory and this one temporary file.
    // Cleanup must never change the publication outcome or remove the final path.
    for (const [operation, target] of [...(callbackFile === undefined ? [] : [['unlinkSync', path.join(temporaryDirectory, 'release-callback.mjs')]]),
      ['unlinkSync', temporaryFile], ['rmdirSync', temporaryDirectory]]) {
      try {
        io[operation](target);
      } catch (error) {
        if (error.code !== 'ENOENT') cleanupWarnings.push({ operation, path: target,
          code: error.code ?? null, message: error.message });
      }
    }
  }
  if (failed) {
    // Preserve the original thrown value, even when it cannot carry diagnostics.
    try { primaryError.cleanupWarnings = cleanupWarnings; } catch { /* Non-extensible error. */ }
    throw primaryError;
  }
  return { publication: 'published', cleanupWarnings,
    files: callbackFile === undefined ? [destination] : [destination, callbackFile], candidatesFile, validation, readyToInstall: false,
    integration: 'pending-pm3-owner-acceptance', registration: 'not-performed' };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const { values } = parseArgs({ options: {
      'install-root': { type: 'string' }, 'output-dir': { type: 'string' },
      'candidates-file': { type: 'string' }, 'pm3-node': { type: 'string' }, 'pm3-home': { type: 'string' },
      port: { type: 'string' }, help: { type: 'boolean' },
      'run-directory': { type: 'string' },
    } });
    if (values.help) {
      console.log('node scripts/prepare-pm3.mjs --install-root ABS --candidates-file ABS --output-dir ABS [--port 3391] [--pm3-node ABS --pm3-home ABS] [--run-directory ABS]\nWith run-directory and selected Node/home, publishes the installed callback and supervision environment. Without a validator: pending/not-validated. No service is registered; owner acceptance remains required.');
    } else console.log(JSON.stringify(prepare(values), null, 2));
  } catch (error) {
    console.error(`PM3 preparation refused: ${error.message}`);
    for (const warning of error.cleanupWarnings ?? []) {
      console.error(`PM3 cleanup warning: ${JSON.stringify(warning)}`);
    }
    process.exitCode = 1;
  }
}
