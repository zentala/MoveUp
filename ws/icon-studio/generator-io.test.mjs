import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { families } from './manifest.mjs';
import { writeIconSet } from './generator-io.mjs';

const svg = (name) => `<svg aria-label="${name}"></svg>`;

test('preview inventory matches generated SVGs and opens through a classic file script', async (t) => {
  const root = await mkdtemp(join(tmpdir(), 'moveup-icon-studio-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeIconSet(root, families, Object.fromEntries(families.map(({ id }) => [id, svg])));

  const browser = { window: {} };
  runInNewContext(await readFile(join(root, 'preview-manifest.js'), 'utf8'), browser);
  assert.deepEqual(JSON.parse(JSON.stringify(browser.window.iconStudioFamilies)), families);
  const preview = await readFile(new URL('./preview.html', import.meta.url), 'utf8');
  assert.match(preview, /<script src="preview-manifest\.js"><\/script>/);
  assert.match(preview, /window\.iconStudioFamilies/);

  for (const family of families) {
    const actual = (await readdir(join(root, 'icons', family.id))).sort();
    assert.deepEqual(actual, family.names.map((name) => `${name}.svg`).sort());
  }
});

test('renamed icons remove only stale direct SVG files in the managed family', async (t) => {
  const root = await mkdtemp(join(tmpdir(), 'moveup-icon-studio-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const family = (names) => [{ id: 'test-family', names, title: 'Test', sizes: [24], themes: ['light'] }];
  await writeIconSet(root, family(['old-name']), { 'test-family': svg });
  const familyDir = join(root, 'icons', 'test-family');
  await writeFile(join(familyDir, 'notes.txt'), 'keep', 'utf8');
  await mkdir(join(familyDir, 'drafts'));
  await writeFile(join(familyDir, 'drafts', 'sketch.svg'), 'keep', 'utf8');
  await mkdir(join(root, 'icons', 'other-family'));
  await writeFile(join(root, 'icons', 'other-family', 'unrelated.svg'), 'keep', 'utf8');

  await writeIconSet(root, family(['new-name']), { 'test-family': svg });
  assert.deepEqual((await readdir(familyDir)).sort(), ['drafts', 'new-name.svg', 'notes.txt']);
  assert.equal(await readFile(join(familyDir, 'drafts', 'sketch.svg'), 'utf8'), 'keep');
  assert.equal(await readFile(join(root, 'icons', 'other-family', 'unrelated.svg'), 'utf8'), 'keep');
});

test('bad manifest or renderer output fails before touching generated files', async (t) => {
  const root = await mkdtemp(join(tmpdir(), 'moveup-icon-studio-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const valid = [{ id: 'test-family', names: ['kept'], title: 'Test', sizes: [24], themes: ['light'] }];
  await writeIconSet(root, valid, { 'test-family': svg });
  const before = await readFile(join(root, 'icons', 'test-family', 'kept.svg'), 'utf8');

  await assert.rejects(writeIconSet(root, [{ ...valid[0], names: ['kept', '../escape'] }], { 'test-family': svg }), /Invalid or duplicate icon name/);
  await assert.rejects(writeIconSet(root, [{ ...valid[0], names: ['kept', 'broken'] }], { 'test-family': (name) => name === 'broken' ? '' : svg(name) }), /invalid SVG/);
  assert.equal(await readFile(join(root, 'icons', 'test-family', 'kept.svg'), 'utf8'), before);
  assert.deepEqual(await readdir(join(root, 'icons', 'test-family')), ['kept.svg']);
});
