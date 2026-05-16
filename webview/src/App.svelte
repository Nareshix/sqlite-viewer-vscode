<script lang="ts">
  import { onMount } from 'svelte';
  import * as monaco from 'monaco-editor';

  import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

  self.MonacoEnvironment = {
    getWorker() {
      // We only need the basic editor worker now since we are making a custom language!
      return new editorWorker();
    }
  };

  const SQLITE_KEYWORDS = [
    "ABORT", "ACTION", "ADD", "AFTER", "ALL", "ALTER", "ALWAYS", "ANALYZE", "AND", "AS", "ASC", "ATTACH", "AUTOINCREMENT", "BEFORE", "BEGIN", "BETWEEN", "BY", "CASCADE", "CASE", "CAST", "CHECK", "COLLATE", "COLUMN", "COMMIT", "CONFLICT", "CONSTRAINT", "CREATE", "CROSS", "CURRENT", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP", "DATABASE", "DEFAULT", "DEFERRABLE", "DEFERRED", "DELETE", "DESC", "DETACH", "DISTINCT", "DO", "DROP", "EACH", "ELSE", "END", "ESCAPE", "EXCEPT", "EXCLUDE", "EXCLUSIVE", "EXISTS", "EXPLAIN", "FAIL", "FILTER", "FIRST", "FOLLOWING", "FOR", "FOREIGN", "FROM", "FULL", "GENERATED", "GLOB", "GROUP", "GROUPS", "HAVING", "IF", "IGNORE", "IMMEDIATE", "IN", "INDEX", "INDEXED", "INITIALLY", "INNER", "INSERT", "INSTEAD", "INTERSECT", "INTO", "IS", "ISNULL", "JOIN", "KEY", "LAST", "LEFT", "LIKE", "LIMIT", "MATCH", "MATERIALIZED", "NATURAL", "NO", "NOT", "NOTHING", "NOTNULL", "NULL", "NULLS", "OF", "OFFSET", "ON", "OR", "ORDER", "OTHERS", "OUTER", "OVER", "PARTITION", "PLAN", "PRAGMA", "PRECEDING", "PRIMARY", "QUERY", "RAISE", "RANGE", "RECURSIVE", "REFERENCES", "REGEXP", "REINDEX", "RELEASE", "RENAME", "REPLACE", "RESTRICT", "RETURNING", "RIGHT", "ROLLBACK", "ROW", "ROWS", "SAVEPOINT", "SELECT", "SET", "TABLE", "TEMP", "TEMPORARY", "THEN", "TIES", "TO", "TRANSACTION", "TRIGGER", "UNBOUNDED", "UNION", "UNIQUE", "UPDATE", "USING", "VACUUM", "VALUES", "VIEW", "VIRTUAL", "WHEN", "WHERE", "WINDOW", "WITH", "WITHOUT"
  ];

  let editorContainer: HTMLDivElement;

  onMount(() => {
    // 1. REGISTER OUR NEW CUSTOM LANGUAGE
    monaco.languages.register({ id: 'sqlite-custom' });

    // 2. DEFINE THE SYNTAX HIGHLIGHTING RULES (Monarch Grammar)
    monaco.languages.setMonarchTokensProvider('sqlite-custom', {
      ignoreCase: true, // SQL is case insensitive (select == SELECT)
      keywords: SQLITE_KEYWORDS,

      tokenizer: {
        root: [
          // If it matches a word, check if it's in our keywords array.
          // If yes, color it as a 'keyword' (blue in dark theme).
          // If no, color it as 'identifier' (plain white).
          [/[a-zA-Z_]\w*/, { cases: { '@keywords': 'keyword', '@default': 'identifier' } }],

          // Match standard SQL strings (single quotes) and color them
          [/'([^'\\]|\\.)*'/, 'string'],

          // Match numbers
          [/\d+/, 'number'],

          // Match comments (e.g., -- this is a comment)
          [/--.*$/, 'comment'],
        ]
      }
    });

    // 3. SET UP AUTOCOMPLETE FOR OUR CUSTOM LANGUAGE
    const sqliteProvider = monaco.languages.registerCompletionItemProvider('sqlite-custom', {
      provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position);
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn
        };

        const keywordSuggestions: monaco.languages.CompletionItem[] = SQLITE_KEYWORDS.map(keyword => {
          return {
            label: keyword,
            kind: monaco.languages.CompletionItemKind.Keyword,
            insertText: keyword,
            range: range
          };
        });

        const tableSuggestions: monaco.languages.CompletionItem[] = [
          { label: 'users', kind: monaco.languages.CompletionItemKind.Class, detail: 'Table', insertText: 'users', range },
          { label: 'posts', kind: monaco.languages.CompletionItemKind.Class, detail: 'Table', insertText: 'posts', range },
        ];

        return { suggestions: [...keywordSuggestions, ...tableSuggestions] };
      }
    });

    // 4. CREATE THE EDITOR USING 'sqlite-custom'
    const editor = monaco.editor.create(editorContainer, {
      value: '-- Custom highlighting active!\nSELECT * FROM user;\n',
      language: 'sqlite-custom', // IMPORTANT: Use our new custom language!
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
  .layout-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    color: #cccccc;
    font-family: sans-serif;
  }

  .editor-pane {
    flex: 1;
    min-height: 0;
  }

  .results-pane {
    flex: 1;
    background-color: #1e1e1e;
    border-top: 1px solid #444444;
    overflow: auto;
    padding: 10px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-family: monospace;
    font-size: 14px;
  }

  th, td {
    border: 1px solid #444444;
    padding: 8px;
    text-align: left;
  }

  th {
    background-color: #2d2d2d;
    color: #ffffff;
  }
</style>

<div class="layout-container">
  <!-- TOP HALF: Monaco Editor -->
  <div class="editor-pane" bind:this={editorContainer}></div>

  <!-- BOTTOM HALF: SQLite Results Table -->
  <div class="results-pane">
    <table>
      <thead>
        <tr>
          <th>id</th>
          <th>username</th>
          <th>email</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>1</td>
          <td>naresh</td>
          <td>naresh@example.com</td>
        </tr>
      </tbody>
    </table>
  </div>
</div>