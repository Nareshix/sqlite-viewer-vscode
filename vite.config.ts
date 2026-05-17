import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import monacoEditorPlugin from 'vite-plugin-monaco-editor';

export default defineConfig({
  plugins: [
    svelte(),
    monacoEditorPlugin({ languageWorkers: [] })
  ],
  root: 'webview',
  base: './',
  build: {
    outDir: '../dist/webview',
    emptyOutDir: true,
    chunkSizeWarningLimit: 3000
  }
});