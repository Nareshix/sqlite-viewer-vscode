#![deny(clippy::all)]

use napi_derive::napi;
use sqlitex::traits::dynamic::Value;
use sqlitex::Connection;
use std::sync::Arc;

#[napi]
pub struct Database {
  // Update the type here to match what sqlitex returns
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
  pub fn query(&self, sql: String) -> napi::Result<String> {
    // Note: conn is an Arc, so we can call methods on it directly
    let results = self
      .conn
      .query(&sql)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let mut rows_json = Vec::new();
    let col_names = results.column_names.clone();

    for row_result in results {
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
    }

    Ok(serde_json::to_string(&rows_json).unwrap())
  }
  #[napi]
pub fn schema(&self) -> napi::Result<String> {
    // Get all user tables (exclude sqlite internal tables)
    let tables_result = self.conn
        .query("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let col_names = tables_result.column_names.clone();
    let mut table_names: Vec<String> = Vec::new();

    for row_result in tables_result {
        let row = row_result.map_err(|e| napi::Error::from_reason(e.to_string()))?;
        if let Value::Text(name) = &row[0] {
            table_names.push(name.clone());
        }
    }

    let mut tables_json = Vec::new();

    for table_name in &table_names {
        // Row count
        let count_sql = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
        let mut count_result = self.conn
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

        // Column info — PRAGMA returns: cid, name, type, notnull, dflt_value, pk
        let pragma_sql = format!("PRAGMA table_info(\"{}\")", table_name);
        let pragma_result = self.conn
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
                    Value::Text(v)    => serde_json::Value::String(v.clone()),
                    Value::Null       => serde_json::Value::Null,
                    Value::Real(v)    => serde_json::Value::Number(serde_json::Number::from_f64(*v).unwrap()),
                    Value::Blob(_)    => serde_json::Value::Null,
                };
                col.insert(col_name.clone(), val);
            }
            columns.push(serde_json::Value::Object(col));
        }

        let mut table_obj = serde_json::Map::new();
        table_obj.insert("name".to_string(),     serde_json::Value::String(table_name.clone()));
        table_obj.insert("rowCount".to_string(), serde_json::Value::Number(row_count.into()));
        table_obj.insert("columns".to_string(),  serde_json::Value::Array(columns));
        tables_json.push(serde_json::Value::Object(table_obj));
    }

    // Views
    let views_result = self.conn
        .query("SELECT name FROM sqlite_master WHERE type='view' ORDER BY name")
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let mut views_json = Vec::new();
    for row_result in views_result {
        let row = row_result.map_err(|e| napi::Error::from_reason(e.to_string()))?;
        if let Value::Text(name) = &row[0] {
            views_json.push(serde_json::Value::String(name.clone()));
        }
    }

    let mut schema = serde_json::Map::new();
    schema.insert("tables".to_string(), serde_json::Value::Array(tables_json));
    schema.insert("views".to_string(),  serde_json::Value::Array(views_json));

    Ok(serde_json::to_string(&serde_json::Value::Object(schema)).unwrap())
}
}

