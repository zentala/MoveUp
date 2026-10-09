import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { dispatch } from './release-callback.mjs';

function fixture(t) {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), 'moveup-callback-'));
  t.after(() => fs.rmSync(base, { recursive: true, force: true }));
  const root = path.join(base, 'release store'); const run = path.join(base, 'runtime');
  fs.mkdirSync(root); fs.mkdirSync(run);
  const candidates = ['0.6.2', '0.6.1'].map(id => {
    fs.mkdirSync(path.join(root, id));
    const exe = path.join(root, id, 'desk.exe'); fs.writeFileSync(exe, 'synthetic artifact');
    return { id, argv: [exe, '--minimized'] };
  });
  fs.writeFileSync(path.join(root, 'candidates.json'), JSON.stringify({ version: 1, candidates }));
  return { root, run, candidates, env: { PM3_CANDIDATE_ID: '0.6.1', PM3_CANDIDATE_PID: '123' } };
}

test('observed fallback is dispatched through structured maintenance arguments', t => {
  const f = fixture(t);
  // Newest executable is broken; it must never be invoked by this dispatcher.
  fs.writeFileSync(f.candidates[0].argv[0], 'broken executable');
  const report = dispatch(f.root, f.run, { env: f.env, execute(exe, argv, options) {
    assert.equal(exe, f.candidates[1].argv[0]);
    assert.deepEqual(argv, ['--release', 'promote', '--root', f.root, '--run-directory', f.run]);
    assert.equal(options.shell, false); assert.equal(options.windowsHide, true);
    assert.equal(options.timeout, 8500); assert.equal(options.env, f.env);
    return { status: 0, stdout: '{"status":"promoted","version":"0.6.1","pid":123}' };
  } });
  assert.equal(report.status, 'promoted');
});

test('callback errors, deadlines and unknown acknowledgements fail explicitly', t => {
  const f = fixture(t);
  for (const result of [
    { status: 1 }, { error: new Error('timeout') }, { signal: 'SIGKILL' },
    { status: 0, stdout: '{}' }, { status: 0, stdout: 'not-json' },
    { status: 0, stdout: '{"status":"promoted","version":"0.6.2","pid":123}' },
    { status: 0, stdout: '{"status":"promoted","version":"0.6.1","pid":999}' },
  ]) assert.throws(() => dispatch(f.root, f.run, { env: f.env, execute: () => result }));
});

test('callback carries selected supervision settings without relying on service env inheritance', t => {
  const f = fixture(t);
  dispatch(f.root, f.run, { env: f.env, pm3Home: f.run, port: '43391', execute(exe, argv, options) {
    assert.equal(options.env.DESK_SUPERVISION, 'pm3');
    assert.equal(options.env.DESK_PM3_HOME, f.run);
    assert.equal(options.env.DESK_PM3_NODE, process.execPath);
    assert.equal(options.env.DESK_PM3_SERVICE, 'moveup/desk');
    assert.equal(options.env.DESK_READINESS_PORT, '43391');
    assert.equal(options.env.PM3_CANDIDATE_ID, '0.6.1');
    return { status: 0, stdout: '{"status":"promoted","version":"0.6.1","pid":123}' };
  } });
  assert.throws(() => dispatch(f.root, f.run, { env: f.env, pm3Home: f.run, port: '0' }), /Invalid readiness port/);
});

test('invalid, unlisted, escaped and duplicate candidates never dispatch', t => {
  const f = fixture(t); let calls = 0;
  const execute = () => { calls++; throw new Error('should not execute'); };
  for (const id of ['../outside', '0.6.3', 'C:/outside', '00.6.1', '0.6.1;command']) {
    assert.throws(() => dispatch(f.root, f.run, { env: { ...f.env, PM3_CANDIDATE_ID: id }, execute }));
  }
  const file = path.join(f.root, 'candidates.json');
  fs.writeFileSync(file, JSON.stringify({ version: 1, candidates: [f.candidates[1], f.candidates[1]] }));
  assert.throws(() => dispatch(f.root, f.run, { env: f.env, execute }));
  fs.writeFileSync(file, JSON.stringify({ version: 1, candidates: [{ ...f.candidates[1], argv: [path.join(f.run, 'foreign.exe')] }] }));
  assert.throws(() => dispatch(f.root, f.run, { env: f.env, execute }));
  assert.equal(calls, 0);
});
