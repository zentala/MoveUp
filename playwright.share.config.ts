import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  testMatch: 'share-popup.test.ts',
  workers: 1,
  retries: 0,
  reporter: 'list',
  outputDir: 'node_modules/.cache/share-popup',
  use: { baseURL: 'http://127.0.0.1:15431', browserName: 'chromium', screenshot: 'only-on-failure' },
  webServer: {
    command: 'pnpm exec vite --host 127.0.0.1 --port 15431 --strictPort',
    url: 'http://127.0.0.1:15431',
    reuseExistingServer: false,
    timeout: 30_000,
  },
});
