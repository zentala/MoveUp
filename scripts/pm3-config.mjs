// Ordinary installed PM3 configuration; no candidate or callback protocol.
import path from 'node:path';
import os from 'node:os';

export const DEFAULT_READINESS_PORT = 3391;

export function absolutePath(value, label) {
  if (typeof value !== 'string' || !path.isAbsolute(value)) {
    throw new Error(label + ' must be an absolute path on this machine');
  }
  return path.normalize(value);
}

export function createConfiguration({ installRoot, executable, port = DEFAULT_READINESS_PORT,
  nodeExecutable = process.execPath, pm3Home = path.join(os.homedir(), '.pm3'), configFile }) {
  installRoot = absolutePath(installRoot, 'installRoot');
  executable = absolutePath(executable, 'executable');
  nodeExecutable = absolutePath(nodeExecutable, 'nodeExecutable');
  pm3Home = absolutePath(pm3Home, 'pm3Home');
  if (configFile !== undefined) configFile = absolutePath(configFile, 'configFile');
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error('DESK_READINESS_PORT must be an integer from 1 to 65535');
  }
  return { config: {
    version: 1, name: 'moveup', nodeAutoInstall: false,
    services: [{
      name: 'desk', cmd: [executable, '--minimized'], cwd: installRoot,
      envFile: [], env: { DESK_READINESS_PORT: String(port), DESK_SUPERVISION: 'pm3',
        DESK_PM3_SERVICE: 'moveup/desk', DESK_PM3_NODE: nodeExecutable, DESK_PM3_HOME: pm3Home,
        ...(configFile === undefined ? {} : { DESK_PM3_CONFIG: configFile, PM3_CONFIG: configFile }) },
      boot: true, autorestart: true,
      backoff: { max: 0, baseMs: 1000, maxMs: 30_000 },
      health: {
        type: 'http', url: 'http://127.0.0.1:' + port + '/display/readiness',
        port, intervalMs: 2000, timeoutMs: 800, startPeriodMs: 30_000,
      },
    }],
  } };
}
