<script lang="ts">
  import { onMount } from 'svelte';
  import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
  import 'monaco-editor/esm/vs/editor/editor.all.js';
  import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

  // 1. Declare VS Code API to avoid TypeScript errors
  declare function acquireVsCodeApi(): any;
  let vscode: any;

  try {
    vscode = acquireVsCodeApi();
  } catch (e) {
    // Fallback if testing outside VS Code
    vscode = { postMessage: () => {} };
  }

  // 2. Reactive state variables for the UI
  let results: any[] = $state([]);
  let columns: string[] = $state([]);
  let errorMessage: string | null = $state(null);
  let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null;

  self.MonacoEnvironment = {
    getWorker() {
      return new editorWorker();
    }
  };

  const SQLITE_KEYWORDS =[
    "ABORT", "ACTION", "ADD", "AFTER", "ALL", "ALTER", "ALWAYS", "ANALYZE", "AND", "AS", "ASC", "ATTACH", "AUTOINCREMENT", "BEFORE", "BEGIN", "BETWEEN", "BY", "CASCADE", "CASE", "CAST", "CHECK", "COLLATE", "COLUMN", "COMMIT", "CONFLICT", "CONSTRAINT", "CREATE", "CROSS", "CURRENT", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP", "DATABASE", "DEFAULT", "DEFERRABLE", "DEFERRED", "DELETE", "DESC", "DETACH", "DISTINCT", "DO", "DROP", "EACH", "ELSE", "END", "ESCAPE", "EXCEPT", "EXCLUDE", "EXCLUSIVE", "EXISTS", "EXPLAIN", "FAIL", "FILTER", "FIRST", "FOLLOWING", "FOR", "FOREIGN", "FROM", "FULL", "GENERATED", "GLOB", "GROUP", "GROUPS", "HAVING", "IF", "IGNORE", "IMMEDIATE", "IN", "INDEX", "INDEXED", "INITIALLY", "INNER", "INSERT", "INSTEAD", "INTERSECT", "INTO", "IS", "ISNULL", "JOIN", "KEY", "LAST", "LEFT", "LIKE", "LIMIT", "MATCH", "MATERIALIZED", "NATURAL", "NO", "NOT", "NOTHING", "NOTNULL", "NULL", "NULLS", "OF", "OFFSET", "ON", "OR", "ORDER", "OTHERS", "OUTER", "OVER", "PARTITION", "PLAN", "PRAGMA", "PRECEDING", "PRIMARY", "QUERY", "RAISE", "RANGE", "RECURSIVE", "REFERENCES", "REGEXP", "REINDEX", "RELEASE", "RENAME", "REPLACE", "RESTRICT", "RETURNING", "RIGHT", "ROLLBACK", "ROW", "ROWS", "SAVEPOINT", "SELECT", "SET", "TABLE", "TEMP", "TEMPORARY", "THEN", "TIES", "TO", "TRANSACTION", "TRIGGER", "UNBOUNDED", "UNION", "UNIQUE", "UPDATE", "USING", "VACUUM", "VALUES", "VIEW", "VIRTUAL", "WHEN", "WHERE", "WINDOW", "WITH", "WITHOUT"
  ];

  let editorContainer: HTMLDivElement;

  // 3. Function to send query to the backend
  function runQuery() {
    if (!editorInstance) return;

    // Get the selected text, or the whole text if nothing is highlighted
    const selection = editorInstance.getSelection();
    let text = editorInstance.getModel()?.getValueInRange(selection!) || '';
    if (!text.trim()) {
      text = editorInstance.getValue();
    }

    errorMessage = null; // Clear previous errors

    vscode.postMessage({
      command: 'runSql',
      text: text
    });
  }

  onMount(() => {
    monaco.languages.register({ id: 'sqlite-custom' });

    monaco.languages.setMonarchTokensProvider('sqlite-custom', {
      ignoreCase: true,
      keywords: SQLITE_KEYWORDS,
      tokenizer: {
        root: [
          [/[a-zA-Z_]\w*/, { cases: { '@keywords': 'keyword', '@default': 'identifier' } }],
          [/'([^'\\]|\\.)*'/, 'string'],
          [/\d+/, 'number'],
          [/--.*$/, 'comment'],
        ]
      }
    });

    const sqliteProvider = monaco.languages.registerCompletionItemProvider('sqlite-custom', {
      provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position);
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn
        };

        const keywordSuggestions = SQLITE_KEYWORDS.map(k => ({
          label: k, kind: monaco.languages.CompletionItemKind.Keyword, insertText: k, range
        }));

        const snippetSuggestions =[
          {
            label: 'SELECT ... FROM ...',
            kind: monaco.languages.CompletionItemKind.Snippet,
            insertText: 'SELECT ${1:*} FROM ${2:table};',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range
          },
          {
            label: 'CREATE TABLE ...',
            kind: monaco.languages.CompletionItemKind.Snippet,
            insertText: 'CREATE TABLE ${1:name} (\n  ${2:id} INTEGER PRIMARY KEY\n);',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range
          }
        ];

        return { suggestions: [...keywordSuggestions, ...snippetSuggestions] };
      }
    });

    editorInstance = monaco.editor.create(editorContainer, {
      value: '-- Custom SQLite Editor\nSELECT * FROM users;\n',
      language: 'sqlite-custom',
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: false },
      wordBasedSuggestions: 'off'
    });

    // Bind Ctrl+Enter (Cmd+Enter on Mac) to run the query inside Monaco
    editorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, runQuery);

    // 4. Listen for the results coming back from VS Code Extension Host
    const messageHandler = (event: MessageEvent) => {
      const message = event.data;

      if (message.command === 'sqlResult') {
        results = message.data || [];
        // Extract column names dynamically from the first row
        columns = results.length > 0 ? Object.keys(results[0]) : [];
      } else if (message.command === 'sqlError') {
        errorMessage = message.error;
        results = [];
        columns = [];
      }
    };

    window.addEventListener('message', messageHandler);

    return () => {
      editorInstance?.dispose();
      sqliteProvider.dispose();
      window.removeEventListener('message', messageHandler);
    };
  });
</script>

<style>
  .layout-container { display: flex; flex-direction: column; height: 100vh; width: 100vw; color: #ccc; font-family: sans-serif; }
  .editor-pane { flex: 1; min-height: 0; position: relative; }
  .results-pane { flex: 1; background: #1e1e1e; border-top: 1px solid #444; overflow: auto; padding: 10px; display: flex; flex-direction: column;}

  table { width: 100%; border-collapse: collapse; font-family: monospace; font-size: 13px; }
  th, td { border: 1px solid #444; padding: 6px 10px; text-align: left; }
  th { background: #2d2d2d; color: #fff; position: sticky; top: 0; }

  .toolbar { padding-bottom: 10px; border-bottom: 1px solid #444; margin-bottom: 10px;}
  button { background: #007acc; color: white; border: none; padding: 6px 12px; cursor: pointer; border-radius: 2px;}
  button:hover { background: #005f9e; }

  .error { color: #f48771; font-family: monospace; white-space: pre-wrap; padding: 10px; background: rgba(255,0,0,0.1); border: 1px solid #f48771; }
  .empty-state { color: #888; font-style: italic; padding: 10px; }
</style>

<div class="layout-container">
  <div class="editor-pane" bind:this={editorContainer}></div>
  <div class="results-pane">
    <div class="toolbar">
      <button onclick={runQuery}>Run Query</button>
    </div>

    <!-- 5. Render Error or Table dynamically -->
    {#if errorMessage}
      <div class="error">{errorMessage}</div>
    {:else if results.length > 0}
      <table>
        <thead>
          <tr>
            {#each columns as col}
              <th>{col}</th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each results as row}
            <tr>
              {#each columns as col}
                <td>{row[col] === null ? 'NULL' : row[col]}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <div class="empty-state">No results to display.</div>
    {/if}
  </div>
</div>