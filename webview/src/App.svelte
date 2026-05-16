<script lang="ts">
  import { onMount } from 'svelte';
  import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
  import 'monaco-editor/esm/vs/editor/editor.all.js';
  import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

  self.MonacoEnvironment = {
    getWorker() {
      return new editorWorker();
    }
  };

  const SQLITE_KEYWORDS =[
    "ABORT", "ACTION", "ADD", "AFTER", "ALL", "ALTER", "ALWAYS", "ANALYZE", "AND", "AS", "ASC", "ATTACH", "AUTOINCREMENT", "BEFORE", "BEGIN", "BETWEEN", "BY", "CASCADE", "CASE", "CAST", "CHECK", "COLLATE", "COLUMN", "COMMIT", "CONFLICT", "CONSTRAINT", "CREATE", "CROSS", "CURRENT", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP", "DATABASE", "DEFAULT", "DEFERRABLE", "DEFERRED", "DELETE", "DESC", "DETACH", "DISTINCT", "DO", "DROP", "EACH", "ELSE", "END", "ESCAPE", "EXCEPT", "EXCLUDE", "EXCLUSIVE", "EXISTS", "EXPLAIN", "FAIL", "FILTER", "FIRST", "FOLLOWING", "FOR", "FOREIGN", "FROM", "FULL", "GENERATED", "GLOB", "GROUP", "GROUPS", "HAVING", "IF", "IGNORE", "IMMEDIATE", "IN", "INDEX", "INDEXED", "INITIALLY", "INNER", "INSERT", "INSTEAD", "INTERSECT", "INTO", "IS", "ISNULL", "JOIN", "KEY", "LAST", "LEFT", "LIKE", "LIMIT", "MATCH", "MATERIALIZED", "NATURAL", "NO", "NOT", "NOTHING", "NOTNULL", "NULL", "NULLS", "OF", "OFFSET", "ON", "OR", "ORDER", "OTHERS", "OUTER", "OVER", "PARTITION", "PLAN", "PRAGMA", "PRECEDING", "PRIMARY", "QUERY", "RAISE", "RANGE", "RECURSIVE", "REFERENCES", "REGEXP", "REINDEX", "RELEASE", "RENAME", "REPLACE", "RESTRICT", "RETURNING", "RIGHT", "ROLLBACK", "ROW", "ROWS", "SAVEPOINT", "SELECT", "SET", "TABLE", "TEMP", "TEMPORARY", "THEN", "TIES", "TO", "TRANSACTION", "TRIGGER", "UNBOUNDED", "UNION", "UNIQUE", "UPDATE", "USING", "VACUUM", "VALUES", "VIEW", "VIRTUAL", "WHEN", "WHERE", "WINDOW", "WITH", "WITHOUT"
  ];

  let editorContainer: HTMLDivElement;

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

        const keywordSuggestions: monaco.languages.CompletionItem[] = SQLITE_KEYWORDS.map(k => ({
          label: k, kind: monaco.languages.CompletionItemKind.Keyword, insertText: k, range
        }));

        const snippetSuggestions: monaco.languages.CompletionItem[] =[
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

    const editor = monaco.editor.create(editorContainer, {
      value: '-- Custom SQLite Editor\nSELECT * FROM users;\n',
      language: 'sqlite-custom',
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: false },
      wordBasedSuggestions: 'off'
    });

    return () => {
      editor.dispose();
      sqliteProvider.dispose();
    };
  });
</script>

<style>
  .layout-container { display: flex; flex-direction: column; height: 100vh; width: 100vw; color: #ccc; font-family: sans-serif; }
  .editor-pane { flex: 1; min-height: 0; }
  .results-pane { flex: 1; background: #1e1e1e; border-top: 1px solid #444; overflow: auto; padding: 10px; }
  table { width: 100%; border-collapse: collapse; font-family: monospace; }
  th, td { border: 1px solid #444; padding: 8px; text-align: left; }
  th { background: #2d2d2d; color: #fff; }
</style>

<div class="layout-container">
  <div class="editor-pane" bind:this={editorContainer}></div>
  <div class="results-pane">
    <table>
      <thead><tr><th>id</th><th>username</th><th>email</th></tr></thead>
      <tbody><tr><td>1</td><td>naresh</td><td>naresh@example.com</td></tr></tbody>
    </table>
  </div>
</div>