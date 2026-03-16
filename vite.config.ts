import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

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
    setupFiles: ["./src/test/setup.ts"],
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
