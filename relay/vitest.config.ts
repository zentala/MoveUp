import path from "node:path";
import { fileURLToPath } from "node:url";

import { cloudflareTest } from "@cloudflare/vitest-pool-workers";
import { defineConfig } from "vitest/config";

const here = path.dirname(fileURLToPath(import.meta.url));

/**
 * Tests run inside workerd through the Workers Vitest integration, so the
 * Durable Object under test is the real one, not a hand-written stand-in.
 *
 * Miniflare options are declared here rather than read from `wrangler.toml`.
 * The Worker's `[assets]` block points at `../dist`, which only exists after a
 * `vite build`; loading the wrangler config in tests would make every relay
 * test depend on a frontend build that has nothing to do with the relay. The
 * bindings below mirror `wrangler.toml` — change the two together.
 */
export default defineConfig({
  plugins: [
    cloudflareTest({
      main: "src/index.ts",
      miniflare: {
        compatibilityDate: "2026-08-22",
        compatibilityFlags: ["nodejs_compat"],
        durableObjects: {
          DESK_ROOM: { className: "DeskRoom", useSQLite: true },
        },
        bindings: {
          RELAY_VERSION: "test",
          // Short enough that the timeout tests finish well inside a second,
          // long enough that a slow machine does not trip them.
          HELLO_TIMEOUT_MS: "250",
          IDLE_TIMEOUT_MS: "700",
        },
      },
    }),
  ],
  resolve: {
    alias: {
      // The protocol schema lives once, in the app (E022-T01), and is imported
      // by the relay through this alias — mirrored in `tsconfig.json` paths.
      "@app": path.resolve(here, "../src"),
    },
  },
});
