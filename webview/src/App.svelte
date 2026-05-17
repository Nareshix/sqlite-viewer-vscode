<script lang="ts">
  import { onMount } from 'svelte';
  import * as monaco from 'monaco-editor';

  declare function acquireVsCodeApi(): any;
  let vscode: any;
  try {
    vscode = acquireVsCodeApi();
  } catch (e) {
    vscode = { postMessage: () => {} };
  }

  type Tab = { id: string; title: string; query: string; results: any[]; columns: string[]; error: string | null; };
  let tabCounter = 1;
  let tabs: Tab[] = $state([{ id: '1', title: 'Query 1', query: '-- Open a table from the sidebar or write a query\n', results: [], columns: [], error: null }]);
  let activeTabId: string = $state('1');
  let activeTab = $derived(tabs.find(t => t.id === activeTabId)!);
  let contextMenu: { visible: boolean; x: number; y: number; tableName: string } = $state({ visible: false, x: 0, y: 0, tableName: '' });
  let isRestored = false;

  let schema: { tables: any[], views: string[] } = $state({ tables: [], views: [] });
let dbName: string = $state('');
  let expandedTables: Set<string> = $state(new Set());
let searchQuery: string = $state('');
let searchOpen: boolean = $state(false);
let searchInput: HTMLInputElement | null = null;

function toggleSearch() {
  searchOpen = !searchOpen;
  if (searchOpen) {
    setTimeout(() => searchInput?.focus(), 0);
  } else {
    searchQuery = '';
  }
}
  let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null;
  let editorContainer: HTMLDivElement;

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
    tabs = tabs.map(t => t.id === activeTabId ? { ...t, error: null } : t);
    vscode.postMessage({ command: 'runSql', text });
  }

  function switchTab(id: string) {
    activeTabId = id;
    const tab = tabs.find(t => t.id === id)!;
    editorInstance?.setValue(tab.query);
  }

  function newTab() {
    tabCounter++;
    const id = String(tabCounter);
    tabs = [...tabs, { id, title: `Query ${tabCounter}`, query: '-- Write a query\n', results: [], columns: [], error: null }];
    switchTab(id);
  }

  function closeTab(id: string, e: MouseEvent) {
    e.stopPropagation();
    const idx = tabs.findIndex(t => t.id === id);
    tabs = tabs.filter(t => t.id !== id);
    if (activeTabId === id) {
      switchTab(tabs[Math.min(idx, tabs.length - 1)].id);
    }
  }

  function toggleTable(name: string) {
    const next = new Set(expandedTables);
    next.has(name) ? next.delete(name) : next.add(name);
    expandedTables = next;
  }

  function browseTable(name: string, forceNew = false) {
    const query = `SELECT * FROM "${name}" LIMIT 100;`;
    if (!forceNew) {
      const existing = tabs.find(t => t.title === name);
      if (existing) { switchTab(existing.id); return; }
    }
    tabCounter++;
    const id = String(tabCounter);
    tabs = [...tabs, { id, title: name, query, results: [], columns: [], error: null }];
    switchTab(id);
    setTimeout(() => runQuery(), 0);
  }

  function showContextMenu(e: MouseEvent, name: string) {
    e.preventDefault();
    contextMenu = { visible: true, x: e.clientX, y: e.clientY, tableName: name };
  }

  function hideContextMenu() {
    contextMenu = { ...contextMenu, visible: false };
  }

  function saveState() {
    if (!isRestored) return;

    // Strip results and columns so we don't blow up the IPC message limit
    // and VS Code's workspaceState size limit.
    const cleanTabs = tabs.map(t => ({
      id: t.id,
      title: t.title,
      query: t.query
    }));

    vscode.postMessage({ command: 'saveState', state: { tabs: cleanTabs, activeTabId, tabCounter } });
  }

  // Format row count: 1234 -> 1.2k
  function fmt(n: number): string {
    if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
    return String(n);
  }

  let filteredTables: any[] = $derived(
    schema.tables.filter((t: any) =>
      t.name.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  $effect(() => {
    tabs; activeTabId;
    saveState();
  });

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
      overviewRulerLanes: 0,
      overviewRulerBorder: false,
      hideCursorInOverviewRuler: true,
      wordBasedSuggestions: 'off'
    });

    editorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, runQuery);

    editorInstance.onDidChangeModelContent(() => {
      const val = editorInstance!.getValue();
      tabs = tabs.map(t => t.id === activeTabId ? { ...t, query: val } : t);
    });

    // Request schema and tell VS Code we are ready to receive state
    vscode.postMessage({ command: 'getSchema' });
    vscode.postMessage({ command: 'ready' });

    // NOTE: isRestored is set only inside the 'restoreState' handler below,
    // never via a setTimeout, so saveState() is a no-op until VS Code has
    // sent us the persisted state (or confirmed there is none).

    const messageHandler = (event: MessageEvent) => {
      const message = event.data;

      if (message.command === 'sqlResult') {
        const data = message.data || [];
        tabs = tabs.map(t =>
          t.id === activeTabId
            ? { ...t, results: data, columns: data.length > 0 ? Object.keys(data[0]) : [] }
            : t
        );
      } else if (message.command === 'sqlError') {
        tabs = tabs.map(t =>
          t.id === activeTabId
            ? { ...t, error: message.error, results: [], columns: [] }
            : t
        );
      } else if (message.command === 'schemaResult') {
        schema = message.schema;
        dbName = message.dbName ?? '';
      } else if (message.command === 'runQuery') {
        runQuery();
      } else if (message.command === 'restoreState') {
        if (message.state?.tabs) {
          // Re-inject empty results/columns/error because we stripped them on save
          tabs = message.state.tabs.map((t: any) => ({
            ...t,
            results: [],
            columns: [],
            error: null
          }));
          activeTabId = message.state.activeTabId;
          tabCounter = message.state.tabCounter;

          const tab = tabs.find((t: Tab) => t.id === activeTabId);
          editorInstance?.setValue(tab?.query ?? '');
        }

        // Mark as restored and immediately sync the clean state back,
        // discarding any corrupted/oversized state that was previously stored.
        isRestored = true;
        saveState();
      }
    };

    window.addEventListener('message', messageHandler);
    window.addEventListener('click', hideContextMenu);

    return () => {
      editorInstance?.dispose();
      sqliteProvider.dispose();
      window.removeEventListener('message', messageHandler);
      window.removeEventListener('click', hideContextMenu);
    };
  });
</script>

<style>
  .layout-container { display: flex; flex-direction: column; height: 100vh; width: 100vw; color: #ccc; font-family: sans-serif; }
  .layout-body { display: flex; flex-direction: row; flex: 1; min-height: 0; }
  .db-header { height: 1px; background: #333; flex-shrink: 0; }

  .main-pane { display: flex; flex-direction: column; flex: 1; min-width: 0; }
  .editor-pane { flex: 1; min-height: 0; position: relative; }
  .results-pane { flex: 1; background: #1e1e1e; border-top: 1px solid #444; overflow: auto; padding: 10px; display: flex; flex-direction: column; }

  /* sidebar */
  .sidebar { width: 240px; min-width: 240px; background: #1e1e1e; border-left: 1px solid #333; display: flex; flex-direction: column; overflow: hidden; }
  .search-icon { color: #666; cursor: pointer; font-size: 14px; padding: 2px 4px; border-radius: 3px; line-height: 1; user-select: none; flex-shrink: 0; }
  .search-icon:hover { color: #ccc; background: #2a2d2e; }
  .search-icon.active { color: #007acc; }
  .search-expand { padding: 4px 10px 6px; }
  .search-expand input { width: 100%; box-sizing: border-box; background: #2d2d2d; border: 1px solid #444; color: #ccc; padding: 4px 7px; font-size: 12px; border-radius: 3px; outline: none; }
  .search-expand input:focus { border-color: #007acc; }
  .sidebar-body { flex: 1; overflow-y: auto; padding: 8px 0; }

.section-header { padding: 4px 12px; font-size: 11px; font-weight: 600; color: #888; letter-spacing: 0.08em; text-transform: uppercase; display: flex; justify-content: space-between; align-items: center; }  .section-count { color: #555; }

  .table-row { display: flex; align-items: center; padding: 4px 8px 4px 12px; cursor: pointer; gap: 6px; }
  .table-row:hover { background: #2a2d2e; }
  .table-name { flex: 1; font-size: 13px; color: #ccc; }
  .table-count { font-size: 11px; color: #666; }
  .chevron { font-size: 15px; color: #555; width: 16px; display: inline-flex; align-items: center; justify-content: center; }
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

  .db-header { display: flex; align-items: center; gap: 6px; padding: 4px 10px; background: #1e1e1e; border-bottom: 1px solid #333; font-size: 12px; color: #888; flex-shrink: 0; }
  .db-name { flex: 1; color: #aaa; font-weight: 500; }
  .db-close { color: #555; font-size: 16px; line-height: 1; cursor: pointer; padding: 0 3px; border-radius: 3px; }
  .db-close:hover { color: #fff; background: #333; }

  table { width: 100%; border-collapse: collapse; font-family: monospace; font-size: 13px; }
  th, td { border: 1px solid #444; padding: 6px 10px; text-align: left; }
  th { background: #2d2d2d; color: #fff; position: sticky; top: 0; }
  .error { color: #f48771; font-family: monospace; white-space: pre-wrap; padding: 10px; background: rgba(255,0,0,0.1); border: 1px solid #f48771; }
  .empty-state { color: #888; font-style: italic; padding: 10px; }

  .tab-bar { display: flex; align-items: center; background: #1e1e1e; overflow-x: auto; flex-shrink: 0; }
  .tab { display: flex; align-items: center; gap: 6px; padding: 8px 14px; font-size: 12px; color: #666; cursor: pointer; white-space: nowrap; min-width: 80px; max-width: 160px; border-bottom: 2px solid transparent; }
  .tab:hover { color: #aaa; }
  .tab.active { color: #fff; border-bottom: 2px solid #007acc; }
  .tab-title { flex: 1; overflow: hidden; text-overflow: ellipsis; }
  .tab-close { color: #555; font-size: 14px; line-height: 1; padding: 0 2px; border-radius: 2px; }
  .tab-close:hover { color: #fff; background: #333; }
  .tab-new { padding: 8px 12px; color: #555; cursor: pointer; font-size: 18px; line-height: 1; }
  .tab-new:hover { color: #ccc; }
  .table-open { opacity: 0; font-size: 11px; color: #666; padding: 1px 3px; border-radius: 2px; }
  .table-row:hover .table-open { opacity: 1; color: #aaa; }
  .table-open:hover { color: #fff !important; background: #3a3a3a; }
  .context-menu { position: fixed; background: #2d2d2d; border: 1px solid #444; border-radius: 4px; z-index: 1000; min-width: 140px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); }
  .context-item { padding: 7px 14px; font-size: 12px; color: #ccc; cursor: pointer; }
  .context-item:hover { background: #007acc; color: #fff; }
</style>

<div class="layout-container">

  <div class="db-header"></div>

  <div class="layout-body">

  <!-- left: editor + results -->
  <div class="main-pane">
    <div class="tab-bar">
      {#each tabs as tab}
        <div class="tab" class:active={tab.id === activeTabId} onclick={() => switchTab(tab.id)}>
          <span class="tab-title">{tab.title}</span>
          {#if tabs.length > 1}
            <span class="tab-close" onclick={(e) => closeTab(tab.id, e)}>×</span>
          {/if}
        </div>
      {/each}
      <div class="tab-new" onclick={newTab} title="New tab">+</div>
    </div>
    <div class="editor-pane" bind:this={editorContainer}></div>
    <div class="results-pane">
      {#if activeTab?.error}
        <div class="error">{activeTab.error}</div>
      {:else if activeTab?.results.length > 0}
        <table>
          <thead>
            <tr>{#each activeTab.columns as col}<th>{col}</th>{/each}</tr>
          </thead>
          <tbody>
            {#each activeTab.results as row}
              <tr>{#each activeTab.columns as col}<td>{row[col] === null ? 'NULL' : row[col]}</td>{/each}</tr>
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
    <div class="sidebar-body">

      <!-- tables -->
      <div class="section-header">
        <span>Tables <span class="section-count">{schema.tables.length}</span></span>
        <span
          class="search-icon"
          class:active={searchOpen}
          onclick={toggleSearch}
          title={searchOpen ? 'Close search' : 'Search tables'}
        >⌕</span>
      </div>

      {#if searchOpen}
        <div class="search-expand">
          <input
            bind:this={searchInput}
            type="text"
            placeholder="search tables..."
            bind:value={searchQuery}
            onkeydown={(e) => e.key === 'Escape' && toggleSearch()}
          />
        </div>
      {/if}

      {#each filteredTables as table}
        <div
          class="table-row"
          onclick={() => toggleTable(table.name)}
          oncontextmenu={(e) => showContextMenu(e, table.name)}
          title="right-click for options"
        >
          <span class="chevron" class:open={expandedTables.has(table.name)}>›</span>
          <span class="table-name">{table.name}</span>
          <span class="table-count">{fmt(table.rowCount)}</span>
          <span class="table-open" onclick={(e) => { e.stopPropagation(); browseTable(table.name); }} title="Open table">↗</span>
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
  </div> <!-- end layout-body -->

  {#if contextMenu.visible}
    <div class="context-menu" style="left: {contextMenu.x}px; top: {contextMenu.y}px">
      <div class="context-item" onclick={() => { browseTable(contextMenu.tableName); hideContextMenu(); }}>Open</div>
      <div class="context-item" onclick={() => { browseTable(contextMenu.tableName, true); hideContextMenu(); }}>Open in New Tab</div>
    </div>
  {/if}

</div>