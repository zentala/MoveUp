import test from 'node:test';
import assert from 'node:assert/strict';
import * as fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createConfiguration } from './pm3-config.mjs';
import { prepare } from './prepare-pm3.mjs';

function fixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'moveup-pm3-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const installRoot = path.join(root, 'installed app');
  const outputDirectory = path.join(root, 'output');
  fs.mkdirSync(installRoot); fs.mkdirSync(outputDirectory);
  const executable = path.join(installRoot, 'desk.exe');
  fs.writeFileSync(executable, 'synthetic application, never executed');
  const cli = { 'install-root': installRoot, 'output-dir': outputDirectory, executable };
  return { root, installRoot, outputDirectory, executable, cli, configFile: path.join(outputDirectory, 'pm3.yaml'),
    destination: path.join(outputDirectory, 'pm3.yaml'),
    validatedCli: { ...cli, 'pm3-node': process.execPath, 'pm3-home': path.join(root, 'pm3') } };
}

test('ordinary config starts exact executable with spaces and bounded restart backoff', t => {
  const f = fixture(t);
  const { config } = createConfiguration(f);
  const service = config.services[0];
  assert.equal(config.version, 1);
  assert.equal(config.name, 'moveup');
  assert.deepEqual(service.cmd, [f.executable, '--minimized']);
  assert.equal(service.cwd, f.installRoot);
  assert.equal(service.boot, true);
  assert.equal(service.autorestart, true);
  assert.deepEqual(service.backoff, { max: 0, baseMs: 1000, maxMs: 30000 });
  assert.deepEqual(service.envFile, []);
  assert.equal(service.env.DESK_READINESS_PORT, '3391');
  assert.equal(service.env.DESK_SUPERVISION, 'pm3');
  assert.equal(service.env.DESK_PM3_SERVICE, 'moveup/desk');
  assert.equal(service.env.DESK_PM3_NODE, process.execPath);
  assert.ok(path.isAbsolute(service.env.DESK_PM3_HOME));
  assert.equal(service.env.PM3_CONFIG, f.destination);
  assert.equal(service.env.DESK_PM3_CONFIG, f.destination);
  assert.equal(service.supervised, undefined);
  assert.equal(service.health.expectJson, undefined);
  assert.equal(service.health.maxBodyBytes, undefined);
  assert.equal(config.nodeAutoInstall, false);
  assert.equal(service.health.url, 'http://127.0.0.1:3391/display/readiness');
});

test('custom readiness port stays aligned and invalid input never publishes', t => {
  const f = fixture(t);
  const { config } = createConfiguration({ ...f, port: 43391 });
  assert.equal(config.services[0].env.DESK_READINESS_PORT, '43391');
  assert.equal(config.services[0].health.port, 43391);
  assert.equal(new URL(config.services[0].health.url).port, '43391');
  for (const port of [0, 65536, 1.5, NaN, '3391']) {
    assert.throws(() => createConfiguration({ ...f, port }), /integer/);
  }
  for (const port of ['bad', '3.5', '65536', '0']) {
    assert.throws(() => prepare({ ...f.cli, port }), /integer/);
  }
  for (const key of ['installRoot', 'executable']) {
    assert.throws(() => createConfiguration({ ...f, [key]: 'relative' }), /absolute/);
  }
  assert.deepEqual(fs.readdirSync(f.outputDirectory), []);
});

test('preparation needs no candidates, callback or PM3 installation and never launches app', t => {
  const f = fixture(t);
  const result = prepare(f.cli, { execute: () => assert.fail('must not execute') });
  assert.equal(result.publication, 'published');
  assert.equal(result.validation.status, 'not-checked');
  assert.equal(result.registration, 'not-performed');
  assert.equal(result.runtime, 'not-checked');
  assert.equal(result.readyToInstall, false);
  assert.deepEqual(result.files, [f.destination]);
  assert.deepEqual(result.cleanupWarnings, []);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), ['pm3.yaml']);
  assert.deepEqual(JSON.parse(fs.readFileSync(f.destination)), createConfiguration(f).config);
  assert.equal(fs.readFileSync(f.executable, 'utf8'), 'synthetic application, never executed');
});

test('optional lint receives staged config and an isolated home, never candidates', t => {
  const f = fixture(t);
  let calls = 0;
  const result = prepare(f.validatedCli, { execute: (executable, argv, settings) => {
    calls++;
    assert.equal(executable, process.execPath);
    assert.deepEqual(argv.slice(0, 2), [path.join(f.root, 'pm3', 'releases', 'launch.mjs'), 'lint']);
    assert.equal(fs.existsSync(f.destination), false);
    assert.deepEqual(JSON.parse(fs.readFileSync(argv[2])), createConfiguration({ ...f, pm3Home: path.join(f.root, 'pm3') }).config);
    assert.equal(settings.env.PM3_CONFIG, argv[2]);
    assert.equal(settings.env.PM3_HOME, path.join(path.dirname(argv[2]), 'lint-home'));
    return 'lint: services processed 1; no findings';
  } });
  assert.equal(calls, 1);
  assert.equal(result.validation.status, 'lint-passed');
  assert.equal(result.validation.warningsAreNonBlocking, true);
  assert.equal(result.runtime, 'not-checked');
});

test('missing executable, retired options and incomplete lint selection leave no output', t => {
  const f = fixture(t);
  assert.throws(() => prepare({ ...f.cli, executable: path.join(f.root, 'missing') }), /ENOENT/);
  assert.throws(() => prepare({ ...f.cli, executable: f.root }), /application file/);
  for (const option of ['candidates-file', 'ordered-candidates', 'pm3-source', 'run-directory']) {
    assert.throws(() => prepare({ ...f.cli, [option]: f.root }), /retired/);
  }
  for (const option of ['pm3-node', 'pm3-home']) {
    assert.throws(() => prepare({ ...f.cli, [option]: f.root }), /requires both/);
  }
  assert.deepEqual(fs.readdirSync(f.outputDirectory), []);
});

test('failed lint cleans staging and permits a fresh retry without publishing failure', t => {
  const f = fixture(t);
  assert.throws(() => prepare(f.validatedCli, { execute: () => { throw { status: 1 }; } }), /lint did not pass/);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), []);
  prepare(f.validatedCli, { execute: () => 'lint passed' });
  assert.equal(fs.existsSync(f.destination), true);
});

for (const operation of ['openSync', 'writeFileSync', 'fsyncSync', 'linkSync']) {
  test(operation + ' failure leaves no partial config or unrelated file damage', t => {
    const f = fixture(t);
    const foreign = path.join(f.outputDirectory, 'keep');
    fs.writeFileSync(foreign, 'unrelated');
    const io = { ...fs, [operation]: () => { throw new Error('injected failure'); } };
    assert.throws(() => prepare(f.cli, { io }), /injected failure/);
    assert.deepEqual(fs.readdirSync(f.outputDirectory), ['keep']);
    assert.equal(fs.readFileSync(foreign, 'utf8'), 'unrelated');
    prepare(f.cli);
  });
}

for (const operation of ['unlinkSync', 'rmdirSync']) {
  test(operation + ' cleanup failure stays explicit after successful publication', t => {
    const f = fixture(t);
    const io = { ...fs, [operation]: () => { throw Object.assign(new Error('cleanup failed'), { code: 'EACCES' }); } };
    const result = prepare(f.cli, { io });
    assert.equal(result.publication, 'published');
    assert.ok(result.cleanupWarnings.some(w => w.operation === operation && w.code === 'EACCES'));
    assert.deepEqual(JSON.parse(fs.readFileSync(f.destination)), createConfiguration(f).config);
    assert.throws(() => prepare(f.cli), /already exists/);
  });
}

test('existing and racing outputs are preserved without overwriting', t => {
  const f = fixture(t);
  const io = { ...fs, linkSync(source, destination) {
    fs.writeFileSync(destination, 'racing writer', { flag: 'wx' });
    fs.linkSync(source, destination);
  } };
  assert.throws(() => prepare(f.cli, { io }), { code: 'EEXIST' });
  assert.equal(fs.readFileSync(f.destination, 'utf8'), 'racing writer');
  assert.throws(() => prepare(f.validatedCli, { execute: () => assert.fail('must not lint') }), /already exists/);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), ['pm3.yaml']);
});
