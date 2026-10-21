// Optional public PM3 lint. It never registers or starts a service.
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { absolutePath } from './pm3-config.mjs';

export const VALIDATION_TIMEOUT_MS = 10_000;
export const VALIDATION_MAX_BYTES = 256 * 1024;

export function validateOffline({ nodeExecutable, pm3Home, configFile, isolatedHome }, execute = execFileSync) {
  nodeExecutable = absolutePath(nodeExecutable, '--pm3-node');
  pm3Home = absolutePath(pm3Home, '--pm3-home');
  configFile = absolutePath(configFile, 'validation config');
  isolatedHome = absolutePath(isolatedHome, 'isolated lint home');
  const argv = [path.join(pm3Home, 'releases', 'launch.mjs'), 'lint', configFile];
  try {
    const output = execute(nodeExecutable, argv, {
      shell: false, windowsHide: true, timeout: VALIDATION_TIMEOUT_MS,
      maxBuffer: VALIDATION_MAX_BYTES, encoding: 'utf8', cwd: path.dirname(configFile),
      env: { ...process.env, PM3_HOME: isolatedHome, PM3_CONFIG: configFile },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    if (typeof output !== 'string' || Buffer.byteLength(output) > VALIDATION_MAX_BYTES) {
      throw Object.assign(new Error('Invalid lint output'), { code: 'INVALID_OUTPUT' });
    }
  } catch (error) {
    const reason = error.code ?? (error.status === 2 ? 'usage-or-unsupported' : 'failed');
    throw new Error('PM3 lint did not pass (' + reason + '); no configuration was published.');
  }
  // PM3 lint warnings are non-blocking; exit zero is not runtime readiness.
  return { status: 'lint-passed', warningsAreNonBlocking: true, runtime: 'not-checked' };
}
