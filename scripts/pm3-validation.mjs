// Public offline process boundary only; no PM3 source imports or shell launcher.
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { absolutePath } from './pm3-config.mjs';

export const VALIDATION_TIMEOUT_MS = 10_000;
export const VALIDATION_MAX_BYTES = 256 * 1024;

export function validateOffline({ nodeExecutable, pm3Home, candidatesFile, configFile }, execute = execFileSync) {
  nodeExecutable = absolutePath(nodeExecutable, '--pm3-node');
  pm3Home = absolutePath(pm3Home, '--pm3-home');
  candidatesFile = absolutePath(candidatesFile, '--candidates-file');
  configFile = absolutePath(configFile, 'validation config');
  const argv = [path.join(pm3Home, 'releases', 'launch.mjs'),
    'candidates', 'validate', candidatesFile, '--config', configFile, '--service', 'desk', '--json'];
  let stdout;
  try {
    stdout = execute(nodeExecutable, argv, {
      shell: false, windowsHide: true, timeout: VALIDATION_TIMEOUT_MS,
      maxBuffer: VALIDATION_MAX_BYTES, encoding: 'utf8', cwd: path.dirname(configFile),
      stdio: ['ignore', 'pipe', 'pipe'],
    });
  } catch (error) {
    // Keep bounded failure identity, never echo arbitrary child output.
    const reason = error.code ?? (error.status === 1 ? 'invalid' : error.status === 2 ? 'usage-or-unsupported' : 'failed');
    throw new Error(`Offline PM3 validation did not pass (${reason}). Obtain the owner's supported offline candidates validate capability; no configuration was published.`);
  }
  let report;
  try {
    if (typeof stdout !== 'string' || Buffer.byteLength(stdout) > VALIDATION_MAX_BYTES) throw new Error();
    report = JSON.parse(stdout);
    if (report?.ok !== true || report.version !== 1 || !Array.isArray(report.problems)
        || report.problems.length !== 0 || !Array.isArray(report.candidates) || report.candidates.length === 0
        || !report.candidates.every(c => c && typeof c.id === 'string' && c.id.length > 0
          && typeof c.argv0 === 'string' && path.isAbsolute(c.argv0)
          && Object.hasOwn(c, 'artifact') && c.expectJson && typeof c.expectJson === 'object'
          && !Array.isArray(c.expectJson))) throw new Error();
  } catch {
    throw new Error('Offline PM3 validation returned malformed, unsupported or non-valid JSON; no configuration was published.');
  }
  return { status: 'offline-valid', report };
}
