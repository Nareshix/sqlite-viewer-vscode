#![deny(clippy::all)]

use napi_derive::napi;
use sqlitex::Connection;
use sqlitex::traits::dynamic::Value;
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
    let results = self.conn.query(&sql).map_err(|e| napi::Error::from_reason(e.to_string()))?;

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
}