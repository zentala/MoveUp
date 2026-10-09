// Pure preparation of the frozen supervised-application v1 contract.
import path from 'node:path';

export const DEFAULT_READINESS_PORT = 3391;
export const EXPECT_JSON = Object.freeze({ app_id: 'io.zntl.desk', ready: true });

export function absolutePath(value, label) {
  if (typeof value !== 'string' || !path.isAbsolute(value)) {
    throw new Error(`${label} must be an absolute path on this machine`);
  }
  return path.normalize(value);
}

export function createConfiguration({ installRoot, candidatesFile, port = DEFAULT_READINESS_PORT }) {
  installRoot = absolutePath(installRoot, 'installRoot');
  candidatesFile = absolutePath(candidatesFile, 'candidatesFile');
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error('DESK_READINESS_PORT must be an integer from 1 to 65535');
  }
  const config = {
    version: 1,
    name: 'moveup',
    nodeAutoInstall: false,
    services: [{
      name: 'desk', boot: false,
      cwd: installRoot, envFile: [],
      env: { DESK_READINESS_PORT: String(port) },
      supervised: {
        candidatesFile,
        missIntervalMs: 60_000, missThreshold: 2, standDown: true,
        window: 'visible',
        instance: { pidField: 'pid', runIdField: 'supervisor_run_id' },
        promote: { afterMs: 30_000, callbackTimeoutMs: 10_000 },
      },
      health: {
        type: 'http', url: `http://127.0.0.1:${port}/display/readiness`,
        port, intervalMs: 60_000, timeoutMs: 800, maxBodyBytes: 4096,
        expectJson: { ...EXPECT_JSON },
      },
    }],
  };
  return { config };
}
