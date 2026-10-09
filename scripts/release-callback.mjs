// Stable Node dispatcher selects the observed retained candidate, never newest.
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

function noLinks(target) {
  if (!path.isAbsolute(target) || target.split(/[\\/]/).includes('..')) throw new Error('Absolute contained paths required');
  const parts = path.resolve(target).slice(path.parse(target).root.length).split(path.sep);
  let current = path.parse(target).root;
  for (const part of parts) {
    current = path.join(current, part);
    if (fs.lstatSync(current).isSymbolicLink()) throw new Error('Symlink/junction paths forbidden');
  }
}

export function dispatch(root, runDirectory, { env = process.env, execute = spawnSync, pm3Home, port } = {}) {
  noLinks(root); noLinks(runDirectory);
  if (pm3Home !== undefined) {
    noLinks(pm3Home);
    if (!/^\d+$/.test(String(port)) || Number(port) < 1 || Number(port) > 65535) throw new Error('Invalid readiness port');
    env = { ...env, DESK_SUPERVISION: 'pm3', DESK_PM3_SERVICE: 'moveup/desk',
      DESK_PM3_NODE: process.execPath, DESK_PM3_HOME: pm3Home, DESK_READINESS_PORT: String(port) };
  }
  const id = env.PM3_CANDIDATE_ID;
  if (!/^(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})$/.test(id ?? '')) throw new Error('Invalid observed candidate');
  const documentFile = path.join(root, 'candidates.json');
  noLinks(documentFile);
  const descriptor = fs.openSync(documentFile, 'r');
  let raw;
  try {
    const bytes = Buffer.alloc(1048577); let length = 0;
    while (length < bytes.length) {
      const count = fs.readSync(descriptor, bytes, length, bytes.length - length, null);
      if (count === 0) break;
      length += count;
    }
    raw = bytes.subarray(0, length);
  } finally { fs.closeSync(descriptor); }
  if (raw.length > 1048576) throw new Error('Oversized candidate document');
  const document = JSON.parse(raw);
  if (document.version !== 1 || !Array.isArray(document.candidates)) throw new Error('Unsupported candidate document');
  const matches = document.candidates.filter(c => c.id === id);
  const executable = path.join(root, id, 'desk.exe');
  if (matches.length !== 1 || matches[0].argv?.[0] !== executable) throw new Error('Observed candidate no longer listed');
  noLinks(executable);
  const result = execute(executable, ['--release', 'promote', '--root', root, '--run-directory', runDirectory],
    { env, shell: false, windowsHide: true, timeout: 8500, maxBuffer: 65536, encoding: 'utf8' });
  if (result.error || result.status !== 0 || result.signal) throw new Error(`Maintenance callback failed: ${result.error?.message ?? result.status ?? result.signal}`);
  let report;
  try { report = JSON.parse(result.stdout); } catch { throw new Error('Invalid maintenance acknowledgement'); }
  if (report.status !== 'promoted' || report.version !== id || String(report.pid) !== env.PM3_CANDIDATE_PID) {
    throw new Error('Maintenance callback did not acknowledge observed promotion');
  }
  return report;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 6) throw new Error('Expected release root, run directory, selected PM3 home and readiness port');
    console.log(JSON.stringify(dispatch(process.argv[2], process.argv[3], { pm3Home: process.argv[4], port: process.argv[5] })));
  } catch (error) {
    console.error(JSON.stringify({ status: 'failed', error: error.message }));
    process.exitCode = 1;
  }
}
