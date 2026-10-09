import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createConfiguration, EXPECT_JSON } from './pm3-config.mjs';
import { loadPm3Source, validateConfiguration } from './pm3-source-validation.mjs';
import { prepare } from './prepare-pm3.mjs';

function fixture(t) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'moveup-pm3-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const installRoot = path.join(root, 'installed app');
  const outputDirectory = path.join(root, 'isolated-home');
  mkdirSync(installRoot); mkdirSync(outputDirectory);
  const versions = ['0.6.1', '0.6.0'];
  for (const version of versions) {
    const dir = path.join(installRoot, 'releases', version);
    mkdirSync(dir, { recursive: true });
    writeFileSync(path.join(dir, 'desk.exe'), 'fixture, never executed');
  }
  const ordered = path.join(root, 'ordered-candidates');
  writeFileSync(ordered, versions.join('\n') + '\n');
  const options = { installRoot, outputDirectory, versions, port: 43391 };
  return { root, options, documents: createConfiguration(options), cli: {
    'install-root': installRoot, 'output-dir': outputDirectory,
    'ordered-candidates': ordered, port: '43391',
  } };
}

test('configuration pins exact readiness keys, identity and the same explicit port', t => {
  const { documents } = fixture(t);
  const service = documents.config.services[0];
  assert.deepEqual(service.health.expectJson, { app_id: 'io.zntl.desk', ready: true });
  assert.deepEqual(service.supervised.instance, { pidField: 'pid', runIdField: 'supervisor_run_id' });
  assert.equal(service.env.DESK_READINESS_PORT, '43391');
  assert.equal(new URL(service.health.url).port, service.env.DESK_READINESS_PORT);
  assert.equal(new URL(service.health.url).hostname, '127.0.0.1');
  assert.equal(new URL(service.health.url).pathname, '/display/readiness');
  assert.equal(service.supervised.missThreshold, 2);
  assert.equal(service.supervised.missIntervalMs, 60000);
  assert.equal(service.supervised.standDown, true);
  assert.equal(service.boot, false);
  assert.equal(service.cmd, undefined);
  assert.deepEqual(service.envFile, []);
  for (const candidate of documents.candidates.candidates) {
    assert.deepEqual(candidate.health.expectJson, { version: candidate.id });
    assert.ok(path.isAbsolute(candidate.argv[0]));
    assert.equal(candidate.argv[1], '--minimized');
  }
});

test('invalid ports, paths and candidate identities are refused', t => {
  const { options } = fixture(t);
  for (const port of [0, 65536, 3.5, NaN, '3391']) {
    assert.throws(() => createConfiguration({ ...options, port }), /integer/);
  }
  for (const versions of [[], ['../escape'], ['0.6', '0.6'], ['.'], ['a', 'b', 'c', 'd']]) {
    assert.throws(() => createConfiguration({ ...options, versions }), /candidate list/);
  }
  assert.throws(() => createConfiguration({ ...options, installRoot: 'relative' }), /absolute/);
});

test('missing source is actionable and writes no configuration', t => {
  const f = fixture(t);
  assert.throws(() => prepare({ ...f.cli, 'pm3-source': path.join(f.root, 'missing') }), /No CLI was started/);
  assert.equal(existsSync(path.join(f.options.outputDirectory, 'pm3.yaml')), false);
  assert.equal(existsSync(path.join(f.options.outputDirectory, 'candidates.json')), false);
});

test('a schema silently stripping supervision is refused', t => {
  const { documents } = fixture(t);
  const api = { Pm3ConfigSchema: { parse: config => {
    const result = structuredClone(config);
    delete result.services[0].supervised;
    return result;
  } } };
  assert.throws(() => validateConfiguration(api, documents), /discarded required fields/);
});

test('actual PM3 source accepts isolated configuration and rejects foreign promotion', {
  skip: !process.env.PM3_SOURCE && 'PENDING: PM3_SOURCE is required for owner-source validation',
}, t => {
  const f = fixture(t);
  const api = loadPm3Source(process.env.PM3_SOURCE);
  const parsed = validateConfiguration(api, f.documents);
  const service = parsed.services[0];
  const context = { deps: { config: () => service } };
  const candidate = api.parseCandidateDocument(f.documents.candidates).candidates[0];
  const expected = api.mergedExpectJson(EXPECT_JSON, candidate);
  const owned = { pid: 1234, runId: 'pm3-spawn' };
  const dev = { app_id: 'io.zntl.desk', ready: true, version: candidate.id,
    pid: 1234, run_id: 'app-diagnostic-uuid', sensor_present: false };
  assert.equal(api.firstMismatch(dev, service.health.expectJson), null);
  assert.equal(api.firstMismatch(dev, expected), null);
  assert.equal(api.isOwnedInstance(context, dev, owned), false);
  const child = { ...dev, supervisor_run_id: 'pm3-spawn' };
  assert.equal(api.isOwnedInstance(context, child, owned), true);
  assert.equal(api.isOwnedInstance(context, { ...child, pid: 999 }, owned), false);
  assert.equal(api.isOwnedInstance(context, { ...child, supervisor_run_id: 'another' }, owned), false);
  assert.equal(api.firstMismatch({ ...child, version: 'wrong' }, expected), 'version');
  assert.equal(api.firstMismatch({ ...child, app_id: 'wrong' }, expected), 'app_id');
  assert.equal(api.firstMismatch({ ...child, ready: 'true' }, expected), 'ready');
  const result = prepare(f.cli);
  assert.equal(result.registration, 'not-performed');
  assert.equal(result.integration, 'pending-pm3-owner-acceptance');
  const config = JSON.parse(readFileSync(result.files[1], 'utf8'));
  assert.equal(config.services[0].supervised.candidatesFile, result.files[0]);
  const before = readFileSync(result.files[1], 'utf8');
  assert.throws(() => prepare(f.cli), /already exist/);
  assert.equal(readFileSync(result.files[1], 'utf8'), before);
  t.diagnostic(`PM3 source hashes: ${JSON.stringify(result.sourceHashes)}`);
});
