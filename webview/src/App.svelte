<script lang="ts">
  import { onMount } from 'svelte';
import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

  declare function acquireVsCodeApi(): any;
  let vscode: any;
  try {
    vscode = acquireVsCodeApi();
  } catch (e) {
    vscode = { postMessage: () => {} };
  }

  // existing state
  let results: any[] = $state([]);
  let columns: string[] = $state([]);
  let errorMessage: string | null = $state(null);

  // sidebar state
  let schema: { tables: any[], views: string[] } = $state({ tables: [], views: [] });
  let expandedTables: Set<string> = $state(new Set());
  let searchQuery: string = $state('');

  let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null;
  let editorContainer: HTMLDivElement;

  self.MonacoEnvironment = { getWorker() { return new editorWorker(); } };

  const SQLITE_KEYWORDS = [
    "ABORT","ACTION","ADD","AFTER","ALL","ALTER","ALWAYS","ANALYZE","AND","AS","ASC","ATTACH",
    "AUTOINCREMENT","BEFORE","BEGIN","BETWEEN","BY","CASCADE","CASE","CAST","CHECK","COLLATE",
    "COLUMN","COMMIT","CONFLICT","CONSTRAINT","CREATE","CROSS","CURRENT","CURRENT_DATE",
    "CURRENT_TIME","CURRENT_TIMESTAMP","DATABASE","DEFAULT","DEFERRABLE","DEFERRED","DELETE",
    "DESC","DETACH","DISTINCT","DO","DROP","EACH","ELSE","END","ESCAPE","EXCEPT","EXCLUDE",
    "EXCLUSIVE","EXISTS","EXPLAIN","FAIL","FILTER","FIRST","FOLLOWING","FOR","FOREIGN","FROM",
    "FULL","GENERATED","GLOB","GROUP","GROUPS","HAVING","IF","IGNORE","IMMEDIATE","IN","INDEX",
    "INDEXED","INITIALLY","INNER","INSERT","INSTEAD","INTERSECT","INTO","IS","ISNULL","JOIN",
    "KEY","LAST","LEFT","LIKE","LIMIT","MATCH","MATERIALIZED","NATURAL","NO","NOT","NOTHING",
    "NOTNULL","NULL","NULLS","OF","OFFSET","ON","OR","ORDER","OTHERS","OUTER","OVER","PARTITION",
    "PLAN","PRAGMA","PRECEDING","PRIMARY","QUERY","RAISE","RANGE","RECURSIVE","REFERENCES",
    "REGEXP","REINDEX","RELEASE","RENAME","REPLACE","RESTRICT","RETURNING","RIGHT","ROLLBACK",
    "ROW","ROWS","SAVEPOINT","SELECT","SET","TABLE","TEMP","TEMPORARY","THEN","TIES","TO",
    "TRANSACTION","TRIGGER","UNBOUNDED","UNION","UNIQUE","UPDATE","USING","VACUUM","VALUES",
    "VIEW","VIRTUAL","WHEN","WHERE","WINDOW","WITH","WITHOUT"
  ];

  function runQuery() {
    if (!editorInstance) return;
    const selection = editorInstance.getSelection();
    let text = editorInstance.getModel()?.getValueInRange(selection!) || '';
    if (!text.trim()) text = editorInstance.getValue();
    errorMessage = null;
    vscode.postMessage({ command: 'runSql', text });
  }

  function toggleTable(name: string) {
    const next = new Set(expandedTables);
    next.has(name) ? next.delete(name) : next.add(name);
    expandedTables = next;
  }

  function browseTable(name: string) {
    if (editorInstance) {
      editorInstance.setValue(`SELECT * FROM "${name}" LIMIT 100;`);
      runQuery();
    }
  }

  // format row count: 1234 -> 1.2k
  function fmt(n: number): string {
    if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
    return String(n);
  }


  let filteredTables: any[] = $derived(
    schema.tables.filter((t: any) =>
      t.name.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

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
          startLineNumber: position.lineNumber, endLineNumber: position.lineNumber,
          startColumn: word.startColumn, endColumn: word.endColumn
        };
        const keywordSuggestions = SQLITE_KEYWORDS.map(k => ({
          label: k, kind: monaco.languages.CompletionItemKind.Keyword, insertText: k, range
        }));
        const snippetSuggestions = [
          {
            label: 'SELECT ... FROM ...',
            kind: monaco.languages.CompletionItemKind.Snippet,
            insertText: 'SELECT ${1:*} FROM ${2:table};',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, range
          },
          {
            label: 'CREATE TABLE ...',
            kind: monaco.languages.CompletionItemKind.Snippet,
            insertText: 'CREATE TABLE ${1:name} (\n  ${2:id} INTEGER PRIMARY KEY\n);',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, range
          }
        ];
        return { suggestions: [...keywordSuggestions, ...snippetSuggestions] };
      }
    });

    editorInstance = monaco.editor.create(editorContainer, {
      value: '-- Open a table from the sidebar or write a query\n',
      language: 'sqlite-custom',
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: false },
      wordBasedSuggestions: 'off'
    });

    editorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, runQuery);

    // request schema immediately on load
    vscode.postMessage({ command: 'getSchema' });

    const messageHandler = (event: MessageEvent) => {
      const message = event.data;
      if (message.command === 'sqlResult') {
        results = message.data || [];
        columns = results.length > 0 ? Object.keys(results[0]) : [];
      } else if (message.command === 'sqlError') {
        errorMessage = message.error;
        results = [];
        columns = [];
      } else if (message.command === 'schemaResult') {
        schema = message.schema;
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
  .layout-container { display: flex; flex-direction: row; height: 100vh; width: 100vw; color: #ccc; font-family: sans-serif; }

  .main-pane { display: flex; flex-direction: column; flex: 1; min-width: 0; }
  .editor-pane { flex: 1; min-height: 0; position: relative; }
  .results-pane { flex: 1; background: #1e1e1e; border-top: 1px solid #444; overflow: auto; padding: 10px; display: flex; flex-direction: column; }

  /* sidebar */
  .sidebar { width: 240px; min-width: 240px; background: #1e1e1e; border-left: 1px solid #333; display: flex; flex-direction: column; overflow: hidden; }
  .sidebar-search { padding: 8px; border-bottom: 1px solid #333; }
  .sidebar-search input { width: 100%; box-sizing: border-box; background: #2d2d2d; border: 1px solid #444; color: #ccc; padding: 5px 8px; font-size: 12px; border-radius: 3px; outline: none; }
  .sidebar-search input:focus { border-color: #007acc; }
  .sidebar-body { flex: 1; overflow-y: auto; padding: 8px 0; }

  .section-header { padding: 4px 12px; font-size: 11px; font-weight: 600; color: #888; letter-spacing: 0.08em; text-transform: uppercase; display: flex; justify-content: space-between; }
  .section-count { color: #555; }

  .table-row { display: flex; align-items: center; padding: 4px 8px 4px 12px; cursor: pointer; gap: 6px; }
  .table-row:hover { background: #2a2d2e; }
  .table-name { flex: 1; font-size: 13px; color: #ccc; }
  .table-count { font-size: 11px; color: #666; }
  .chevron { font-size: 10px; color: #555; width: 12px; transition: transform 0.15s; }
  .chevron.open { transform: rotate(90deg); }

  .column-list { padding: 0 0 4px 28px; }
  .column-item { display: flex; align-items: center; gap: 5px; padding: 2px 8px; font-size: 11px; color: #888; }
  .col-name { color: #bbb; }
  .col-type { color: #666; font-family: monospace; }
  .badge { font-size: 9px; padding: 1px 4px; border-radius: 2px; font-weight: 600; }
  .badge-pk { background: #2d4a1e; color: #7ec850; }
  .badge-null { background: #333; color: #888; }

  .view-row { padding: 4px 12px; font-size: 13px; color: #9cdcfe; cursor: pointer; }
  .view-row:hover { background: #2a2d2e; }

  /* existing styles */
  table { width: 100%; border-collapse: collapse; font-family: monospace; font-size: 13px; }
  th, td { border: 1px solid #444; padding: 6px 10px; text-align: left; }
  th { background: #2d2d2d; color: #fff; position: sticky; top: 0; }
  .toolbar { padding-bottom: 10px; border-bottom: 1px solid #444; margin-bottom: 10px; }
  button { background: #007acc; color: white; border: none; padding: 6px 12px; cursor: pointer; border-radius: 2px; }
  button:hover { background: #005f9e; }
  .error { color: #f48771; font-family: monospace; white-space: pre-wrap; padding: 10px; background: rgba(255,0,0,0.1); border: 1px solid #f48771; }
  .empty-state { color: #888; font-style: italic; padding: 10px; }
</style>

<div class="layout-container">

  <!-- left: editor + results -->
  <div class="main-pane">
    <div class="editor-pane" bind:this={editorContainer}></div>
    <div class="results-pane">
      <div class="toolbar">
        <button onclick={runQuery}>Run Query</button>
      </div>
      {#if errorMessage}
        <div class="error">{errorMessage}</div>
      {:else if results.length > 0}
        <table>
          <thead>
            <tr>{#each columns as col}<th>{col}</th>{/each}</tr>
          </thead>
          <tbody>
            {#each results as row}
              <tr>{#each columns as col}<td>{row[col] === null ? 'NULL' : row[col]}</td>{/each}</tr>
            {/each}
          </tbody>
        </table>
      {:else}
        <div class="empty-state">No results to display.</div>
      {/if}
    </div>
  </div>

  <!-- right: sidebar -->
  <div class="sidebar">
    <div class="sidebar-search">
      <input
        type="text"
        placeholder="search tables..."
        bind:value={searchQuery}
      />
    </div>
    <div class="sidebar-body">

      <!-- tables -->
      <div class="section-header">
        Tables <span class="section-count">{schema.tables.length}</span>
      </div>

      {#each filteredTables as table}
        <div
          class="table-row"
          onclick={() => toggleTable(table.name)}
          ondblclick={() => browseTable(table.name)}
          title="double-click to browse"
        >
          <span class="chevron" class:open={expandedTables.has(table.name)}>›</span>
          <span class="table-name">{table.name}</span>
          <span class="table-count">{fmt(table.rowCount)}</span>
        </div>

        {#if expandedTables.has(table.name)}
          <div class="column-list">
            {#each table.columns as col}
              <div class="column-item">
                {#if col.pk > 0}
                  <span class="badge badge-pk">PK</span>
                {/if}
                <span class="col-name">{col.name}</span>
                <span class="col-type">{col.type || 'ANY'}</span>
                {#if col.notnull === 0 && col.pk === 0}
                  <span class="badge badge-null">null</span>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      {/each}

      <!-- views -->
      {#if schema.views.length > 0}
        <div class="section-header" style="margin-top: 12px">
          Views <span class="section-count">{schema.views.length}</span>
        </div>
        {#each schema.views as view}
          <div class="view-row" ondblclick={() => browseTable(view)}>{view}</div>
        {/each}
      {/if}

    </div>
  </div>

</div>