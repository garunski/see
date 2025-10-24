import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: '/assets/workflow-visualizer/',
  build: {
    outDir: '../gui/assets/workflow-visualizer',
    emptyOutDir: true,
    assetsDir: '.', // Don't nest assets in subdirectory
    rollupOptions: {
      output: {
        entryFileNames: 'index.js',
        chunkFileNames: '[name].js',
        assetFileNames: '[name].[ext]',
      },
    },
  },
});

