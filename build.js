const esbuild = require('esbuild');
const fs = require('fs-extra');
const path = require('path');

// Copy Monaco's prebuilt min/vs folder into media/
fs.copySync(
  path.join(__dirname, 'node_modules/monaco-editor/min/vs'),
  path.join(__dirname, 'media/vs')
);

// Bundle your extension code
esbuild.build({
  entryPoints: ['src/extension.ts'],
  bundle: true,
  outfile: 'dist/extension.js',
  external: ['vscode'],   // vscode is always external
  format: 'cjs',
  platform: 'node',
  sourcemap: true,
}).catch(() => process.exit(1));