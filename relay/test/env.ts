/**
 * The bindings a test sees. `cloudflare:test`'s `env` is typed as the module
 * augmentation `ProvidedEnv`, which this project does not declare; casting to
 * this interface keeps the extra test-only binding visible without pretending
 * the Worker itself has one.
 */
import type { D1Migration } from "@cloudflare/vitest-pool-workers";

import type { Env } from "../src/env";

export interface TestEnv extends Env {
  TEST_MIGRATIONS: D1Migration[];
}
