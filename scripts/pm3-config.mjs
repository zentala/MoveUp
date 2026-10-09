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

export function createConfiguration({ installRoot, outputDirectory, versions, port = DEFAULT_READINESS_PORT }) {
  installRoot = absolutePath(installRoot, 'installRoot');
  outputDirectory = absolutePath(outputDirectory, 'outputDirectory');
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error('DESK_READINESS_PORT must be an integer from 1 to 65535');
  }
  if (!Array.isArray(versions) || versions.length === 0 || versions.length > 3
      || new Set(versions).size !== versions.length
      || versions.some(v => typeof v !== 'string' || !/^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$/.test(v))) {
    throw new Error('The ordered candidate list must contain 1–3 unique version identifiers');
  }
  const candidates = {
    version: 1,
    candidates: versions.map(version => ({
      id: version,
      argv: [path.join(installRoot, 'releases', version, 'desk.exe'), '--minimized'],
      health: { expectJson: { version } },
    })),
  };
  const config = {
    version: 1,
    name: 'moveup',
    nodeAutoInstall: false,
    services: [{
      name: 'desk', boot: false,
      cwd: installRoot, envFile: [],
      env: { DESK_READINESS_PORT: String(port) },
      supervised: {
        candidatesFile: path.join(outputDirectory, 'candidates.json'),
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
  return { config, candidates };
}
