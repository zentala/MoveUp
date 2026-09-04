import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import electron from 'vite-plugin-electron';
import path from 'node:path';

const projectRoot = process.cwd();

export default defineConfig({
  root: 'src/renderer',
  plugins: [
    react(),
    electron([
      {
        entry: path.resolve(projectRoot, 'src/main/app.ts'),
        vite: {
          build: {
            outDir: path.resolve(projectRoot, 'dist-electron'),
          },
        },
      },
      {
        entry: path.resolve(projectRoot, 'src/renderer/preload.ts'),
        onstart: (options) => {
          options.reload();
        },
        vite: {
          build: {
            outDir: path.resolve(projectRoot, 'dist-electron'),
          },
        },
      },
    ]),
  ],
  build: {
    outDir: path.resolve(projectRoot, 'dist/renderer'),
    emptyOutDir: false,
  },
});
