/**
 * Applies `migrations/*.sql` to the test D1 database before each test file.
 *
 * The schema under test is therefore the schema that ships: a column renamed in
 * `0001_init.sql` and not in the code fails here, rather than in production.
 */
import { applyD1Migrations, env } from "cloudflare:test";

import type { TestEnv } from "./env";

const testEnv = env as TestEnv;
if (!testEnv.DB) throw new Error("test setup: no DB binding — check vitest.config.ts");
await applyD1Migrations(testEnv.DB, testEnv.TEST_MIGRATIONS);
