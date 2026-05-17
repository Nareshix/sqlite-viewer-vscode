import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { Database } from '../backend';

let activePanel: vscode.WebviewPanel | null = null;

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
    webviewPanel.onDidDispose(() => { activePanel = null; });
    webviewPanel.onDidChangeViewState(e => { if (e.webviewPanel.active) activePanel = webviewPanel; });

    try {
      const db = new Database(document.uri.fsPath);

      webviewPanel.webview.onDidReceiveMessage(message => {
        // Handle initialization request
        if (message.command === 'ready') {
          const saved = this.context.workspaceState.get(document.uri.toString());
          webviewPanel.webview.postMessage({ command: 'restoreState', state: saved });
        }

        if (message.command === 'runSql') {
          try {
            const jsonResult = db.query(message.text);
            const data = JSON.parse(jsonResult);
            webviewPanel.webview.postMessage({ command: 'sqlResult', data });
          } catch (err: any) {
            webviewPanel.webview.postMessage({ command: 'sqlError', error: err.message });
          }
        }

        if (message.command === 'saveState') {
          this.context.workspaceState.update(document.uri.toString(), message.state);
        }

        if (message.command === 'getSchema') {
          try {
            const jsonResult = db.schema();
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