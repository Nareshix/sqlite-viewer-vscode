// E.g., src/extension.ts (Your VS Code backend)
import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { performance } from 'perf_hooks';
import { Database } from '../backend';

let activePanel: vscode.WebviewPanel | null = null;
let statusBarItem: vscode.StatusBarItem;

class SQLiteEditorProvider implements vscode.CustomReadonlyEditorProvider {
  constructor(private readonly context: vscode.ExtensionContext) {}

  openCustomDocument(uri: vscode.Uri): vscode.CustomDocument {
    return { uri, dispose: () => {} };
  }

  resolveCustomEditor(
    document: vscode.CustomDocument,
    webviewPanel: vscode.WebviewPanel,
  ): void {
    const webviewDir = vscode.Uri.file(
      path.join(this.context.extensionPath, 'dist', 'webview')
    );

    webviewPanel.webview.options = {
      enableScripts: true,
      localResourceRoots: [webviewDir],
    };

    const htmlPath = path.join(webviewDir.fsPath, 'index.html');
    let html = fs.readFileSync(htmlPath, 'utf8');
    const webviewUri = webviewPanel.webview.asWebviewUri(webviewDir);
    html = html.replace(/(<head.*?>)/i, `$1\n    <base href="${webviewUri}/">`);
    webviewPanel.webview.html = html;

    activePanel = webviewPanel;

    // Status Bar visibility logic
    webviewPanel.onDidChangeViewState(e => {
      if (e.webviewPanel.active) {
        activePanel = webviewPanel;
        if (statusBarItem.text) statusBarItem.show();
      } else {
        if (activePanel === webviewPanel) {
          activePanel = null;
          statusBarItem.hide();
        }
      }
    });

    webviewPanel.onDidDispose(() => {
      if (activePanel === webviewPanel) {
        activePanel = null;
        statusBarItem.hide();
      }
    });

    try {
      const db = new Database(document.uri.fsPath);

      // NOTICE: We made this callback async!
      webviewPanel.webview.onDidReceiveMessage(async (message) => {
        if (message.command === 'ready') {
          const saved = this.context.workspaceState.get(document.uri.toString());
          webviewPanel.webview.postMessage({ command: 'restoreState', state: saved });
        }

        if (message.command === 'updateStatus') {
          if (message.text) {
            statusBarItem.text = message.text;
            if (activePanel === webviewPanel) statusBarItem.show();
          } else {
            statusBarItem.hide();
          }
        }

        if (message.command === 'runSql') {
          const start = performance.now();
          try {
            // PROPER ASYNC EXECUTION!
            // Yields to the Node.js event loop while Rust works on a background thread.
            const jsonResult = await db.query(message.text);

            const data = JSON.parse(jsonResult);
            const timeMs = performance.now() - start;
            webviewPanel.webview.postMessage({ command: 'sqlResult', data, timeMs });
          } catch (err: any) {
            webviewPanel.webview.postMessage({ command: 'sqlError', error: err.message });
          }
        }

        if (message.command === 'saveState') {
          this.context.workspaceState.update(document.uri.toString(), message.state);
        }

        if (message.command === 'getSchema') {
          try {
            // Async schema load!
            const jsonResult = await db.schema();
            const schema = JSON.parse(jsonResult);
            webviewPanel.webview.postMessage({ command: 'schemaResult', schema });
          } catch (err: any) {
            webviewPanel.webview.postMessage({ command: 'sqlError', error: err.message });
          }
        }
      });
    } catch (err: any) {
      vscode.window.showErrorMessage(`Failed to open database: ${err.message}`);
    }
  }
}

export function activate(context: vscode.ExtensionContext) {
  statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  context.subscriptions.push(statusBarItem);

  const provider = new SQLiteEditorProvider(context);

  context.subscriptions.push(
    vscode.commands.registerCommand('monaco-hello.runQuery', () => {
      activePanel?.webview.postMessage({ command: 'runQuery' });
    })
  );

  context.subscriptions.push(
    vscode.window.registerCustomEditorProvider(
      'monaco-hello.sqliteEditor',
      provider,
      { webviewOptions: { retainContextWhenHidden: true } }
    )
  );
}

export function deactivate() {}