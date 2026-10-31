import { defineConfig } from 'vite';
export default defineConfig({
  root: 'web',
  clearScreen: false,
  build: { outDir: '../dist', emptyOutDir: true, target: 'es2022' },
  worker: { format: 'es' },
  server: { port: 1420, strictPort: true, host: '127.0.0.1' },
});
