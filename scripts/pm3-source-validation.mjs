// Read-only source validation. Never imports PM3's CLI, loader or daemon.
import { readFileSync, existsSync } from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { compileFunction } from 'node:vm';
import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import ts from 'typescript';
import { absolutePath } from './pm3-config.mjs';

const REQUIRED = 'PM3 supervised candidate-file v1 with typed health, PID/run binding and independent suspension holds';

export function loadPm3Source(root) {
  try {
    root = absolutePath(root, 'PM3_SOURCE');
    const cache = new Map();
    const hashes = {};
    function load(relative) {
      const file = path.resolve(root, relative);
      if (cache.has(file)) return cache.get(file).exports;
      const source = readFileSync(file, 'utf8');
      hashes[path.relative(root, file).replaceAll('\\', '/')] = createHash('sha256').update(source).digest('hex');
      const output = ts.transpileModule(source, {
        compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022, esModuleInterop: true },
        fileName: file,
      }).outputText;
      const module = { exports: {} };
      cache.set(file, module);
      const require = createRequire(file);
      const localRequire = specifier => {
        // PM3's workspace package has import-only published exports. Read its
        // source too, rather than trusting an unrelated/stale built package.
        if (specifier === '@internal/runtime-release') return load('packages/runtime-release/src/index.ts');
        const sourceFile = path.resolve(path.dirname(file), specifier.replace(/\.js$/, '.ts'));
        return specifier.startsWith('.') && existsSync(sourceFile) ? load(sourceFile) : require(specifier);
      };
      compileFunction(output, ['exports', 'require', 'module', '__filename', '__dirname'], { filename: file })(
        module.exports, localRequire, module, file, path.dirname(file));
      return module.exports;
    }
    const schema = load('src/config-schema.ts');
    const candidates = load('src/supervised/candidate-file.ts');
    const health = load('src/health-json.ts');
    const state = load('src/supervised/controller-state.ts');
    if (!schema.Pm3ConfigSchema || !candidates.parseCandidateDocument || !state.isOwnedInstance) {
      throw new Error('required source exports are absent');
    }
    return { ...schema, ...candidates, ...health, ...state, hashes };
  } catch (error) {
    throw new Error(`Missing usable ${REQUIRED}. Supply PM3_SOURCE pointing to the owner-provided source with installed dependencies. No CLI was started. Cause: ${error.message}`);
  }
}

export function validateConfiguration(api, { config, candidates }) {
  const parsed = api.Pm3ConfigSchema.parse(config);
  const service = parsed.services[0];
  const original = config.services[0];
  if (!isDeepStrictEqual(service.supervised, original.supervised)
      || !isDeepStrictEqual(service.health.expectJson, original.health.expectJson)
      || service.health.maxBodyBytes !== 4096) {
    throw new Error(`PM3 schema discarded required fields; upgrade to ${REQUIRED}. Do not register this configuration.`);
  }
  const result = api.parseCandidateDocument(candidates);
  if (result.kind !== 'ok') throw new Error(`PM3 rejected candidate file: ${result.kind}: ${result.detail ?? ''}`);
  return parsed;
}
