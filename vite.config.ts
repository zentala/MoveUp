import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path, { resolve } from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    exclude: ["node_modules/**", ".claude/**", "tests/**", "src-tauri/**"],
    setupFiles: ["./src/test/setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      // CSS and JSON fixtures are assets, not code — v8 reports them as 0%
      // and drags every global percentage down for files that can never
      // carry a branch or a function.
      exclude: [
        "tests/**",
        "src-tauri/**",
        "node_modules/**",
        "**/*.css",
        "**/*.json",
      ],
      // Raised 2026-09-06 after adding tests for DebugSection, ShareStats and
      // AnalystLive (previously 0% or near-0%, per BACKLOG.md's "E018-T05
      // lowered coverage thresholds" follow-up). Measured on that run: lines
      // 89.57, statements 87.56, functions 81.08, branches 79.23 — back above
      // the original 80/80/75 target. Thresholds are set a few points below
      // the measured value, not equal to it, so ordinary test-count drift
      // doesn't fail the gate.
      thresholds: {
        lines: 85,
        functions: 78,
        branches: 76,
      },
    },
  },

  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        welcome: resolve(__dirname, "welcome.html"),
      },
    },
  },

  clearScreen: false,
  server: {
    port: 1443,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1444 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
    // The remote display in dev mode (E022-T12, closing the first "Dev-mode
    // remote display" backlog entry from E015-T05).
    //
    // `useRemoteDesk` builds its socket URL from `window.location.host`, so on
    // the Vite server it asks for `ws://localhost:1443/display/ws` — where
    // nothing answered, leaving the page stuck on "Reconnecting…". Proxying
    // `/display` to the desktop app's embedded axum server gives the real
    // React app the real data over the dev server, with no `tauri build`.
    //
    // `127.0.0.1`, not `localhost`: Node resolves `localhost` to `::1` on
    // Windows, and `remote_server.rs` binds IPv4 (rules/typescript.md).
    proxy: {
      "/display": {
        target: `http://127.0.0.1:${process.env.DESK_REMOTE_PORT ?? 3390}`,
        changeOrigin: true,
        ws: true,
      },
    },
  },
});
