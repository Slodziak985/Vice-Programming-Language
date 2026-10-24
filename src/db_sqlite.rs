use crate::value::{MapKey, Value};
use once_cell::sync::Lazy;
use rusqlite::types::{ToSqlOutput, ValueRef};
use rusqlite::{Connection, ToSql};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static NEXT_SQLITE_HANDLE: AtomicUsize = AtomicUsize::new(1);
static SQLITE_CONNS: Lazy<Mutex<HashMap<usize, Connection>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn instance(data_name: &str, fields: HashMap<String, Value>) -> Value {
    Value::Instance {
        data_name: data_name.to_string(),
        field_types: HashMap::new(),
        fields: Rc::new(RefCell::new(fields)),
    }
}

fn check_sandbox(path: &str) -> Result<(), String> {
    if let Some(root) = crate::stdlib::sandbox_root() {
        if path == ":memory:" {
            return Ok(());
        }
        let p = Path::new(path);
        let canonical = std::fs::canonicalize(p)
            .or_else(|_| {
                if let Some(parent) = p.parent() {
                    std::fs::canonicalize(parent).map(|cp| cp.join(p.file_name().unwrap()))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File not found",
                    ))
                }
            })
            .map_err(|e| format!("Cannot resolve database path '{}': {}", path, e))?;

        if !canonical.starts_with(&root) {
            return Err(format!(
                "Sandbox denied database access to '{}'; root is '{}'",
                canonical.display(),
                root.display()
            ));
        }
    }
    Ok(())
}

pub fn open(path: &str) -> Result<Value, String> {
    check_sandbox(path)?;
    let conn = if path == ":memory:" {
        Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory SQLite database: {}", e))?
    } else {
        Connection::open(path)
            .map_err(|e| format!("Failed to open SQLite database '{}': {}", path, e))?
    };

    let id = NEXT_SQLITE_HANDLE.fetch_add(1, Ordering::SeqCst);
    let mut map = SQLITE_CONNS
        .lock()
        .map_err(|_| "SQLite mutex poisoned".to_string())?;
    map.insert(id, conn);

    Ok(instance(
        "Handle",
        HashMap::from([
            ("id".to_string(), Value::Int(id as i64)),
            ("path".to_string(), Value::String(path.to_string())),
        ]),
    ))
}

pub fn close(handle_val: &Value) -> Result<Value, String> {
    let id = crate::ffi::extract_handle_id(handle_val)?;
    let mut map = SQLITE_CONNS
        .lock()
        .map_err(|_| "SQLite mutex poisoned".to_string())?;
    if map.remove(&id).is_some() {
        Ok(Value::Bool(true))
    } else {
        Err(format!("SQLite handle {} not found or already closed", id))
    }
}

struct SqlParam(Value);

impl ToSql for SqlParam {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match &self.0 {
            Value::Int(i) => Ok(ToSqlOutput::from(*i)),
            Value::Float(f) => Ok(ToSqlOutput::from(*f)),
            Value::String(s) => Ok(ToSqlOutput::from(s.as_str())),
            Value::Bool(b) => Ok(ToSqlOutput::from(*b)),
            Value::Unit => Ok(ToSqlOutput::from(rusqlite::types::Null)),
            _ => Ok(ToSqlOutput::from(crate::stdlib::value_to_string(&self.0))),
        }
    }
}

pub fn execute(handle_val: &Value, sql: &str, params: &[Value]) -> Result<Value, String> {
    let id = crate::ffi::extract_handle_id(handle_val)?;
    let map = SQLITE_CONNS
        .lock()
        .map_err(|_| "SQLite mutex poisoned".to_string())?;
    let conn = map
        .get(&id)
        .ok_or_else(|| format!("SQLite handle {} is invalid or closed", id))?;

    let sql_params: Vec<SqlParam> = params.iter().cloned().map(SqlParam).collect();
    let params_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p as &dyn ToSql).collect();

    let rows_affected = conn
        .execute(sql, params_refs.as_slice())
        .map_err(|e| format!("SQLite execute error: {}", e))?;

    Ok(Value::Int(rows_affected as i64))
}

pub fn query(handle_val: &Value, sql: &str, params: &[Value]) -> Result<Value, String> {
    let id = crate::ffi::extract_handle_id(handle_val)?;
    let map = SQLITE_CONNS
        .lock()
        .map_err(|_| "SQLite mutex poisoned".to_string())?;
    let conn = map
        .get(&id)
        .ok_or_else(|| format!("SQLite handle {} is invalid or closed", id))?;

    let sql_params: Vec<SqlParam> = params.iter().cloned().map(SqlParam).collect();
    let params_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p as &dyn ToSql).collect();

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("SQLite prepare error: {}", e))?;

    let col_names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows = stmt
        .query(params_refs.as_slice())
        .map_err(|e| format!("SQLite query error: {}", e))?;

    let mut result_rows = Vec::new();
    while let Some(row) = rows.next().map_err(|e| format!("SQLite row error: {}", e))? {
        let mut map_data = HashMap::new();
        for (i, name) in col_names.iter().enumerate() {
            let val_ref = row
                .get_ref(i)
                .map_err(|e| format!("SQLite column read error: {}", e))?;
            let vice_val = match val_ref {
                ValueRef::Null => Value::Unit,
                ValueRef::Integer(i) => Value::Int(i),
                ValueRef::Real(r) => Value::Float(r),
                ValueRef::Text(t) => {
                    let s = String::from_utf8_lossy(t).into_owned();
                    Value::String(s)
                }
                ValueRef::Blob(b) => {
                    let arr: Vec<Value> = b.iter().map(|&byte| Value::Int(byte as i64)).collect();
                    Value::Array(arr)
                }
            };
            map_data.insert(MapKey::String(name.clone()), vice_val);
        }
        result_rows.push(Value::Map(map_data));
    }

    Ok(Value::Array(result_rows))
}

pub fn last_insert_rowid(handle_val: &Value) -> Result<Value, String> {
    let id = crate::ffi::extract_handle_id(handle_val)?;
    let map = SQLITE_CONNS
        .lock()
        .map_err(|_| "SQLite mutex poisoned".to_string())?;
    let conn = map
        .get(&id)
        .ok_or_else(|| format!("SQLite handle {} is invalid or closed", id))?;
    Ok(Value::Int(conn.last_insert_rowid()))
}
