/**
 * vitest.perf.config.ts
 *
 * Configuration for performance tests (memory profiling, etc)
 * Uses node environment without setup files.
 *
 * Usage:
 *   vitest run --config vitest.perf.config.ts tests/perf
 */

import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    globals: true,
    include: ['tests/perf/**/*.test.ts'],
    // No setup files - perf tests are isolated
  },
});
