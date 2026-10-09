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

export function createConfiguration({ installRoot, candidatesFile, port = DEFAULT_READINESS_PORT,
  nodeExecutable, pm3Home, callbackFile, runDirectory }) {
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
  const selected = [nodeExecutable, pm3Home, callbackFile, runDirectory];
  if (selected.some(v => v !== undefined)) {
    if (selected.some(v => v === undefined)) throw new Error('Installed supervision requires nodeExecutable, pm3Home, callbackFile and runDirectory');
    nodeExecutable = absolutePath(nodeExecutable, 'nodeExecutable');
    pm3Home = absolutePath(pm3Home, 'pm3Home');
    callbackFile = absolutePath(callbackFile, 'callbackFile');
    runDirectory = absolutePath(runDirectory, 'runDirectory');
    const service = config.services[0];
    Object.assign(service.env, { DESK_SUPERVISION: 'pm3', DESK_PM3_SERVICE: 'moveup/desk',
      DESK_PM3_NODE: nodeExecutable, DESK_PM3_HOME: pm3Home });
    // PM3 callbacks inherit daemon env, not the candidate's service env.
    service.supervised.promote.callback = [nodeExecutable, callbackFile, path.dirname(candidatesFile), runDirectory,
      pm3Home, String(port)];
  }
  return { config };
}
