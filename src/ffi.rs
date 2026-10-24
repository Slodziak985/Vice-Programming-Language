use crate::value::Value;
use libloading::Library;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(1);

use once_cell::sync::Lazy;
static LIBS: Lazy<Mutex<HashMap<usize, Library>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn instance(data_name: &str, fields: HashMap<String, Value>) -> Value {
    Value::Instance {
        data_name: data_name.to_string(),
        field_types: HashMap::new(),
        fields: Rc::new(RefCell::new(fields)),
    }
}

fn check_sandbox() -> Result<(), String> {
    if crate::stdlib::sandbox_root().is_some() {
        return Err("C-FFI dynamic loading is disabled in sandbox mode".to_string());
    }
    Ok(())
}

pub fn load(path: &str) -> Result<Value, String> {
    check_sandbox()?;
    unsafe {
        let lib = Library::new(path)
            .map_err(|e| format!("Failed to load library '{}': {}", path, e))?;
        let id = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
        let mut map = LIBS.lock().map_err(|_| "FFI mutex poisoned".to_string())?;
        map.insert(id, lib);
        
        Ok(instance(
            "Handle",
            HashMap::from([
                ("id".to_string(), Value::Int(id as i64)),
                ("path".to_string(), Value::String(path.to_string())),
            ]),
        ))
    }
}

pub fn free(handle_val: &Value) -> Result<Value, String> {
    let id = extract_handle_id(handle_val)?;
    let mut map = LIBS.lock().map_err(|_| "FFI mutex poisoned".to_string())?;
    if map.remove(&id).is_some() {
        Ok(Value::Bool(true))
    } else {
        Err(format!("FFI handle {} not found or already closed", id))
    }
}

pub fn extract_handle_id(val: &Value) -> Result<usize, String> {
    match val {
        Value::Int(id) if *id > 0 => Ok(*id as usize),
        Value::Instance { data_name, fields, .. } if data_name == "Handle" => {
            let f = fields.borrow();
            match f.get("id") {
                Some(Value::Int(id)) if *id > 0 => Ok(*id as usize),
                _ => Err("Invalid FFI Handle object".to_string()),
            }
        }
        _ => Err("Expected FFI Handle".to_string()),
    }
}

pub fn call(
    handle_val: &Value,
    symbol_name: &str,
    args: &[Value],
    ret_type: &str,
) -> Result<Value, String> {
    check_sandbox()?;
    let id = extract_handle_id(handle_val)?;
    let map = LIBS.lock().map_err(|_| "FFI mutex poisoned".to_string())?;
    let lib = map
        .get(&id)
        .ok_or_else(|| format!("FFI handle {} is invalid or closed", id))?;

    unsafe {
        match args.len() {
            0 => match ret_type {
                "int" | "i32" => {
                    let func: libloading::Symbol<unsafe extern "C" fn() -> i32> = lib
                        .get(symbol_name.as_bytes())
                        .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                    Ok(Value::Int(func() as i64))
                }
                "i64" => {
                    let func: libloading::Symbol<unsafe extern "C" fn() -> i64> = lib
                        .get(symbol_name.as_bytes())
                        .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                    Ok(Value::Int(func()))
                }
                "float" | "f64" => {
                    let func: libloading::Symbol<unsafe extern "C" fn() -> f64> = lib
                        .get(symbol_name.as_bytes())
                        .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                    Ok(Value::Float(func()))
                }
                "void" => {
                    let func: libloading::Symbol<unsafe extern "C" fn()> = lib
                        .get(symbol_name.as_bytes())
                        .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                    func();
                    Ok(Value::Unit)
                }
                _ => Err(format!("Unsupported FFI return type: {}", ret_type)),
            },
            1 => {
                let a1 = match &args[0] {
                    Value::Int(i) => *i,
                    _ => return Err("FFI argument 1 must be integer".to_string()),
                };
                match ret_type {
                    "int" | "i32" => {
                        let func: libloading::Symbol<unsafe extern "C" fn(i32) -> i32> = lib
                            .get(symbol_name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                        Ok(Value::Int(func(a1 as i32) as i64))
                    }
                    "i64" => {
                        let func: libloading::Symbol<unsafe extern "C" fn(i64) -> i64> = lib
                            .get(symbol_name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                        Ok(Value::Int(func(a1)))
                    }
                    "void" => {
                        let func: libloading::Symbol<unsafe extern "C" fn(i64)> = lib
                            .get(symbol_name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                        func(a1);
                        Ok(Value::Unit)
                    }
                    _ => Err(format!("Unsupported FFI return type: {}", ret_type)),
                }
            }
            2 => {
                let a1 = match &args[0] {
                    Value::Int(i) => *i,
                    _ => return Err("FFI argument 1 must be integer".to_string()),
                };
                let a2 = match &args[1] {
                    Value::Int(i) => *i,
                    _ => return Err("FFI argument 2 must be integer".to_string()),
                };
                match ret_type {
                    "int" | "i32" => {
                        let func: libloading::Symbol<unsafe extern "C" fn(i32, i32) -> i32> = lib
                            .get(symbol_name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                        Ok(Value::Int(func(a1 as i32, a2 as i32) as i64))
                    }
                    "i64" => {
                        let func: libloading::Symbol<unsafe extern "C" fn(i64, i64) -> i64> = lib
                            .get(symbol_name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", symbol_name, e))?;
                        Ok(Value::Int(func(a1, a2)))
                    }
                    _ => Err(format!("Unsupported FFI return type: {}", ret_type)),
                }
            }
            _ => Err(format!("FFI call currently supports 0-2 arguments, got {}", args.len())),
        }
    }
}
