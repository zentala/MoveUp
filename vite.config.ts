import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import electron from 'vite-plugin-electron';

export default defineConfig({
  plugins: [
    react(),
    electron([
      {
        entry: 'src/main/app.ts',
        vite: {
          build: {
            outDir: 'dist/main',
          },
        },
      },
      {
        entry: 'src/renderer/preload.ts',
        onstart: (options) => {
          options.reload();
        },
        vite: {
          build: {
            outDir: 'dist/preload',
          },
        },
      },
    ]),
  ],
  build: {
    outDir: 'dist',
  },
});
