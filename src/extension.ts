import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { Database } from '../backend';

class SQLiteEditorProvider implements vscode.CustomReadonlyEditorProvider {

  constructor(private readonly context: vscode.ExtensionContext) {}

  openCustomDocument(uri: vscode.Uri): vscode.CustomDocument {
    // VS Code needs a document object — we just need the uri, nothing fancy
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

    // Load the Vite-built HTML, inject base href (same as before)
    const htmlPath = path.join(webviewDir.fsPath, 'index.html');
    let html = fs.readFileSync(htmlPath, 'utf8');
    const webviewUri = webviewPanel.webview.asWebviewUri(webviewDir);
    html = html.replace('<head>', `<head>\n    <base href="${webviewUri}/">`);
    webviewPanel.webview.html = html;

    try {
      // Open the actual .db file instead of :memory:
      const db = new Database(document.uri.fsPath);

      webviewPanel.webview.onDidReceiveMessage(message => {
        if (message.command === 'runSql') {
          try {
            const jsonResult = db.query(message.text);
            const data = JSON.parse(jsonResult);
            webviewPanel.webview.postMessage({ command: 'sqlResult', data });
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
    vscode.window.registerCustomEditorProvider(
      'monaco-hello.sqliteEditor', // must match viewType in package.json
      provider,
      { webviewOptions: { retainContextWhenHidden: true } }
    )
  );
}

export function deactivate() {}