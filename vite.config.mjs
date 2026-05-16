import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  root: 'webview', // Assuming your Svelte code is in a folder named 'webview'
  base: './',
  build: {
    outDir: '../dist/webview', // MUST have ../ so it goes to the root dist folder!
    emptyOutDir: true
  }
});