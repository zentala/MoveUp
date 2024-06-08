import { defineConfig } from 'electron-vite';
import react from '@vitejs/plugin-react-swc';
import { resolve } from 'path';

export default defineConfig({
  main: {
    build: {
      lib: {
        entry: 'src/main/app.ts',
      },
      outDir: 'dist/main',
    },
  },
  preload: {
    build: {
      lib: {
        entry: 'src/renderer/preload.ts',
      },
      outDir: 'dist/preload',
    },
    onstart: (options) => {
      options.reload();
    },
  },
  renderer: {
    build: {
      rollupOptions: {
        input: 'src/renderer/index.html',
      },
      outDir: 'dist/renderer',
    },
    plugins: [react()],
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
      },
    },
  },
});
