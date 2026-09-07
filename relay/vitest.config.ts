import path from "node:path";
import { fileURLToPath } from "node:url";

import { cloudflareTest, readD1Migrations } from "@cloudflare/vitest-pool-workers";
import { defineConfig } from "vitest/config";

const here = path.dirname(fileURLToPath(import.meta.url));

/**
 * Tests run inside workerd through the Workers Vitest integration, so the
 * Durable Object under test is the real one, not a hand-written stand-in, and
 * the credential store is a real D1 database with the real migrations applied.
 *
 * Miniflare options are declared here rather than read from `wrangler.toml`.
 * The Worker's `[assets]` block points at `../dist`, which only exists after a
 * `vite build`; loading the wrangler config in tests would make every relay
 * test depend on a frontend build that has nothing to do with the relay. The
 * bindings below mirror `wrangler.toml` — change the two together.
 */
const migrations = await readD1Migrations(path.join(here, "migrations"));

export default defineConfig({
  plugins: [
    cloudflareTest({
      main: "src/index.ts",
      miniflare: {
        compatibilityDate: "2026-08-22",
        compatibilityFlags: ["nodejs_compat"],
        durableObjects: {
          DESK_ROOM: { className: "DeskRoom", useSQLite: true },
          RATE_LIMITER: { className: "RateLimiter", useSQLite: true },
        },
        d1Databases: { DB: "moveup-relay-test" },
        bindings: {
          RELAY_VERSION: "test",
          // Short enough that the timeout tests finish well inside a second,
          // long enough that a slow machine does not trip them.
          HELLO_TIMEOUT_MS: "250",
          IDLE_TIMEOUT_MS: "700",
          // Pairing timing is asserted in tests, so the values are small and
          // exact rather than the five and fifteen minutes of production.
          PAIRING_TTL_MS: "1000",
          PAIRING_LOCKOUT_MS: "2000",
          PAIRING_MAX_ATTEMPTS: "10",
          TEST_MIGRATIONS: migrations,
        },
      },
    }),
  ],
  test: {
    setupFiles: ["./test/apply-migrations.ts"],
  },
  resolve: {
    alias: {
      // The protocol schema lives once, in the app (E022-T01), and is imported
      // by the relay through this alias — mirrored in `tsconfig.json` paths.
      "@app": path.resolve(here, "../src"),
    },
  },
});
