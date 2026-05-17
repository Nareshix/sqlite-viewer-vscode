<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { store, vscode } from '../store.svelte';

  let { height = null }: { height: number | null } = $props();

  let editorContainer: HTMLDivElement;
  let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null;

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

    const tab = store.activeTab;
    if (tab) tab.error = null;

    vscode.postMessage({ command: 'runSql', text });
  }

  store.executeRunQuery = runQuery;

  $effect(() => {
    if (editorInstance && store.activeTabId) {
      const currentTab = store.activeTab;
      if (currentTab && editorInstance.getValue() !== currentTab.query) {
        editorInstance.setValue(currentTab.query);
      }
    }
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
          { label: 'SELECT ... FROM ...', kind: monaco.languages.CompletionItemKind.Snippet, insertText: 'SELECT ${1:*} FROM ${2:table};', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, range },
          { label: 'CREATE TABLE ...', kind: monaco.languages.CompletionItemKind.Snippet, insertText: 'CREATE TABLE ${1:name} (\n  ${2:id} INTEGER PRIMARY KEY\n);', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, range }
        ];
        return { suggestions: [...keywordSuggestions, ...snippetSuggestions] };
      }
    });

    editorInstance = monaco.editor.create(editorContainer, {
      value: store.activeTab?.query || '',
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
      const tab = store.tabs.find(t => t.id === store.activeTabId);
      if (tab) tab.query = val;
    });

    return () => {
      editorInstance?.dispose();
      sqliteProvider.dispose();
    };
  });
</script>

<div
  bind:this={editorContainer}
  style={height !== null ? `height: ${height}px` : ''}
  class={height !== null ? 'relative shrink-0' : 'flex-1 min-h-0 relative'}
></div>