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
  const outputDirectory = path.join(root, 'isolated-home');
  fs.mkdirSync(installRoot); fs.mkdirSync(outputDirectory);
  const candidatesFile = path.join(installRoot, 'app-owned.json');
  // Deliberately non-inferred layout, order and whitespace. Never execute artifacts.
  const candidates = { version: 1, candidates: ['0.6.1', '0.6.0'].map(id => ({
    id, argv: [path.join(installRoot, 'custom artifacts', id, 'desk.exe'), '--minimized'],
    health: { expectJson: { version: id } },
  })) };
  const bytes = JSON.stringify(candidates, null, 4) + '\r\n';
  fs.writeFileSync(candidatesFile, bytes);
  const options = { installRoot, candidatesFile, port: 43391 };
  const cli = { 'install-root': installRoot, 'output-dir': outputDirectory,
    'candidates-file': candidatesFile, port: '43391' };
  const validatedCli = { ...cli, 'pm3-node': process.execPath, 'pm3-home': path.join(root, 'synthetic pm3') };
  const report = { ok: true, version: 1, problems: [], candidates: candidates.candidates.map(c => ({
    id: c.id, argv0: c.argv[0], artifact: null,
    expectJson: { app_id: 'io.zntl.desk', ready: true, version: c.id },
  })) };
  return { root, options, cli, validatedCli, report, bytes, outputDirectory, candidatesFile,
    destination: path.join(outputDirectory, 'pm3.yaml') };
}

test('configuration preserves readiness, identity, port and opt-in supervision', t => {
  const f = fixture(t);
  const { config } = createConfiguration(f.options);
  const service = config.services[0];
  assert.equal(config.name, 'moveup');
  assert.equal(config.nodeAutoInstall, false);
  assert.equal(service.name, 'desk');
  assert.deepEqual(service.health.expectJson, { app_id: 'io.zntl.desk', ready: true });
  assert.deepEqual(service.supervised.instance, { pidField: 'pid', runIdField: 'supervisor_run_id' });
  assert.equal(service.env.DESK_READINESS_PORT, '43391');
  assert.equal(service.health.port, 43391);
  assert.equal(new URL(service.health.url).port, service.env.DESK_READINESS_PORT);
  assert.equal(new URL(service.health.url).hostname, '127.0.0.1');
  assert.equal(new URL(service.health.url).pathname, '/display/readiness');
  assert.equal(service.supervised.missThreshold, 2);
  assert.equal(service.supervised.missIntervalMs, 60000);
  assert.equal(service.supervised.standDown, true);
  assert.equal(service.boot, false);
  assert.equal(service.cwd, f.options.installRoot);
  assert.equal(service.cmd, undefined);
  assert.deepEqual(service.envFile, []);
  assert.equal(service.supervised.promote.callback, undefined);
  assert.equal(service.supervised.candidatesFile, f.candidatesFile);
});

test('installed configuration pins structured fallback callback and supervision environment', t => {
  const f = fixture(t);
  const callbackFile = path.join(f.outputDirectory, 'release-callback.mjs');
  const runDirectory = path.join(f.root, 'app run evidence');
  fs.mkdirSync(runDirectory);
  const { config } = createConfiguration({ ...f.options, callbackFile, runDirectory,
    nodeExecutable: process.execPath, pm3Home: path.join(f.root, 'pm3 home') });
  const service = config.services[0];
  assert.deepEqual(service.supervised.promote.callback,
    [process.execPath, callbackFile, path.dirname(f.candidatesFile), runDirectory, path.join(f.root, 'pm3 home'), '43391']);
  assert.equal(service.env.DESK_SUPERVISION, 'pm3');
  assert.equal(service.env.DESK_PM3_NODE, process.execPath);
  assert.equal(service.env.DESK_PM3_SERVICE, 'moveup/desk');
  assert.equal(service.supervised.promote.afterMs, 30000);
  assert.equal(service.supervised.promote.callbackTimeoutMs, 10000);
  assert.throws(() => createConfiguration({ ...f.options, runDirectory }), /requires/);
  const result = prepare({ ...f.validatedCli, 'run-directory': runDirectory }, {
    execute: () => JSON.stringify(f.report),
  });
  assert.deepEqual(result.files, [f.destination, callbackFile]);
  assert.match(fs.readFileSync(callbackFile, 'utf8'), /PM3_CANDIDATE_ID/);
  const published = JSON.parse(fs.readFileSync(f.destination));
  assert.deepEqual(published.services[0].supervised.promote.callback,
    [process.execPath, callbackFile, path.dirname(f.candidatesFile), runDirectory, f.validatedCli['pm3-home'], '43391']);
});

test('callback publication is rolled back if final configuration publication fails', t => {
  const f = fixture(t);
  const runDirectory = path.join(f.root, 'run evidence'); fs.mkdirSync(runDirectory);
  const callbackFile = path.join(f.outputDirectory, 'release-callback.mjs');
  const io = { ...fs, linkSync(source, destination) {
    if (destination === f.destination) throw new Error('config publication failed');
    fs.linkSync(source, destination);
  } };
  assert.throws(() => prepare({ ...f.validatedCli, 'run-directory': runDirectory }, {
    io, execute: () => JSON.stringify(f.report),
  }), /config publication failed/);
  assert.equal(fs.existsSync(callbackFile), false);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), []);
  assert.equal(fs.readFileSync(f.candidatesFile, 'utf8'), f.bytes);
});

test('invalid ports and relative paths are refused', t => {
  const f = fixture(t);
  for (const port of [0, 65536, 3.5, NaN, '3391']) {
    assert.throws(() => createConfiguration({ ...f.options, port }), /integer/);
  }
  for (const key of ['installRoot', 'candidatesFile']) {
    assert.throws(() => createConfiguration({ ...f.options, [key]: 'relative' }), /absolute/);
  }
  for (const port of ['bad', '3.5', '65536']) assert.throws(() => prepare({ ...f.cli, port }), /integer/);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), []);
});

test('draft requires no PM3 dependency, preserves input bytes and points to future app updates', t => {
  const f = fixture(t);
  const result = prepare(f.cli, { execute: () => assert.fail('draft must never launch a process') });
  assert.equal(result.validation.status, 'pending-not-validated');
  assert.equal(result.readyToInstall, false);
  assert.equal(result.integration, 'pending-pm3-owner-acceptance');
  assert.equal(result.registration, 'not-performed');
  assert.deepEqual(result.files, [f.destination]);
  assert.equal(result.publication, 'published');
  assert.deepEqual(result.cleanupWarnings, []);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), ['pm3.yaml']);
  assert.equal(fs.readFileSync(f.candidatesFile, 'utf8'), f.bytes);
  const configured = JSON.parse(fs.readFileSync(f.destination)).services[0].supervised.candidatesFile;
  assert.equal(configured, f.candidatesFile);
  // Simulate the app's atomic publication; the config must follow the same pathname.
  const replacement = path.join(f.options.installRoot, 'next.json');
  fs.writeFileSync(replacement, '{"version":1,"candidates":[]}');
  fs.renameSync(replacement, f.candidatesFile);
  assert.equal(fs.readFileSync(configured, 'utf8'), '{"version":1,"candidates":[]}');
});

test('missing, non-JSON, directory and legacy candidates refuse without output', t => {
  const f = fixture(t);
  assert.throws(() => prepare({ ...f.cli, 'candidates-file': path.join(f.root, 'missing') }), /ENOENT/);
  assert.throws(() => prepare({ ...f.cli, 'candidates-file': f.root }), /app-owned JSON/);
  assert.throws(() => prepare({ ...f.cli, 'ordered-candidates': f.candidatesFile }), /no longer supported/);
  fs.writeFileSync(f.candidatesFile, '0.6.1\n0.6.0');
  assert.throws(() => prepare(f.cli), SyntaxError);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), []);
});

test('incomplete validator selection refuses before writing', t => {
  const f = fixture(t);
  for (const key of ['pm3-node', 'pm3-home']) {
    assert.throws(() => prepare({ ...f.cli, [key]: f.root }), /requires both/);
  }
  assert.deepEqual(fs.readdirSync(f.outputDirectory), []);
});

test('synthetic offline validation receives the complete staged config and original candidate path', t => {
  const f = fixture(t);
  let calls = 0;
  const result = prepare(f.validatedCli, { execute: (executable, argv) => {
    calls++;
    assert.equal(executable, process.execPath);
    assert.equal(argv[3], f.candidatesFile);
    assert.equal(fs.existsSync(f.destination), false);
    const config = JSON.parse(fs.readFileSync(argv[5]));
    assert.equal(config.services[0].supervised.candidatesFile, f.candidatesFile);
    return JSON.stringify(f.report);
  } });
  assert.equal(calls, 1);
  assert.equal(result.validation.status, 'offline-valid');
  assert.equal(result.readyToInstall, false);
  assert.equal(result.integration, 'pending-pm3-owner-acceptance');
  assert.equal(fs.readFileSync(f.candidatesFile, 'utf8'), f.bytes);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), ['pm3.yaml']);
});

test('failed validator cleans staged files and permits retry without losing candidate bytes', t => {
  const f = fixture(t);
  assert.throws(() => prepare(f.validatedCli, { execute: () => { throw { status: 2 }; } }), /usage-or-unsupported/);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), []);
  assert.equal(fs.readFileSync(f.candidatesFile, 'utf8'), f.bytes);
  prepare(f.validatedCli, { execute: () => JSON.stringify(f.report) });
  assert.deepEqual(fs.readdirSync(f.outputDirectory), ['pm3.yaml']);
});

for (const operation of ['openSync', 'writeFileSync', 'fsyncSync', 'linkSync']) {
  test(`injected ${operation} failure leaves no partial config and retry succeeds`, t => {
    const f = fixture(t);
    const foreign = path.join(f.outputDirectory, '.moveup-pm3-foreign');
    fs.mkdirSync(foreign);
    fs.writeFileSync(path.join(foreign, 'keep'), 'foreign');
    const io = { ...fs, [operation]: (...args) => {
      if (operation === 'writeFileSync') fs.writeFileSync(args[0], '{"partial":');
      throw new Error('injected publication failure');
    } };
    assert.throws(() => prepare(f.cli, { io }), /injected publication failure/);
    assert.equal(fs.existsSync(f.destination), false);
    assert.deepEqual(fs.readdirSync(f.outputDirectory), ['.moveup-pm3-foreign']);
    assert.equal(fs.readFileSync(path.join(foreign, 'keep'), 'utf8'), 'foreign');
    assert.equal(fs.readFileSync(f.candidatesFile, 'utf8'), f.bytes);
    prepare(f.cli);
    assert.equal(JSON.parse(fs.readFileSync(f.destination)).name, 'moveup');
  });
}

for (const operation of ['unlinkSync', 'rmdirSync']) {
  function cleanupFailure(f) {
    const foreign = path.join(f.outputDirectory, '.moveup-pm3-foreign');
    fs.mkdirSync(foreign);
    fs.writeFileSync(path.join(foreign, 'keep'), 'foreign');
    const calls = [];
    const io = { ...fs };
    for (const cleanup of ['unlinkSync', 'rmdirSync']) {
      io[cleanup] = target => {
        calls.push({ operation: cleanup, path: target });
        assert.notEqual(target, f.destination);
        assert.notEqual(target, foreign);
        if (cleanup === operation) throw Object.assign(new Error(`injected ${operation} failure`), { code: 'EACCES' });
        return fs[cleanup](target);
      };
    }
    return { io, check(warnings) {
      assert.deepEqual(calls.map(c => c.operation), ['unlinkSync', 'rmdirSync']);
      const injected = warnings.find(w => w.operation === operation);
      assert.equal(injected.code, 'EACCES');
      assert.equal(injected.message, `injected ${operation} failure`);
      assert.equal(injected.path, calls.find(c => c.operation === operation).path);
      assert.equal(warnings.length, operation === 'unlinkSync' ? 2 : 1);
      assert.equal(fs.existsSync(path.dirname(calls[0].path)), true);
      assert.equal(fs.existsSync(calls[0].path), operation === 'unlinkSync');
      assert.equal(fs.readFileSync(path.join(foreign, 'keep'), 'utf8'), 'foreign');
      assert.equal(fs.readFileSync(f.candidatesFile, 'utf8'), f.bytes);
    } };
  }

  test(`${operation} cleanup failure reports published output with warnings and refuses replacement`, t => {
    const f = fixture(t);
    const cleanup = cleanupFailure(f);
    const result = prepare(f.cli, { io: cleanup.io });
    assert.equal(result.publication, 'published');
    assert.deepEqual(result.files, [f.destination]);
    assert.equal(result.validation.status, 'pending-not-validated');
    assert.equal(result.readyToInstall, false);
    cleanup.check(result.cleanupWarnings);
    const published = fs.readFileSync(f.destination, 'utf8');
    assert.deepEqual(JSON.parse(published), createConfiguration(f.options).config);
    assert.throws(() => prepare(f.cli), /already exists/);
    assert.equal(fs.readFileSync(f.destination, 'utf8'), published);
  });

  test(`${operation} cleanup failure preserves primary publication refusal and foreign output`, t => {
    const f = fixture(t);
    const cleanup = cleanupFailure(f);
    let primary;
    cleanup.io.linkSync = (source, destination) => {
      fs.writeFileSync(destination, 'racing writer', { flag: 'wx' });
      try { fs.linkSync(source, destination); } catch (error) { primary = error; throw error; }
    };
    assert.throws(() => prepare(f.cli, { io: cleanup.io }), error => {
      assert.equal(error, primary);
      assert.equal(error.code, 'EEXIST');
      cleanup.check(error.cleanupWarnings);
      return true;
    });
    assert.equal(fs.readFileSync(f.destination, 'utf8'), 'racing writer');
  });

  test(`${operation} cleanup failure preserves offline validation refusal without publication`, t => {
    const f = fixture(t);
    const cleanup = cleanupFailure(f);
    assert.throws(() => prepare(f.validatedCli, { io: cleanup.io,
      execute: () => { throw { status: 2 }; } }), error => {
      assert.match(error.message, /Offline PM3 validation did not pass \(usage-or-unsupported\)/);
      cleanup.check(error.cleanupWarnings);
      return true;
    });
    assert.equal(fs.existsSync(f.destination), false);
  });
}

test('existing output is preserved byte-for-byte without invoking the validator', t => {
  const f = fixture(t);
  fs.writeFileSync(f.destination, 'existing configuration');
  assert.throws(() => prepare(f.validatedCli, { execute: () => assert.fail('must not invoke') }), /already exists/);
  assert.equal(fs.readFileSync(f.destination, 'utf8'), 'existing configuration');
  assert.equal(fs.readFileSync(f.candidatesFile, 'utf8'), f.bytes);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), ['pm3.yaml']);
});

test('racing writer wins without overwrite or cleanup of its output', t => {
  const f = fixture(t);
  const io = { ...fs, linkSync: (source, destination) => {
    assert.equal(fs.existsSync(destination), false);
    assert.equal(JSON.parse(fs.readFileSync(source)).name, 'moveup');
    fs.writeFileSync(destination, 'racing writer', { flag: 'wx' });
    fs.linkSync(source, destination);
  } };
  assert.throws(() => prepare(f.cli, { io }), { code: 'EEXIST' });
  assert.equal(fs.readFileSync(f.destination, 'utf8'), 'racing writer');
  assert.equal(fs.readFileSync(f.candidatesFile, 'utf8'), f.bytes);
  assert.deepEqual(fs.readdirSync(f.outputDirectory), ['pm3.yaml']);
});
