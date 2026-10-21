import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { validateOffline, VALIDATION_MAX_BYTES, VALIDATION_TIMEOUT_MS } from './pm3-validation.mjs';

const options = { nodeExecutable: process.execPath,
  pm3Home: path.resolve('synthetic pm3 & home'),
  configFile: path.resolve('synthetic output', 'pm3.yaml'),
  isolatedHome: path.resolve('synthetic isolated home') };

test('public lint uses structured hidden bounded process and isolated PM3 home', () => {
  const result = validateOffline(options, (executable, argv, settings) => {
    assert.equal(executable, process.execPath);
    assert.deepEqual(argv, [path.join(options.pm3Home, 'releases', 'launch.mjs'), 'lint', options.configFile]);
    assert.equal(settings.shell, false);
    assert.equal(settings.windowsHide, true);
    assert.equal(settings.timeout, VALIDATION_TIMEOUT_MS);
    assert.equal(settings.maxBuffer, VALIDATION_MAX_BYTES);
    assert.deepEqual(settings.stdio, ['ignore', 'pipe', 'pipe']);
    assert.equal(settings.cwd, path.dirname(options.configFile));
    assert.equal(settings.env.PM3_HOME, options.isolatedHome);
    assert.equal(settings.env.PM3_CONFIG, options.configFile);
    return 'WARN portable path: lint warnings are non-blocking';
  });
  assert.deepEqual(result, { status: 'lint-passed', warningsAreNonBlocking: true, runtime: 'not-checked' });
});

for (const [name, error, expected] of [
  ['timeout', { code: 'ETIMEDOUT' }, /ETIMEDOUT/],
  ['missing executable', { code: 'ENOENT' }, /ENOENT/],
  ['filesystem permission', { code: 'EPERM' }, /EPERM/],
  ['output overflow', { code: 'ENOBUFS' }, /ENOBUFS/],
  ['lint failure', { status: 1 }, /failed/],
  ['unsupported command', { status: 2 }, /usage-or-unsupported/],
  ['signal termination', { status: null, signal: 'SIGTERM' }, /failed/],
]) {
  test(name + ' is a failure, never successful validation', () => {
    assert.throws(() => validateOffline(options, () => { throw error; }), expected);
  });
}
test('invalid output and relative paths never appear validated', () => {
  for (const output of [undefined, Buffer.from('text'), ' '.repeat(VALIDATION_MAX_BYTES + 1)]) {
    assert.throws(() => validateOffline(options, () => output), /INVALID_OUTPUT/);
  }
  for (const key of Object.keys(options)) {
    assert.throws(() => validateOffline({ ...options, [key]: 'relative' }, () => assert.fail('must not launch')), /absolute/);
  }
});
