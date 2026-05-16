import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  root: 'webview',
  base: './', // Ensures relative paths for VS Code
  build: {
    outDir: '../dist/webview',
    emptyOutDir: true
  }
});