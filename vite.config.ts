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
      exclude: ["tests/**", "src-tauri/**", "node_modules/**"],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 75,
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
  },
});
