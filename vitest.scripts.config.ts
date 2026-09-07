/**
 * vitest.scripts.config.ts
 *
 * Configuration for script unit tests (build-report.js, etc)
 * Uses node environment without setup files.
 *
 * Usage:
 *   vitest run --config vitest.scripts.config.ts tests/scripts
 */

import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    globals: true,
    include: ['tests/scripts/**/*.test.{js,ts}'],
    // No setup files - scripts are isolated unit tests
  },
});
