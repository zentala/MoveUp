import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { validateOffline, VALIDATION_MAX_BYTES, VALIDATION_TIMEOUT_MS } from './pm3-validation.mjs';

const options = { nodeExecutable: process.execPath,
  pm3Home: path.resolve('synthetic pm3 & home'),
  candidatesFile: path.resolve('synthetic app', 'candidates.json'),
  configFile: path.resolve('synthetic output', 'pm3.yaml') };
const report = { ok: true, version: 1, problems: [], candidates: [{
  id: '0.6.1', argv0: path.resolve('synthetic app', 'desk.exe'), artifact: null,
  expectJson: { app_id: 'io.zntl.desk', ready: true, version: '0.6.1' },
}] };

test('public launcher uses explicit Node, structured argv, no shell, hidden window and finite bounds', () => {
  const result = validateOffline(options, (executable, argv, settings) => {
    assert.equal(executable, process.execPath);
    assert.deepEqual(argv, [path.join(options.pm3Home, 'releases', 'launch.mjs'),
      'candidates', 'validate', options.candidatesFile, '--config', options.configFile,
      '--service', 'desk', '--json']);
    assert.equal(settings.shell, false);
    assert.equal(settings.windowsHide, true);
    assert.equal(settings.timeout, 10000);
    assert.equal(settings.timeout, VALIDATION_TIMEOUT_MS);
    assert.equal(settings.maxBuffer, 256 * 1024);
    assert.equal(settings.maxBuffer, VALIDATION_MAX_BYTES);
    assert.deepEqual(settings.stdio, ['ignore', 'pipe', 'pipe']);
    assert.equal(settings.cwd, path.dirname(options.configFile));
    return JSON.stringify(report);
  });
  assert.deepEqual(result, { status: 'offline-valid', report });
});

for (const [name, error, expected] of [
  ['timeout', { code: 'ETIMEDOUT' }, /ETIMEDOUT/],
  ['missing executable', { code: 'ENOENT' }, /ENOENT/],
  ['spawn permission', { code: 'EPERM' }, /EPERM/],
  ['output overflow', { code: 'ENOBUFS' }, /ENOBUFS/],
  ['invalid candidates', { status: 1 }, /invalid/],
  ['unsupported command', { status: 2 }, /usage-or-unsupported/],
  ['signal termination', { status: null, signal: 'SIGTERM' }, /failed/],
]) {
  test(`offline ${name} is a refusal, never a pending success`, () => {
    assert.throws(() => validateOffline(options, () => { throw error; }), expected);
  });
}

for (const [name, value] of [
  ['malformed JSON', 'not JSON'], ['null', 'null'],
  ['false ok', JSON.stringify({ ...report, ok: false })],
  ['unknown version', JSON.stringify({ ...report, version: 2 })],
  ['reported problem', JSON.stringify({ ...report, problems: ['bad'] })],
  ['missing candidates', JSON.stringify({ ok: true, version: 1, problems: [] })],
  ['empty candidates', JSON.stringify({ ...report, candidates: [] })],
  ['incomplete candidate', JSON.stringify({ ...report, candidates: [{ id: 'a' }] })],
  ['oversize output', ' '.repeat(VALIDATION_MAX_BYTES + 1)],
]) {
  test(`refuses ${name}`, () => {
    assert.throws(() => validateOffline(options, () => value), /malformed, unsupported or non-valid/);
  });
}

test('relative paths never reach the process boundary', () => {
  for (const key of Object.keys(options)) {
    assert.throws(() => validateOffline({ ...options, [key]: 'relative' }, () => assert.fail('must not launch')), /absolute/);
  }
});
