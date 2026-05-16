Some ideas
<img width="1440" height="1040" alt="image" src="https://github.com/user-attachments/assets/f42b6d0f-edd8-4db9-becd-9bd75cf8940a" />


<img width="1440" height="1120" alt="image" src="https://github.com/user-attachments/assets/72f0822d-19a9-430f-98e7-83198de08ec9" />
<img width="1440" height="1160" alt="image" src="https://github.com/user-attachments/assets/b1fd781f-ca2a-4af0-92e4-49511b65c800" />
<img width="1440" height="1160" alt="image" src="https://github.com/user-attachments/assets/1fc42489-84dd-4b3f-bb61-2875f9179813" />


some interesting ideas
FK navigation — when you're browsing orders and see user_id = 1042, clicking that value jumps you directly to the users table and highlights the row where id = 1042. You follow the relationship without writing a JOIN. That's FK navigation — treating foreign key values as clickable links rather than dead numbers.

```ts
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>monaco sql test</title>
  <style>
    html, body { margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: #1e1e1e; }
    #editor { width: 100%; height: 100%; }
  </style>
</head>
<body>
<div id="editor"></div>
<script type="module">
import { init } from "https://esm.sh/modern-monaco@0.4.1";

const monaco = await init({ defaultTheme: "dark-plus" });

const editor = monaco.editor.create(document.getElementById("editor"), {
  language: "sql",
  fontSize: 15,
  value: "SELECT * FROM users WHERE id = 1;\n",
  automaticLayout: true,  // handles resize automatically
});

// SQL keyword completions
const SQL_KEYWORDS = [
  "SELECT","FROM","WHERE","JOIN","LEFT JOIN","RIGHT JOIN","INNER JOIN","OUTER JOIN",
  "ON","AS","AND","OR","NOT","IN","EXISTS","BETWEEN","LIKE","IS NULL","IS NOT NULL",
  "GROUP BY","ORDER BY","HAVING","LIMIT","OFFSET","DISTINCT","ALL","UNION","UNION ALL",
  "INSERT INTO","VALUES","UPDATE","SET","DELETE FROM","CREATE TABLE","DROP TABLE",
  "ALTER TABLE","ADD COLUMN","DROP COLUMN","PRIMARY KEY","FOREIGN KEY","REFERENCES",
  "UNIQUE","NOT NULL","DEFAULT","COUNT","SUM","AVG","MIN","MAX","COALESCE","NULLIF",
  "CASE","WHEN","THEN","ELSE","END","CAST","NOW","CONCAT","LENGTH","UPPER","LOWER",
  "TRIM","SUBSTRING","REPLACE","ROUND","FLOOR","CEIL","ABS","IFNULL",
];

monaco.languages.registerCompletionItemProvider("sql", {
  provideCompletionItems(model, position) {
    const word = model.getWordUntilPosition(position);
    const range = {
      startLineNumber: position.lineNumber,
      endLineNumber: position.lineNumber,
      startColumn: word.startColumn,
      endColumn: word.endColumn,
    };
    return {
      suggestions: SQL_KEYWORDS.map(kw => ({
        label: kw,
        kind: monaco.languages.CompletionItemKind.Keyword,
        insertText: kw,
        range,
      })),
    };
  },
});
</script>
</body>
</html>
```
