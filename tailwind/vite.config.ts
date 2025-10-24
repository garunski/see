import { defineConfig } from 'vite';
import { resolve } from 'path';
import { fileURLToPath, URL } from 'node:url';
import tailwindcss from 'tailwindcss';
import autoprefixer from 'autoprefixer';

export default defineConfig({
  build: {
    outDir: '../gui/assets',
    emptyOutDir: false, // Don't clear the entire assets directory
    rollupOptions: {
      input: {
        'tailwind': resolve(fileURLToPath(new URL('.', import.meta.url)), 'src/tailwind.css'),
      },
      output: {
        assetFileNames: (assetInfo) => {
          if (assetInfo.name === 'tailwind.css') {
            return 'tailwind.css';
          }
          return '[name].[ext]';
        }
      }
    }
  },
  css: {
    postcss: {
      plugins: [
        tailwindcss,
        autoprefixer,
      ],
    },
  },
});
