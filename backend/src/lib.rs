#![deny(clippy::all)]

use napi_derive::napi;
use sqlitex::traits::dynamic::Value;
use sqlitex::Connection;
use std::collections::HashMap;
use std::sync::Arc;

#[napi]
pub struct Database {
  conn: Arc<Connection>,
}

#[napi]
impl Database {
  #[napi(constructor)]
  pub fn new(path: String) -> napi::Result<Self> {
    let conn = if path == ":memory:" {
      Connection::open_memory().map_err(|e| napi::Error::from_reason(e.to_string()))?
    } else {
      Connection::open(&path).map_err(|e| napi::Error::from_reason(e.to_string()))?
    };
    Ok(Self { conn })
  }

  #[napi]
  pub async fn query(&self, sql: String) -> napi::Result<String> {
    let conn = self.conn.clone();

    tokio::task::spawn_blocking(move || {
      let results = conn
        .query(&sql)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

      let mut rows_json = Vec::new();
      let col_names = results.column_names.clone();

      let mut count = 0;
      // THE LIFEGUARD: Protects VS Code from Out-Of-Memory crashes.
      let max_rows = 100000;

      for row_result in results {
        // Scenario A: User forgot LIMIT on a 10-million row table.
        // We forcefully break at 100,000 to save the extension host from crashing.
        if count >= max_rows {
          break;
        }

        let row = row_result.map_err(|e| napi::Error::from_reason(e.to_string()))?;
        let mut row_map = serde_json::Map::new();

        for (i, col_name) in col_names.iter().enumerate() {
          let json_val = match &row[i] {
            Value::Integer(v) => serde_json::Value::Number((*v).into()),
            Value::Real(v) => serde_json::Value::Number(serde_json::Number::from_f64(*v).unwrap()),
            Value::Text(v) => serde_json::Value::String(v.clone()),
            Value::Blob(_) => serde_json::Value::String("[BLOB]".to_string()),
            Value::Null => serde_json::Value::Null,
          };
          row_map.insert(col_name.clone(), json_val);
        }
        rows_json.push(serde_json::Value::Object(row_map));

        count += 1;
        // Scenario B: User typed "LIMIT 5".
        // This loop will naturally exit after count == 5 because `results` runs out of rows.
      }

      Ok(serde_json::to_string(&rows_json).unwrap())
    })
    .await
    .unwrap_or_else(|_| Err(napi::Error::from_reason("Background thread panicked".to_string())))
  }
  #[napi]
  pub async fn schema(&self) -> napi::Result<String> {
    let conn = self.conn.clone();

    // Schema generation can also take time on large databases, so put it in a background thread too!
    tokio::task::spawn_blocking(move || {
      let tables_result = conn
        .query("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

      let mut table_names: Vec<String> = Vec::new();
      for row_result in tables_result {
        let row = row_result.map_err(|e| napi::Error::from_reason(e.to_string()))?;
        if let Value::Text(name) = &row[0] {
          table_names.push(name.clone());
        }
      }

      let mut tables_json = Vec::new();

      for table_name in &table_names {
        let count_sql = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
        let mut count_result = conn
          .query(&count_sql)
          .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let row_count: i64 = count_result
          .next()
          .ok_or_else(|| napi::Error::from_reason("no count".to_string()))?
          .map_err(|e| napi::Error::from_reason(e.to_string()))
          .map(|row| match &row[0] {
            Value::Integer(v) => *v,
            _ => 0,
          })?;

        let pragma_sql = format!("PRAGMA table_info(\"{}\")", table_name);
        let pragma_result = conn
          .query(&pragma_sql)
          .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let pragma_cols = pragma_result.column_names.clone();
        let mut columns = Vec::new();

        for row_result in pragma_result {
          let row = row_result.map_err(|e| napi::Error::from_reason(e.to_string()))?;
          let mut col = serde_json::Map::new();
          for (i, col_name) in pragma_cols.iter().enumerate() {
            let val = match &row[i] {
              Value::Integer(v) => serde_json::Value::Number((*v).into()),
              Value::Text(v) => serde_json::Value::String(v.clone()),
              Value::Null => serde_json::Value::Null,
              Value::Real(v) => serde_json::Value::Number(serde_json::Number::from_f64(*v).unwrap()),
              Value::Blob(_) => serde_json::Value::Null,
            };
            col.insert(col_name.clone(), val);
          }
          columns.push(serde_json::Value::Object(col));
        }

        let fk_sql = format!("PRAGMA foreign_key_list(\"{}\")", table_name);
        let fk_result = conn
          .query(&fk_sql)
          .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let fk_cols = fk_result.column_names.clone();
        let from_idx  = fk_cols.iter().position(|c| c == "from").unwrap_or(3);
        let table_idx = fk_cols.iter().position(|c| c == "table").unwrap_or(2);
        let to_idx    = fk_cols.iter().position(|c| c == "to").unwrap_or(4);

        let mut fk_map: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
        for row_result in fk_result {
          let row = row_result.map_err(|e| napi::Error::from_reason(e.to_string()))?;
          if let (Value::Text(from_col), Value::Text(ref_table), Value::Text(ref_col)) =
            (&row[from_idx], &row[table_idx], &row[to_idx])
          {
            fk_map.insert(from_col.clone(), (ref_table.clone(), ref_col.clone()));
          }
        }

        for col in &mut columns {
          if let serde_json::Value::Object(ref mut map) = col {
            let col_name = match map.get("name") {
              Some(serde_json::Value::String(n)) => n.clone(),
              _ => continue,
            };
            if let Some((ref_table, ref_col)) = fk_map.get(&col_name) {
              let mut fk_obj = serde_json::Map::new();
              fk_obj.insert("table".to_string(), serde_json::Value::String(ref_table.clone()));
              fk_obj.insert("to".to_string(),    serde_json::Value::String(ref_col.clone()));
              map.insert("fk".to_string(), serde_json::Value::Object(fk_obj));
            } else {
              map.insert("fk".to_string(), serde_json::Value::Null);
            }
          }
        }

        let mut table_obj = serde_json::Map::new();
        table_obj.insert("name".to_string(),     serde_json::Value::String(table_name.clone()));
        table_obj.insert("rowCount".to_string(), serde_json::Value::Number(row_count.into()));
        table_obj.insert("columns".to_string(),  serde_json::Value::Array(columns));
        tables_json.push(serde_json::Value::Object(table_obj));
      }

      let mut schema = serde_json::Map::new();
      schema.insert("tables".to_string(), serde_json::Value::Array(tables_json));
      schema.insert("views".to_string(),  serde_json::Value::Array(Vec::new()));

      Ok(serde_json::to_string(&serde_json::Value::Object(schema)).unwrap())
    })
    .await
    .unwrap_or_else(|_| Err(napi::Error::from_reason("Background thread panicked".to_string())))
  }
}
