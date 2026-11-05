import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

// This opt-in build never reads or replaces product dist/ and starts no server.
export default defineConfig({
  root: resolve(__dirname, "tests/native"),
  plugins: [react()],
  resolve: { alias: { "@": resolve(__dirname, "src") } },
  build: {
    outDir: resolve(__dirname, "node_modules/.cache/reminder-native-dist"),
    emptyOutDir: true,
    rollupOptions: { input: resolve(__dirname, "tests/native/reminder-native.html") },
  },
});
