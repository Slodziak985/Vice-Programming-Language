use crate::ui::{UiButton, UiLabel, UiRect, UiRenderCommand, UiWindowConfig};
use crate::value::{MapKey, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const BUILTIN_MODULES: &[&str] = &[
    "time", "math", "ui", "game2d", "fs", "string", "array", "path", "env", "random", "json",
    "engine", "audio", "process", "result", "option", "cli", "map", "hash", "ffi", "http", "matrix",
    "crypto", "regex", "sqlite",
];

pub const JSON_MAX_INPUT_BYTES: usize = 1_048_576;
pub const JSON_MAX_DEPTH: usize = 64;
pub const JSON_MAX_NODES: usize = 50_000;

thread_local! {
    static PROCESS_ARGS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static SANDBOX_ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static SANDBOX_ALLOW_FS: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

/// Replaces the arguments exposed to Vice code through `process.args()` on this thread.
pub fn set_process_args(args: Vec<String>) {
    PROCESS_ARGS.with(|current| *current.borrow_mut() = args);
}

pub fn set_sandbox_root(root: Option<PathBuf>) -> Result<(), String> {
    let root =
        match root {
            Some(root) => Some(std::fs::canonicalize(&root).map_err(|error| {
                format!("cannot enable sandbox '{}': {}", root.display(), error)
            })?),
            None => None,
        };
    SANDBOX_ROOT.with(|current| *current.borrow_mut() = root);
    Ok(())
}

pub fn sandbox_root() -> Option<PathBuf> {
    SANDBOX_ROOT.with(|current| current.borrow().clone())
}

pub fn set_sandbox_allow_fs(paths: Vec<PathBuf>) -> Result<(), String> {
    let mut canonical_paths = Vec::with_capacity(paths.len());
    for path in paths {
        let canonical = std::fs::canonicalize(&path).map_err(|error| {
            format!("cannot add allowed fs path '{}': {}", path.display(), error)
        })?;
        canonical_paths.push(canonical);
    }
    SANDBOX_ALLOW_FS.with(|current| *current.borrow_mut() = canonical_paths);
    Ok(())
}

pub fn sandbox_allow_fs() -> Vec<PathBuf> {
    SANDBOX_ALLOW_FS.with(|current| current.borrow().clone())
}

pub fn json_to_vice(val: &serde_json::Value) -> Value {
    json_to_vice_checked(val).unwrap_or_else(|_| Value::Map(std::collections::HashMap::new()))
}

pub fn json_to_vice_checked(val: &serde_json::Value) -> Result<Value, String> {
    let mut budget = JsonBudget::default();
    json_to_vice_limited(val, 0, &mut budget)
}

pub fn vice_to_json(val: &Value) -> serde_json::Value {
    vice_to_json_checked(val).unwrap_or(serde_json::Value::Null)
}

pub fn vice_to_json_checked(val: &Value) -> Result<serde_json::Value, String> {
    let mut budget = JsonBudget::default();
    vice_to_json_limited(val, 0, &mut budget)
}

#[derive(Default)]
struct JsonBudget {
    nodes: usize,
}

fn json_budget_step(depth: usize, budget: &mut JsonBudget) -> Result<(), String> {
    if depth > JSON_MAX_DEPTH {
        return Err(format!(
            "JSON depth limit exceeded; max depth is {JSON_MAX_DEPTH}"
        ));
    }
    budget.nodes += 1;
    if budget.nodes > JSON_MAX_NODES {
        return Err(format!(
            "JSON node limit exceeded; max nodes is {JSON_MAX_NODES}"
        ));
    }
    Ok(())
}

fn check_json_source_limits(source: &str, context: &str) -> Result<(), String> {
    if source.len() > JSON_MAX_INPUT_BYTES {
        return Err(format!(
            "{} exceeds JSON input limit of {} bytes",
            context, JSON_MAX_INPUT_BYTES
        ));
    }
    Ok(())
}

fn parse_json_limited(source: &str, context: &str) -> Result<Value, String> {
    check_json_source_limits(source, context)?;
    let json_val = serde_json::from_str::<serde_json::Value>(source)
        .map_err(|error| format!("JSON parsing error: {}", error))?;
    json_to_vice_checked(&json_val)
}

fn stringify_json_limited(value: &Value, context: &str, pretty: bool) -> Result<String, String> {
    let json = vice_to_json_checked(value)?;
    let body = if pretty {
        serde_json::to_string_pretty(&json)
    } else {
        serde_json::to_string(&json)
    }
    .map_err(|error| format!("Failed to encode JSON for {context}: {error}"))?;
    if body.len() > JSON_MAX_INPUT_BYTES {
        return Err(format!(
            "{} exceeds JSON output limit of {} bytes",
            context, JSON_MAX_INPUT_BYTES
        ));
    }
    Ok(body)
}

fn json_to_vice_limited(
    val: &serde_json::Value,
    depth: usize,
    budget: &mut JsonBudget,
) -> Result<Value, String> {
    json_budget_step(depth, budget)?;
    match val {
        serde_json::Value::Null => Ok(Value::Map(std::collections::HashMap::new())), // Vice doesn't have Null, we map to empty Map like old interpreter
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Ok(Value::Int(0))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut res = Vec::new();
            for v in arr {
                res.push(json_to_vice_limited(v, depth + 1, budget)?);
            }
            Ok(Value::Array(res))
        }
        serde_json::Value::Object(obj) => {
            if let Some(serde_json::Value::String(type_name)) = obj.get("__type__") {
                let mut fields = std::collections::HashMap::new();
                for (k, v) in obj {
                    if k != "__type__" {
                        fields.insert(k.clone(), json_to_vice_limited(v, depth + 1, budget)?);
                    }
                }
                return Ok(Value::Instance {
                    data_name: type_name.clone(),
                    fields: std::rc::Rc::new(std::cell::RefCell::new(fields)),
                    field_types: std::collections::HashMap::new(),
                });
            }
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(
                    MapKey::String(k.clone()),
                    json_to_vice_limited(v, depth + 1, budget)?,
                );
            }
            Ok(Value::Map(map))
        }
    }
}

fn vice_to_json_limited(
    val: &Value,
    depth: usize,
    budget: &mut JsonBudget,
) -> Result<serde_json::Value, String> {
    json_budget_step(depth, budget)?;
    match val {
        Value::Int(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
        )),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Array(arr) => {
            let mut res = Vec::new();
            for v in arr {
                res.push(vice_to_json_limited(v, depth + 1, budget)?);
            }
            Ok(serde_json::Value::Array(res))
        }
        Value::Map(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.to_string(), vice_to_json_limited(v, depth + 1, budget)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        Value::Instance { fields, .. } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in &*fields.borrow() {
                obj.insert(k.clone(), vice_to_json_limited(v, depth + 1, budget)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        _ => Ok(serde_json::Value::Null),
    }
}

pub fn value_to_quoted_string(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        _ => value_to_string(value),
    }
}

pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => format!(
            "[{}]",
            arr.iter()
                .map(value_to_quoted_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Map(map) => {
            if map.is_empty() {
                "{}".to_string()
            } else {
                let mut entries: Vec<_> = map.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0)); // Sort keys for predictable output
                format!(
                    "{{{}}}",
                    entries
                        .iter()
                        .map(|(k, v)| {
                            format!("{}: {}", k.to_source_string(), value_to_quoted_string(v))
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        Value::Instance {
            data_name, fields, ..
        } => {
            let fields_borrow = fields.borrow();
            if fields_borrow.is_empty() {
                format!("{}()", data_name)
            } else {
                let mut entries: Vec<_> = fields_borrow.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0)); // Sort keys for predictable output
                format!(
                    "{}({})",
                    data_name,
                    entries
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, value_to_quoted_string(v)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        _ => format!("{:?}", value),
    }
}

pub fn as_int(val: &Value) -> i64 {
    match val {
        Value::Int(i) => *i,
        Value::Float(f) => *f as i64,
        _ => 0,
    }
}

pub fn as_float(val: &Value) -> f64 {
    match val {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => 0.0,
    }
}

fn instance(data_name: &str, fields: HashMap<String, Value>) -> Value {
    Value::Instance {
        data_name: data_name.to_string(),
        field_types: HashMap::new(),
        fields: std::rc::Rc::new(std::cell::RefCell::new(fields)),
    }
}

fn error_value(message: impl Into<String>, code: i64) -> Value {
    instance(
        "Error",
        HashMap::from([
            ("message".to_string(), Value::String(message.into())),
            ("code".to_string(), Value::Int(code)),
        ]),
    )
}

fn result_ok(value: Value) -> Value {
    instance(
        "Result",
        HashMap::from([
            ("is_ok".to_string(), Value::Bool(true)),
            ("value".to_string(), value),
            ("error".to_string(), Value::Unit),
        ]),
    )
}

fn result_err(message: impl Into<String>, code: i64) -> Value {
    instance(
        "Result",
        HashMap::from([
            ("is_ok".to_string(), Value::Bool(false)),
            ("value".to_string(), Value::Unit),
            ("error".to_string(), error_value(message, code)),
        ]),
    )
}

fn option_some(value: Value) -> Value {
    instance(
        "Option",
        HashMap::from([
            ("is_some".to_string(), Value::Bool(true)),
            ("value".to_string(), value),
        ]),
    )
}

fn option_none() -> Value {
    instance(
        "Option",
        HashMap::from([
            ("is_some".to_string(), Value::Bool(false)),
            ("value".to_string(), Value::Unit),
        ]),
    )
}

fn window_value(width: u32, height: u32, is_open: bool) -> Value {
    instance(
        "Window",
        HashMap::from([
            ("is_open".to_string(), Value::Bool(is_open)),
            ("width".to_string(), Value::Int(width as i64)),
            ("height".to_string(), Value::Int(height as i64)),
            ("dpi".to_string(), Value::Float(1.0)),
        ]),
    )
}

fn ui_response(clicked: bool, hovered: bool, active: bool, focused: bool, changed: bool) -> Value {
    instance(
        "UiResponse",
        HashMap::from([
            ("clicked".to_string(), Value::Bool(clicked)),
            ("hovered".to_string(), Value::Bool(hovered)),
            ("active".to_string(), Value::Bool(active)),
            ("focused".to_string(), Value::Bool(focused)),
            ("changed".to_string(), Value::Bool(changed)),
        ]),
    )
}

fn text_input_response(value: String, changed: bool, submitted: bool, focused: bool) -> Value {
    instance(
        "TextInputResponse",
        HashMap::from([
            ("value".to_string(), Value::String(value)),
            ("changed".to_string(), Value::Bool(changed)),
            ("submitted".to_string(), Value::Bool(submitted)),
            ("focused".to_string(), Value::Bool(focused)),
        ]),
    )
}

fn canvas_value(id: String, width: u32, height: u32) -> Value {
    instance(
        "Canvas",
        HashMap::from([
            ("id".to_string(), Value::String(id)),
            ("width".to_string(), Value::Int(width as i64)),
            ("height".to_string(), Value::Int(height as i64)),
        ]),
    )
}

fn arg_string(args: &[Value], index: usize, default: &str) -> String {
    args.get(index)
        .map(value_to_string)
        .unwrap_or_else(|| default.to_string())
}

fn fs_read_path(path: &str) -> Result<PathBuf, String> {
    let candidate = path_to_candidate(path);
    match sandbox_root() {
        Some(root) => {
            let canonical = std::fs::canonicalize(&candidate)
                .map_err(|error| format!("Failed to access {}: {}", path, error))?;
            if sandbox_allows_path(&canonical, &root) {
                Ok(canonical)
            } else {
                Err(format!(
                    "Sandbox denied access to '{}'; root is '{}'; allowed fs roots: {}",
                    path,
                    root.display(),
                    sandbox_allowed_display()
                ))
            }
        }
        None => Ok(candidate),
    }
}

fn fs_write_path(path: &str) -> Result<PathBuf, String> {
    let candidate = path_to_candidate(path);
    match sandbox_root() {
        Some(root) => {
            let parent = candidate
                .parent()
                .ok_or_else(|| format!("Failed to resolve parent for {}", path))?;
            let canonical_parent = canonical_existing_parent(parent)?;
            if sandbox_allows_path(&canonical_parent, &root) {
                Ok(candidate)
            } else {
                Err(format!(
                    "Sandbox denied write to '{}'; root is '{}'; allowed fs roots: {}",
                    path,
                    root.display(),
                    sandbox_allowed_display()
                ))
            }
        }
        None => Ok(candidate),
    }
}

fn sandbox_allows_path(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
        || SANDBOX_ALLOW_FS.with(|allowed| {
            allowed
                .borrow()
                .iter()
                .any(|allowed_root| path.starts_with(allowed_root))
        })
}

fn sandbox_allowed_display() -> String {
    SANDBOX_ALLOW_FS.with(|allowed| {
        let allowed = allowed.borrow();
        if allowed.is_empty() {
            "<none>".to_string()
        } else {
            allowed
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("; ")
        }
    })
}

fn canonical_existing_parent(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return std::fs::canonicalize(path)
            .map_err(|error| format!("Failed to resolve {}: {}", path.display(), error));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("Failed to resolve {}", path.display()))?;
    canonical_existing_parent(parent)
}

fn path_to_candidate(path: &str) -> PathBuf {
    let raw = Path::new(path);
    if raw.is_absolute() {
        raw.to_path_buf()
    } else if let Some(root) = sandbox_root() {
        root.join(raw)
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(raw)
    }
}

fn arg_pair(args: &[Value], index: usize, default: [i64; 2]) -> Result<[i64; 2], String> {
    match args.get(index) {
        Some(Value::Array(items)) if items.len() >= 2 => Ok([as_int(&items[0]), as_int(&items[1])]),
        Some(_) => Err("expected [x, y] array".to_string()),
        None => Ok(default),
    }
}

fn positive_size(args: &[Value], index: usize, default: [u32; 2]) -> Result<[u32; 2], String> {
    let pair = arg_pair(args, index, [default[0] as i64, default[1] as i64])?;
    if pair[0] <= 0 || pair[1] <= 0 {
        return Err("expects positive dimensions".to_string());
    }
    Ok([pair[0] as u32, pair[1] as u32])
}

fn arg_color(args: &[Value], index: usize, default: [u8; 4]) -> Result<[u8; 4], String> {
    match args.get(index) {
        Some(value) => {
            let text = value_to_string(value);
            crate::ui::parse_hex_color(&text)
                .ok_or_else(|| "color must be #rrggbb or #rrggbbaa".to_string())
        }
        None => Ok(default),
    }
}

fn require_id(args: &[Value], index: usize, widget: &str) -> Result<String, String> {
    let id = arg_string(args, index, "");
    if id.trim().is_empty() {
        Err(format!("ui.{} requires a stable non-empty id", widget))
    } else {
        Ok(id)
    }
}

fn canvas_arg_offset(args: &[Value]) -> usize {
    if matches!(
        args.first(),
        Some(Value::Instance {
            data_name,
            ..
        }) if data_name == "Canvas"
    ) {
        1
    } else {
        0
    }
}

fn time_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

pub fn call_native_vm(module: &str, method: &str, mut args: Vec<Value>) -> Result<Value, String> {
    match (module, method) {
        // --- TIME ---
        ("time", "now_ms") => {
            let since_the_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| "System clock is before the Unix epoch".to_string())?;
            Ok(Value::Int(since_the_epoch.as_millis() as i64))
        }

        // --- MATH ---
        ("math", "abs") => {
            if args.is_empty() {
                return Err("math.abs requires 1 argument".into());
            }
            match args.pop() {
                Some(Value::Int(i)) => Ok(Value::Int(i.wrapping_abs())),
                Some(Value::Float(f)) => Ok(Value::Float(f.abs())),
                _ => Err("math.abs requires a number".into()),
            }
        }
        ("math", "sin") => {
            if args.is_empty() {
                return Err("math.sin requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).sin()))
        }
        ("math", "cos") => {
            if args.is_empty() {
                return Err("math.cos requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).cos()))
        }
        ("math", "tan") => {
            if args.is_empty() {
                return Err("math.tan requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).tan()))
        }
        ("math", "sqrt") => {
            if args.is_empty() {
                return Err("math.sqrt requires 1 argument".into());
            }
            let val = as_float(&args[0]);
            if val < 0.0 {
                return Err("math.sqrt domain error: argument must be non-negative".into());
            }
            Ok(Value::Float(val.sqrt()))
        }
        ("math", "round") => {
            if args.is_empty() {
                return Err("math.round requires 1 argument".into());
            }
            Ok(Value::Int(as_float(&args[0]).round() as i64))
        }
        ("math", "floor") => {
            if args.is_empty() {
                return Err("math.floor requires 1 argument".into());
            }
            Ok(Value::Int(as_float(&args[0]).floor() as i64))
        }
        ("math", "ceil") => {
            if args.is_empty() {
                return Err("math.ceil requires 1 argument".into());
            }
            Ok(Value::Int(as_float(&args[0]).ceil() as i64))
        }
        ("math", "pow") => {
            if args.len() < 2 {
                return Err("math.pow requires 2 arguments".into());
            }
            Ok(Value::Float(as_float(&args[0]).powf(as_float(&args[1]))))
        }
        ("math", "random") => {
            // Pseudo-random based on time
            let time = time_nanos();
            let val = (time % 1000000) as f64 / 1000000.0;
            Ok(Value::Float(val))
        }
        ("math", "min") => {
            if args.len() < 2 {
                return Err("math.min requires 2 arguments".into());
            }
            let a = &args[0];
            let b = &args[1];
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int((*x).min(*y))),
                (Value::Float(x), Value::Float(y)) => Ok(Value::Float((*x).min(*y))),
                (Value::Int(x), Value::Float(y)) => Ok(Value::Float((*x as f64).min(*y))),
                (Value::Float(x), Value::Int(y)) => Ok(Value::Float((*x).min(*y as f64))),
                _ => Err("math.min requires numbers".into()),
            }
        }
        ("math", "max") => {
            if args.len() < 2 {
                return Err("math.max requires 2 arguments".into());
            }
            let a = &args[0];
            let b = &args[1];
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int((*x).max(*y))),
                (Value::Float(x), Value::Float(y)) => Ok(Value::Float((*x).max(*y))),
                (Value::Int(x), Value::Float(y)) => Ok(Value::Float((*x as f64).max(*y))),
                (Value::Float(x), Value::Int(y)) => Ok(Value::Float((*x).max(*y as f64))),
                _ => Err("math.max requires numbers".into()),
            }
        }
        ("math", "clamp") => {
            if args.len() < 3 {
                return Err("math.clamp requires 3 arguments (val, min, max)".into());
            }
            match (&args[0], &args[1], &args[2]) {
                (Value::Int(v), Value::Int(min), Value::Int(max)) => {
                    if min > max {
                        return Err("math.clamp: min cannot be greater than max".into());
                    }
                    Ok(Value::Int((*v).max(*min).min(*max)))
                }
                (Value::Float(v), Value::Float(min), Value::Float(max)) => {
                    if min > max {
                        return Err("math.clamp: min cannot be greater than max".into());
                    }
                    Ok(Value::Float((*v).max(*min).min(*max)))
                }
                (Value::Int(v), Value::Float(min), Value::Float(max)) => {
                    let v_f = *v as f64;
                    if min > max {
                        return Err("math.clamp: min cannot be greater than max".into());
                    }
                    Ok(Value::Float(v_f.max(*min).min(*max)))
                }
                (Value::Float(v), Value::Int(min), Value::Int(max)) => {
                    let min_f = *min as f64;
                    let max_f = *max as f64;
                    if min > max {
                        return Err("math.clamp: min cannot be greater than max".into());
                    }
                    Ok(Value::Float((*v).max(min_f).min(max_f)))
                }
                _ => Err("math.clamp requires numbers".into()),
            }
        }
        ("math", "sign") => {
            if args.is_empty() {
                return Err("math.sign requires 1 argument".into());
            }
            match &args[0] {
                Value::Int(i) => {
                    if *i > 0 {
                        Ok(Value::Int(1))
                    } else if *i < 0 {
                        Ok(Value::Int(-1))
                    } else {
                        Ok(Value::Int(0))
                    }
                }
                Value::Float(f) => {
                    if *f > 0.0 {
                        Ok(Value::Int(1))
                    } else if *f < 0.0 {
                        Ok(Value::Int(-1))
                    } else {
                        Ok(Value::Int(0))
                    }
                }
                _ => Err("math.sign requires a number".into()),
            }
        }
        ("math", "log") => {
            if args.is_empty() {
                return Err("math.log requires 1 argument".into());
            }
            let val = as_float(&args[0]);
            if val <= 0.0 {
                return Err("math.log domain error: argument must be positive".into());
            }
            Ok(Value::Float(val.ln()))
        }
        ("math", "log10") => {
            if args.is_empty() {
                return Err("math.log10 requires 1 argument".into());
            }
            let val = as_float(&args[0]);
            if val <= 0.0 {
                return Err("math.log10 domain error: argument must be positive".into());
            }
            Ok(Value::Float(val.log10()))
        }
        ("math", "log2") => {
            if args.is_empty() {
                return Err("math.log2 requires 1 argument".into());
            }
            let val = as_float(&args[0]);
            if val <= 0.0 {
                return Err("math.log2 domain error: argument must be positive".into());
            }
            Ok(Value::Float(val.log2()))
        }
        ("math", "hypot") => {
            if args.len() < 2 {
                return Err("math.hypot requires 2 arguments (x, y)".into());
            }
            let x = as_float(&args[0]);
            let y = as_float(&args[1]);
            Ok(Value::Float(x.hypot(y)))
        }
        ("math", "atan2") => {
            if args.len() < 2 {
                return Err("math.atan2 requires 2 arguments (y, x)".into());
            }
            let y = as_float(&args[0]);
            let x = as_float(&args[1]);
            Ok(Value::Float(y.atan2(x)))
        }
        ("math", "asin") => {
            if args.is_empty() {
                return Err("math.asin requires 1 argument".into());
            }
            let val = as_float(&args[0]);
            if !(-1.0..=1.0).contains(&val) {
                return Err("math.asin domain error: argument must be between -1.0 and 1.0".into());
            }
            Ok(Value::Float(val.asin()))
        }
        ("math", "acos") => {
            if args.is_empty() {
                return Err("math.acos requires 1 argument".into());
            }
            let val = as_float(&args[0]);
            if !(-1.0..=1.0).contains(&val) {
                return Err("math.acos domain error: argument must be between -1.0 and 1.0".into());
            }
            Ok(Value::Float(val.acos()))
        }
        ("math", "pi") => Ok(Value::Float(std::f64::consts::PI)),
        ("math", "e") => Ok(Value::Float(std::f64::consts::E)),
        ("math", "deg") => {
            if args.is_empty() {
                return Err("math.deg requires 1 argument (radians)".into());
            }
            Ok(Value::Float(as_float(&args[0]).to_degrees()))
        }
        ("math", "rad") => {
            if args.is_empty() {
                return Err("math.rad requires 1 argument (degrees)".into());
            }
            Ok(Value::Float(as_float(&args[0]).to_radians()))
        }
        ("math", "signum") => {
            if args.is_empty() {
                return Err("math.signum requires 1 argument".into());
            }
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(if *i > 0 { 1 } else if *i < 0 { -1 } else { 0 })),
                Value::Float(f) => Ok(Value::Float(if *f > 0.0 { 1.0 } else if *f < 0.0 { -1.0 } else { 0.0 })),
                _ => Ok(Value::Int(0)),
            }
        }
        ("math", "trunc") => {
            if args.is_empty() {
                return Err("math.trunc requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).trunc()))
        }
        ("math", "fract") => {
            if args.is_empty() {
                return Err("math.fract requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).fract()))
        }
        ("math", "cbrt") => {
            if args.is_empty() {
                return Err("math.cbrt requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).cbrt()))
        }
        ("math", "gcd") => {
            if args.len() < 2 {
                return Err("math.gcd requires 2 arguments (a, b)".into());
            }
            let a = as_int(&args[0]);
            let b = as_int(&args[1]);
            Ok(Value::Int(crate::vm::compiler::gcd_i64(a, b)))
        }
        ("math", "lcm") => {
            if args.len() < 2 {
                return Err("math.lcm requires 2 arguments (a, b)".into());
            }
            let a = as_int(&args[0]);
            let b = as_int(&args[1]);
            Ok(Value::Int(crate::vm::compiler::lcm_i64(a, b)))
        }
        ("math", "is_nan") => {
            if args.is_empty() {
                return Err("math.is_nan requires 1 argument".into());
            }
            Ok(Value::Bool(as_float(&args[0]).is_nan()))
        }
        ("math", "is_finite") => {
            if args.is_empty() {
                return Err("math.is_finite requires 1 argument".into());
            }
            Ok(Value::Bool(as_float(&args[0]).is_finite()))
        }
        ("math", "radians") => {
            if args.is_empty() {
                return Err("math.radians requires 1 argument (deg)".into());
            }
            Ok(Value::Float(as_float(&args[0]).to_radians()))
        }
        ("math", "degrees") => {
            if args.is_empty() {
                return Err("math.degrees requires 1 argument (rad)".into());
            }
            Ok(Value::Float(as_float(&args[0]).to_degrees()))
        }
        ("math", "factorial") => {
            if args.is_empty() {
                return Err("math.factorial requires 1 argument (n)".into());
            }
            let n = as_int(&args[0]);
            if n < 0 {
                return Err("math.factorial: argument cannot be negative".into());
            }
            if n > 20 {
                return Err("math.factorial: argument exceeds 20 (integer overflow limit)".into());
            }
            let mut acc: i64 = 1;
            for k in 2..=n {
                acc = acc.saturating_mul(k);
            }
            Ok(Value::Int(acc))
        }
        ("math", "round_to") => {
            if args.len() < 2 {
                return Err("math.round_to requires 2 arguments (val, decimals)".into());
            }
            let val = as_float(&args[0]);
            let dec = as_int(&args[1]).clamp(0, 10) as u32;
            let factor = 10f64.powi(dec as i32);
            Ok(Value::Float((val * factor).round() / factor))
        }
        ("math", "lerp") => {
            if args.len() < 3 {
                return Err("math.lerp requires 3 arguments (a, b, t)".into());
            }
            let a = as_float(&args[0]);
            let b = as_float(&args[1]);
            let t = as_float(&args[2]);
            Ok(Value::Float(a + (b - a) * t))
        }
        ("math", "exp") => {
            if args.is_empty() {
                return Err("math.exp requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).exp()))
        }
        ("math", "is_even") => {
            if args.is_empty() {
                return Err("math.is_even requires 1 argument".into());
            }
            Ok(Value::Bool(as_int(&args[0]) % 2 == 0))
        }
        ("math", "is_square") => {
            if args.is_empty() {
                return Err("math.is_square requires 1 argument (n)".into());
            }
            let n = as_int(&args[0]);
            Ok(Value::Bool(is_integer_square(n)))
        }
        ("math", "is_odd") => {
            if args.is_empty() {
                return Err("math.is_odd requires 1 argument".into());
            }
            Ok(Value::Bool(as_int(&args[0]) % 2 != 0))
        }
        ("math", "bitwise_and") => {
            if args.len() < 2 {
                return Err("math.bitwise_and requires 2 arguments".into());
            }
            Ok(Value::Int(as_int(&args[0]) & as_int(&args[1])))
        }
        ("math", "bitwise_or") => {
            if args.len() < 2 {
                return Err("math.bitwise_or requires 2 arguments".into());
            }
            Ok(Value::Int(as_int(&args[0]) | as_int(&args[1])))
        }
        ("math", "bitwise_xor") => {
            if args.len() < 2 {
                return Err("math.bitwise_xor requires 2 arguments".into());
            }
            Ok(Value::Int(as_int(&args[0]) ^ as_int(&args[1])))
        }
        ("math", "bitwise_not") => {
            if args.is_empty() {
                return Err("math.bitwise_not requires 1 argument".into());
            }
            Ok(Value::Int(!as_int(&args[0])))
        }
        ("math", "bitwise_shl") => {
            if args.len() < 2 {
                return Err("math.bitwise_shl requires 2 arguments".into());
            }
            let a = as_int(&args[0]);
            let shift = (as_int(&args[1]) & 63) as u32;
            Ok(Value::Int(a.wrapping_shl(shift)))
        }
        ("math", "bitwise_shr") => {
            if args.len() < 2 {
                return Err("math.bitwise_shr requires 2 arguments".into());
            }
            let a = as_int(&args[0]);
            let shift = (as_int(&args[1]) & 63) as u32;
            Ok(Value::Int(a.wrapping_shr(shift)))
        }
        ("math", "wrap") => {
            if args.len() < 3 {
                return Err("math.wrap requires 3 arguments (val, min, max)".into());
            }
            let is_float = matches!(&args[0], Value::Float(_))
                || matches!(&args[1], Value::Float(_))
                || matches!(&args[2], Value::Float(_));
            if is_float {
                let v = as_float(&args[0]);
                let min = as_float(&args[1]);
                let max = as_float(&args[2]);
                let range = max - min;
                if range <= 0.0 {
                    return Ok(Value::Float(min));
                }
                let offset = ((v - min) % range + range) % range;
                Ok(Value::Float(min + offset))
            } else {
                let v = as_int(&args[0]);
                let min = as_int(&args[1]);
                let max = as_int(&args[2]);
                let range = max - min;
                if range <= 0 {
                    return Ok(Value::Int(min));
                }
                let offset = ((v - min) % range + range) % range;
                Ok(Value::Int(min + offset))
            }
        }
        ("math", "smoothstep") => {
            if args.len() < 3 {
                return Err("math.smoothstep requires 3 arguments (edge0, edge1, x)".into());
            }
            let edge0 = as_float(&args[0]);
            let edge1 = as_float(&args[1]);
            let x = as_float(&args[2]);
            if (edge1 - edge0).abs() < 1e-12 {
                return Ok(Value::Float(if x >= edge1 { 1.0 } else { 0.0 }));
            }
            let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
            Ok(Value::Float(t * t * (3.0 - 2.0 * t)))
        }
        ("math", "deg_to_rad") => {
            if args.is_empty() {
                return Err("math.deg_to_rad requires 1 argument (degrees)".into());
            }
            let deg = as_float(&args[0]);
            Ok(Value::Float(deg.to_radians()))
        }
        ("math", "rad_to_deg") => {
            if args.is_empty() {
                return Err("math.rad_to_deg requires 1 argument (radians)".into());
            }
            let rad = as_float(&args[0]);
            Ok(Value::Float(rad.to_degrees()))
        }
        ("math", "cosh") => {
            if args.is_empty() {
                return Err("math.cosh requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).cosh()))
        }
        ("math", "sinh") => {
            if args.is_empty() {
                return Err("math.sinh requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).sinh()))
        }
        ("math", "tanh") => {
            if args.is_empty() {
                return Err("math.tanh requires 1 argument".into());
            }
            Ok(Value::Float(as_float(&args[0]).tanh()))
        }
        ("math", "copysign") => {
            if args.len() < 2 {
                return Err("math.copysign requires 2 arguments (x, y)".into());
            }
            let a = as_float(&args[0]);
            let b = as_float(&args[1]);
            Ok(Value::Float(a.copysign(b)))
        }
        ("math", "dist") => {
            if args.len() < 4 {
                return Err("math.dist requires 4 arguments (x1, y1, x2, y2)".into());
            }
            let x1 = as_float(&args[0]);
            let y1 = as_float(&args[1]);
            let x2 = as_float(&args[2]);
            let y2 = as_float(&args[3]);
            let dx = x2 - x1;
            let dy = y2 - y1;
            Ok(Value::Float(dx.hypot(dy)))
        }
        ("math", "is_prime") => {
            if args.is_empty() {
                return Err("math.is_prime requires 1 argument (n)".into());
            }
            let n = as_int(&args[0]);
            if n <= 1 {
                return Ok(Value::Bool(false));
            }
            if n <= 3 {
                return Ok(Value::Bool(true));
            }
            if n % 2 == 0 || n % 3 == 0 {
                return Ok(Value::Bool(false));
            }
            let mut i = 5i64;
            let mut is_p = true;
            while i * i <= n {
                if n % i == 0 || n % (i + 2) == 0 {
                    is_p = false;
                    break;
                }
                i += 6;
            }
            Ok(Value::Bool(is_p))
        }
        ("math", "count_ones") => {
            if args.is_empty() {
                return Err("math.count_ones requires 1 argument (n)".into());
            }
            let n = as_int(&args[0]);
            Ok(Value::Int(n.count_ones() as i64))
        }
        ("math", "leading_zeros") => {
            if args.is_empty() {
                return Err("math.leading_zeros requires 1 argument (n)".into());
            }
            let n = as_int(&args[0]);
            Ok(Value::Int(n.leading_zeros() as i64))
        }
        ("math", "trailing_zeros") => {
            if args.is_empty() {
                return Err("math.trailing_zeros requires 1 argument (n)".into());
            }
            let n = as_int(&args[0]);
            Ok(Value::Int(n.trailing_zeros() as i64))
        }
        ("math", "is_power_of_two") => {
            if args.is_empty() {
                return Err("math.is_power_of_two requires 1 argument (n)".into());
            }
            let n = as_int(&args[0]);
            Ok(Value::Bool(n > 0 && (n & (n - 1)) == 0))
        }
        ("math", "next_power_of_two") => {
            if args.is_empty() {
                return Err("math.next_power_of_two requires 1 argument (n)".into());
            }
            let n = as_int(&args[0]);
            if n <= 1 {
                return Ok(Value::Int(1));
            }
            let u = n as u64;
            let next = u.next_power_of_two();
            Ok(Value::Int(next as i64))
        }
        ("math", "clamp01") => {
            if args.is_empty() {
                return Err("math.clamp01 requires 1 argument".into());
            }
            match &args[0] {
                Value::Int(i) => Ok(Value::Int((*i).clamp(0, 1))),
                Value::Float(f) => Ok(Value::Float(clamp01_f64(*f))),
                _ => Err("math.clamp01 requires a number".into()),
            }
        }
        ("math", "lerp_inv") => {
            if args.len() < 3 {
                return Err("math.lerp_inv requires 3 arguments (a, b, v)".into());
            }
            let a = as_float(&args[0]);
            let b = as_float(&args[1]);
            let v = as_float(&args[2]);
            Ok(Value::Float(lerp_inv_f64(a, b, v)))
        }
        ("math", "remap") => {
            if args.len() < 5 {
                return Err("math.remap requires 5 arguments (v, in_min, in_max, out_min, out_max)".into());
            }
            let v = as_float(&args[0]);
            let in_min = as_float(&args[1]);
            let in_max = as_float(&args[2]);
            let out_min = as_float(&args[3]);
            let out_max = as_float(&args[4]);
            Ok(Value::Float(remap_f64(v, in_min, in_max, out_min, out_max)))
        }
        ("math", "midpoint") => {
            if args.len() < 2 {
                return Err("math.midpoint requires 2 arguments (a, b)".into());
            }
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(midpoint_i64(*a, *b))),
                _ => {
                    let a = as_float(&args[0]);
                    let b = as_float(&args[1]);
                    Ok(Value::Float((a + b) / 2.0))
                }
            }
        }
        ("math", "approx_eq") => {
            if args.len() < 2 {
                return Err("math.approx_eq requires at least 2 arguments (a, b, [eps])".into());
            }
            let a = as_float(&args[0]);
            let b = as_float(&args[1]);
            let eps = if args.len() > 2 { as_float(&args[2]) } else { 1e-6 };
            Ok(Value::Bool(approx_eq_f64(a, b, eps)))
        }
        ("math", "hypot3") => {
            if args.len() < 3 {
                return Err("math.hypot3 requires 3 arguments (x, y, z)".into());
            }
            let x = as_float(&args[0]);
            let y = as_float(&args[1]);
            let z = as_float(&args[2]);
            Ok(Value::Float(hypot3_f64(x, y, z)))
        }
        ("math", "clamp_int") => {
            if args.len() < 3 {
                return Err("math.clamp_int requires 3 arguments (v, min, max)".into());
            }
            let v = as_int(&args[0]);
            let min_v = as_int(&args[1]);
            let max_v = as_int(&args[2]);
            Ok(Value::Int(clamp_int_i64(v, min_v, max_v)))
        }

        // --- RANDOM ---
        ("random", "int") => {
            if args.len() < 2 {
                return Err("random.int requires 2 arguments".into());
            }
            let min = as_int(&args[0]);
            let max = as_int(&args[1]);
            if min >= max {
                return Ok(Value::Int(min));
            }
            let time = time_nanos();
            let range = (max as i128 - min as i128 + 1) as u128;
            let value = min as i128 + (time % range) as i128;
            Ok(Value::Int(value as i64))
        }
        ("random", "float") => {
            let time = time_nanos();
            let val = (time % 1000000) as f64 / 1000000.0;
            Ok(Value::Float(val))
        }

        // --- FS ---
        ("fs", "read_text") => {
            if args.is_empty() {
                return Err("fs.read_text requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            let path = fs_read_path(&path)?;
            match std::fs::read_to_string(&path) {
                Ok(content) => Ok(Value::String(content)),
                Err(e) => Err(format!("Failed to read {}: {}", path.display(), e)),
            }
        }
        ("fs", "try_read_text") => {
            if args.is_empty() {
                return Err("fs.try_read_text requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            Ok(match fs_read_path(&path) {
                Ok(path) => match std::fs::read_to_string(&path) {
                    Ok(content) => result_ok(Value::String(content)),
                    Err(error) => {
                        result_err(format!("Failed to read {}: {}", path.display(), error), 1)
                    }
                },
                Err(error) => result_err(error, 1),
            })
        }
        ("fs", "write_text") => {
            if args.len() < 2 {
                return Err("fs.write_text requires 2 arguments".into());
            }
            let path = value_to_string(&args[0]);
            let content = value_to_string(&args[1]);
            let path = fs_write_path(&path)?;
            match std::fs::write(&path, content) {
                Ok(_) => Ok(Value::Bool(true)),
                Err(e) => Err(format!("Failed to write {}: {}", path.display(), e)),
            }
        }
        ("fs", "try_write_text") => {
            if args.len() < 2 {
                return Err("fs.try_write_text requires 2 arguments".into());
            }
            let path = value_to_string(&args[0]);
            let content = value_to_string(&args[1]);
            Ok(match fs_write_path(&path) {
                Ok(path) => match std::fs::write(&path, content) {
                    Ok(_) => result_ok(Value::Bool(true)),
                    Err(error) => {
                        result_err(format!("Failed to write {}: {}", path.display(), error), 1)
                    }
                },
                Err(error) => result_err(error, 1),
            })
        }
        ("fs", "read_json") => {
            if args.is_empty() {
                return Err("fs.read_json requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            let path = fs_read_path(&path)?;
            let content = std::fs::read_to_string(&path)
                .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
            parse_json_limited(&content, &format!("JSON file {}", path.display()))
        }
        ("fs", "write_json") => {
            if args.len() < 2 {
                return Err("fs.write_json requires 2 arguments".into());
            }
            let path = value_to_string(&args[0]);
            let path = fs_write_path(&path)?;
            let json =
                stringify_json_limited(&args[1], &format!("JSON file {}", path.display()), true)?;
            std::fs::write(&path, format!("{}\n", json))
                .map(|_| Value::Bool(true))
                .map_err(|error| format!("Failed to write {}: {}", path.display(), error))
        }
        ("fs", "append_text") => {
            use std::io::Write;

            if args.len() < 2 {
                return Err("fs.append_text requires 2 arguments".into());
            }
            let path = value_to_string(&args[0]);
            let content = value_to_string(&args[1]);
            let path = fs_write_path(&path)?;
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .and_then(|mut file| file.write_all(content.as_bytes()))
                .map(|_| Value::Bool(true))
                .map_err(|error| format!("Failed to append {}: {}", path.display(), error))
        }
        ("fs", "list_dir") => {
            if args.is_empty() {
                return Err("fs.list_dir requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            let path = fs_read_path(&path)?;
            let mut names = std::fs::read_dir(&path)
                .map_err(|error| format!("Failed to list {}: {}", path.display(), error))?
                .map(|entry| {
                    entry
                        .map(|entry| entry.file_name().to_string_lossy().to_string())
                        .map_err(|error| format!("Failed to list {}: {}", path.display(), error))
                })
                .collect::<Result<Vec<_>, _>>()?;
            names.sort();
            Ok(Value::Array(names.into_iter().map(Value::String).collect()))
        }
        ("fs", "create_dir") => {
            if args.is_empty() {
                return Err("fs.create_dir requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            let path = fs_write_path(&path)?;
            std::fs::create_dir_all(&path)
                .map(|_| Value::Bool(true))
                .map_err(|error| {
                    format!("Failed to create directory {}: {}", path.display(), error)
                })
        }

        // --- STRING ---
        ("string", "contains") => {
            if args.len() < 2 {
                return Err("string.contains requires 2 arguments".into());
            }
            let s = value_to_string(&args[0]);
            let sub = value_to_string(&args[1]);
            Ok(Value::Bool(s.contains(&sub)))
        }
        ("string", "starts_with") => {
            if args.len() < 2 {
                return Err("string.starts_with requires 2 arguments".into());
            }
            let s = value_to_string(&args[0]);
            let sub = value_to_string(&args[1]);
            Ok(Value::Bool(s.starts_with(&sub)))
        }
        ("string", "ends_with") => {
            if args.len() < 2 {
                return Err("string.ends_with requires 2 arguments".into());
            }
            let s = value_to_string(&args[0]);
            let sub = value_to_string(&args[1]);
            Ok(Value::Bool(s.ends_with(&sub)))
        }
        ("string", "len") => {
            if args.is_empty() {
                return Err("string.len requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::Int(s.chars().count() as i64))
        }
        ("string", "split") => {
            if args.len() < 2 {
                return Err("string.split requires 2 arguments".into());
            }
            let s = value_to_string(&args[0]);
            let delim = value_to_string(&args[1]);
            let parts = s
                .split(&delim)
                .map(|x| Value::String(x.to_string()))
                .collect();
            Ok(Value::Array(parts))
        }
        ("string", "trim") => {
            if args.is_empty() {
                return Err("string.trim requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(s.trim().to_string()))
        }
        ("string", "lower") | ("string", "to_lower") => {
            if args.is_empty() {
                return Err("string.lower requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(s.to_lowercase()))
        }
        ("string", "upper") | ("string", "to_upper") => {
            if args.is_empty() {
                return Err("string.upper requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(s.to_uppercase()))
        }
        ("string", "replace") => {
            if args.len() < 3 {
                return Err("string.replace requires 3 arguments".into());
            }
            let s = value_to_string(&args[0]);
            let from = value_to_string(&args[1]);
            let to = value_to_string(&args[2]);
            Ok(Value::String(s.replace(&from, &to)))
        }
        ("string", "is_empty") => {
            if args.is_empty() {
                return Err("string.is_empty requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::Bool(s.is_empty()))
        }
        ("string", "slice") => {
            if args.len() < 3 {
                return Err("string.slice requires 3 arguments (s, start, end)".into());
            }
            let s = value_to_string(&args[0]);
            let start = as_int(&args[1]);
            let end = as_int(&args[2]);
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let start_idx = start.max(0).min(len) as usize;
            let end_idx = end.max(0).min(len) as usize;
            let sliced: String = if start_idx <= end_idx {
                chars[start_idx..end_idx].iter().collect()
            } else {
                String::new()
            };
            Ok(Value::String(sliced))
        }
        ("string", "reverse") => {
            if args.is_empty() {
                return Err("string.reverse requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            let rev: String = s.chars().rev().collect();
            Ok(Value::String(rev))
        }
        ("string", "repeat") => {
            if args.len() < 2 {
                return Err("string.repeat requires 2 arguments (s, count)".into());
            }
            let s = value_to_string(&args[0]);
            let count = as_int(&args[1]);
            if count < 0 {
                return Err("string.repeat count cannot be negative".into());
            }
            let repeated = s.repeat(count as usize);
            if repeated.len() > crate::vm::machine::MAX_STRING_SIZE {
                return Err("String size exceeds maximum limit of 64MB".into());
            }
            Ok(Value::String(repeated))
        }
        ("string", "char_at") => {
            if args.len() < 2 {
                return Err("string.char_at requires 2 arguments (s, index)".into());
            }
            let s = value_to_string(&args[0]);
            let idx = as_int(&args[1]);
            let chars: Vec<char> = s.chars().collect();
            if idx < 0 || (idx as usize) >= chars.len() {
                return Err(format!("string.char_at index out of bounds: index {}, length {}", idx, chars.len()));
            }
            Ok(Value::String(chars[idx as usize].to_string()))
        }
        ("string", "count") => {
            if args.len() < 2 {
                return Err("string.count requires 2 arguments (s, sub)".into());
            }
            let s = value_to_string(&args[0]);
            let needle = value_to_string(&args[1]);
            let cnt = if needle.is_empty() {
                s.chars().count() as i64 + 1
            } else {
                s.matches(&needle).count() as i64
            };
            Ok(Value::Int(cnt))
        }
        ("string", "capitalize") => {
            if args.is_empty() {
                return Err("string.capitalize requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut chars = s.chars();
            let cap = match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            };
            Ok(Value::String(cap))
        }
        ("string", "index_of") => {
            if args.len() < 2 {
                return Err("string.index_of requires 2 arguments (s, sub)".into());
            }
            let s = value_to_string(&args[0]);
            let needle = value_to_string(&args[1]);
            let idx = s.find(&needle).map(|byte_idx| {
                s[..byte_idx].chars().count() as i64
            }).unwrap_or(-1);
            Ok(Value::Int(idx))
        }
        ("string", "last_index_of") => {
            if args.len() < 2 {
                return Err("string.last_index_of requires 2 arguments (s, sub)".into());
            }
            let s = value_to_string(&args[0]);
            let needle = value_to_string(&args[1]);
            let idx = s.rfind(&needle).map(|byte_idx| {
                s[..byte_idx].chars().count() as i64
            }).unwrap_or(-1);
            Ok(Value::Int(idx))
        }
        ("string", "pad_start") => {
            if args.len() < 3 {
                return Err("string.pad_start requires 3 arguments (s, target_len, pad)".into());
            }
            let s = value_to_string(&args[0]);
            let target_len = as_int(&args[1]).max(0) as usize;
            let pad = value_to_string(&args[2]);
            let char_count = s.chars().count();
            if pad.is_empty() || char_count >= target_len {
                return Ok(Value::String(s));
            }
            let pad_chars: Vec<char> = pad.chars().collect();
            let mut needed = target_len - char_count;
            let mut prefix = String::new();
            let mut pad_idx = 0;
            while needed > 0 {
                prefix.push(pad_chars[pad_idx % pad_chars.len()]);
                pad_idx += 1;
                needed -= 1;
            }
            let res = format!("{}{}", prefix, s);
            if res.len() > crate::vm::machine::MAX_STRING_SIZE {
                return Err("String size exceeds maximum limit of 64MB".into());
            }
            Ok(Value::String(res))
        }
        ("string", "pad_end") => {
            if args.len() < 3 {
                return Err("string.pad_end requires 3 arguments (s, target_len, pad)".into());
            }
            let s = value_to_string(&args[0]);
            let target_len = as_int(&args[1]).max(0) as usize;
            let pad = value_to_string(&args[2]);
            let char_count = s.chars().count();
            if pad.is_empty() || char_count >= target_len {
                return Ok(Value::String(s));
            }
            let pad_chars: Vec<char> = pad.chars().collect();
            let mut needed = target_len - char_count;
            let mut suffix = String::new();
            let mut pad_idx = 0;
            while needed > 0 {
                suffix.push(pad_chars[pad_idx % pad_chars.len()]);
                pad_idx += 1;
                needed -= 1;
            }
            let res = format!("{}{}", s, suffix);
            if res.len() > crate::vm::machine::MAX_STRING_SIZE {
                return Err("String size exceeds maximum limit of 64MB".into());
            }
            Ok(Value::String(res))
        }
        ("string", "is_numeric") => {
            if args.is_empty() {
                return Err("string.is_numeric requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_ascii_digit())))
        }
        ("string", "is_alpha") => {
            if args.is_empty() {
                return Err("string.is_alpha requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_alphabetic())))
        }
        ("string", "words") => {
            if args.is_empty() {
                return Err("string.words requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            let w = s.split_whitespace().map(|word| Value::String(word.to_string())).collect();
            Ok(Value::Array(w))
        }
        ("string", "trim_start") => {
            if args.is_empty() {
                return Err("string.trim_start requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(s.trim_start().to_string()))
        }
        ("string", "trim_end") => {
            if args.is_empty() {
                return Err("string.trim_end requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(s.trim_end().to_string()))
        }
        ("string", "lines") => {
            if args.is_empty() {
                return Err("string.lines requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            let lines = s.lines().map(|line| Value::String(line.to_string())).collect();
            Ok(Value::Array(lines))
        }
        ("string", "rot13") => {
            if args.is_empty() {
                return Err("string.rot13 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let rotated: String = s.chars().map(|c| match c {
                'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
                'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
                _ => c,
            }).collect();
            Ok(Value::String(rotated))
        }
        ("string", "levenshtein") => {
            if args.len() < 2 {
                return Err("string.levenshtein requires 2 arguments (s1, s2)".into());
            }
            let s1 = value_to_string(&args[0]);
            let s2 = value_to_string(&args[1]);
            let dist = levenshtein_distance(&s1, &s2);
            Ok(Value::Int(dist as i64))
        }
        ("string", "hamming") => {
            if args.len() < 2 {
                return Err("string.hamming requires 2 arguments (s1, s2)".into());
            }
            let s1 = value_to_string(&args[0]);
            let s2 = value_to_string(&args[1]);
            let dist = hamming_distance(&s1, &s2);
            Ok(Value::Int(dist))
        }
        ("string", "slugify") => {
            if args.is_empty() {
                return Err("string.slugify requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(slugify(&s)))
        }
        ("string", "truncate") => {
            if args.len() < 2 {
                return Err("string.truncate requires at least 2 arguments (s, max_len, [ellipsis])".into());
            }
            let s = value_to_string(&args[0]);
            let max_len = as_int(&args[1]).max(0) as usize;
            let ellipsis = if args.len() > 2 { value_to_string(&args[2]) } else { "...".to_string() };
            Ok(Value::String(truncate_string(&s, max_len, &ellipsis)))
        }
        ("string", "from_int") | ("string", "from_float") | ("string", "from_bool") | ("string", "to_string") => {
            if args.is_empty() {
                return Err("string conversion requires 1 argument".into());
            }
            Ok(Value::String(value_to_string(&args[0])))
        }
        ("string", "to_int") | ("string", "parse_int") => {
            if args.is_empty() {
                return Err("string.to_int requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            match s.trim().parse::<i64>() {
                Ok(n) => Ok(Value::Int(n)),
                Err(_) => Err(format!("Cannot parse '{}' to integer", s)),
            }
        }
        ("string", "to_float") | ("string", "parse_float") => {
            if args.is_empty() {
                return Err("string.to_float requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            match s.trim().parse::<f64>() {
                Ok(n) => Ok(Value::Float(n)),
                Err(_) => Err(format!("Cannot parse '{}' to float", s)),
            }
        }
        ("string", "hex_encode") => {
            if args.is_empty() {
                return Err("string.hex_encode requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(hex_encode(&s)))
        }
        ("string", "hex_decode") => {
            if args.is_empty() {
                return Err("string.hex_decode requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(hex_decode(&s)))
        }
        ("string", "fnv1a") => {
            if args.is_empty() {
                return Err("string.fnv1a requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut hash: u64 = 0xcbf29ce484222325;
            for byte in s.as_bytes() {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Ok(Value::Int(hash as i64))
        }
        ("string", "djb2") => {
            if args.is_empty() {
                return Err("string.djb2 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut hash: u64 = 5381;
            for byte in s.as_bytes() {
                hash = hash.wrapping_mul(33).wrapping_add(*byte as u64);
            }
            Ok(Value::Int(hash as i64))
        }
        ("string", "adler32") => {
            if args.is_empty() {
                return Err("string.adler32 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut a: u32 = 1;
            let mut b: u32 = 0;
            for byte in s.as_bytes() {
                a = (a + *byte as u32) % 65521;
                b = (b + a) % 65521;
            }
            let adler = ((b << 16) | a) as i64;
            Ok(Value::Int(adler))
        }
        ("string", "crc32") => {
            if args.is_empty() {
                return Err("string.crc32 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut crc: u32 = 0xFFFFFFFF;
            for byte in s.as_bytes() {
                crc ^= *byte as u32;
                for _ in 0..8 {
                    crc = if (crc & 1) != 0 {
                        (crc >> 1) ^ 0xEDB88320
                    } else {
                        crc >> 1
                    };
                }
            }
            let res = (!crc) as i64;
            Ok(Value::Int(res))
        }
        ("string", "is_ascii") => {
            if args.is_empty() {
                return Err("string.is_ascii requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::Bool(is_string_ascii(&s)))
        }
        ("string", "is_alphanumeric") => {
            if args.is_empty() {
                return Err("string.is_alphanumeric requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::Bool(is_string_alphanumeric(&s)))
        }
        ("string", "count_occurrences") => {
            if args.len() < 2 {
                return Err("string.count_occurrences requires 2 arguments (s, sub)".into());
            }
            let s = value_to_string(&args[0]);
            let sub = value_to_string(&args[1]);
            Ok(Value::Int(count_string_occurrences(&s, &sub)))
        }
        ("string", "pad_zero") => {
            if args.len() < 2 {
                return Err("string.pad_zero requires 2 arguments (s, width)".into());
            }
            let s = value_to_string(&args[0]);
            let width = as_int(&args[1]).max(0) as usize;
            Ok(Value::String(pad_zero_string(&s, width)))
        }
        ("string", "lines_count") => {
            if args.is_empty() {
                return Err("string.lines_count requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::Int(string_lines_count(&s)))
        }
        ("string", "words_count") => {
            if args.is_empty() {
                return Err("string.words_count requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::Int(string_words_count(&s)))
        }

        // --- ARRAY ---
        ("array", "push") | ("array", "append") => {
            if args.len() < 2 {
                return Err("array.push requires 2 arguments (arr, element)".into());
            }
            if let Value::Array(arr) = &args[0] {
                if arr.len() >= crate::vm::machine::MAX_ARRAY_SIZE {
                    return Err("Array size exceeds maximum limit of 10,000,000".into());
                }
                let mut res = arr.clone();
                res.push(args[1].clone());
                Ok(Value::Array(res))
            } else {
                Err("array.push requires an array".into())
            }
        }
        ("array", "pop") => {
            if args.is_empty() {
                return Err("array.pop requires 1 argument (arr)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let mut res = arr.clone();
                res.pop();
                Ok(Value::Array(res))
            } else {
                Err("array.pop requires an array".into())
            }
        }
        ("array", "len") => {
            if args.is_empty() {
                return Err("array.len requires 1 argument".into());
            }
            if let Value::Array(arr) = &args[0] {
                Ok(Value::Int(arr.len() as i64))
            } else {
                Ok(Value::Int(0))
            }
        }
        ("array", "is_empty") => {
            if args.is_empty() {
                return Err("array.is_empty requires 1 argument".into());
            }
            if let Value::Array(arr) = &args[0] {
                Ok(Value::Bool(arr.is_empty()))
            } else {
                Ok(Value::Bool(true))
            }
        }
        ("array", "contains") => {
            if args.len() < 2 {
                return Err("array.contains requires 2 arguments".into());
            }
            if let Value::Array(arr) = &args[0] {
                Ok(Value::Bool(arr.contains(&args[1])))
            } else {
                Ok(Value::Bool(false))
            }
        }
        ("array", "reverse") => {
            if args.is_empty() {
                return Err("array.reverse requires 1 argument".into());
            }
            if let Value::Array(arr) = &args[0] {
                let mut rev = arr.clone();
                rev.reverse();
                Ok(Value::Array(rev))
            } else {
                Ok(Value::Array(Vec::new()))
            }
        }
        ("array", "sort") => {
            if args.is_empty() {
                return Err("array.sort requires 1 argument".into());
            }
            if let Value::Array(arr) = &args[0] {
                let mut sorted = arr.clone();
                sorted.sort_by(|a, b| a.compare_order(b));
                Ok(Value::Array(sorted))
            } else {
                Err("array.sort requires an array".into())
            }
        }
        ("array", "slice") => {
            if args.len() < 3 {
                return Err("array.slice requires 3 arguments (arr, start, end)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let start = as_int(&args[1]);
                let end = as_int(&args[2]);
                let len = arr.len() as i64;
                let s = start.max(0).min(len) as usize;
                let e = end.max(0).min(len) as usize;
                let sliced = if s <= e { arr[s..e].to_vec() } else { Vec::new() };
                Ok(Value::Array(sliced))
            } else {
                Err("array.slice requires an array".into())
            }
        }
        ("array", "index_of") => {
            if args.len() < 2 {
                return Err("array.index_of requires 2 arguments (arr, value)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let target = &args[1];
                let idx = arr.iter().position(|v| v == target).map(|i| i as i64).unwrap_or(-1);
                Ok(Value::Int(idx))
            } else {
                Err("array.index_of requires an array".into())
            }
        }
        ("array", "concat") => {
            if args.len() < 2 {
                return Err("array.concat requires 2 arguments (arr1, arr2)".into());
            }
            match (&args[0], &args[1]) {
                (Value::Array(a), Value::Array(b)) => {
                    let mut res = a.clone();
                    if res.len().saturating_add(b.len()) > crate::vm::machine::MAX_ARRAY_SIZE {
                        return Err("Array size exceeds maximum limit of 10,000,000".into());
                    }
                    res.extend(b.clone());
                    Ok(Value::Array(res))
                }
                _ => Err("array.concat requires two arrays".into()),
            }
        }
        ("array", "join") => {
            if args.len() < 2 {
                return Err("array.join requires 2 arguments".into());
            }
            let delim = value_to_string(&args[1]);
            if let Value::Array(arr) = &args[0] {
                let parts: Vec<String> = arr.iter().map(value_to_string).collect();
                Ok(Value::String(parts.join(&delim)))
            } else {
                Ok(Value::String("".to_string()))
            }
        }
        ("array", "first") => {
            if args.is_empty() {
                return Err("array.first requires 1 argument".into());
            }
            if let Value::Array(arr) = &args[0] {
                if let Some(val) = arr.first() {
                    Ok(val.clone())
                } else {
                    Err("array.first called on empty array".into())
                }
            } else {
                Err("array.first requires an array".into())
            }
        }
        ("array", "last") => {
            if args.is_empty() {
                return Err("array.last requires 1 argument".into());
            }
            if let Value::Array(arr) = &args[0] {
                if let Some(val) = arr.last() {
                    Ok(val.clone())
                } else {
                    Err("array.last called on empty array".into())
                }
            } else {
                Err("array.last requires an array".into())
            }
        }
        ("array", "swap") => {
            if args.len() < 3 {
                return Err("array.swap requires 3 arguments (arr, i, j)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let i = as_int(&args[1]);
                let j = as_int(&args[2]);
                let len = arr.len() as i64;
                if i < 0 || i >= len || j < 0 || j >= len {
                    return Err(format!("array.swap index out of bounds: ({}, {}), length {}", i, j, len));
                }
                let mut swapped = arr.clone();
                swapped.swap(i as usize, j as usize);
                Ok(Value::Array(swapped))
            } else {
                Err("array.swap requires an array".into())
            }
        }
        ("array", "count") => {
            if args.len() < 2 {
                return Err("array.count requires 2 arguments (arr, val)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let target = &args[1];
                let cnt = arr.iter().filter(|v| *v == target).count() as i64;
                Ok(Value::Int(cnt))
            } else {
                Err("array.count requires an array".into())
            }
        }
        ("array", "take") => {
            if args.len() < 2 {
                return Err("array.take requires 2 arguments (arr, n)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let n = as_int(&args[1]).max(0) as usize;
                Ok(Value::Array(arr.iter().take(n).cloned().collect()))
            } else {
                Err("array.take requires an array".into())
            }
        }
        ("array", "drop") => {
            if args.len() < 2 {
                return Err("array.drop requires 2 arguments (arr, n)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let n = as_int(&args[1]).max(0) as usize;
                Ok(Value::Array(arr.iter().skip(n).cloned().collect()))
            } else {
                Err("array.drop requires an array".into())
            }
        }
        ("array", "remove_at") => {
            if args.len() < 2 {
                return Err("array.remove_at requires 2 arguments (arr, index)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let idx = as_int(&args[1]);
                if idx < 0 || (idx as usize) >= arr.len() {
                    return Err(format!("array.remove_at index out of bounds: index {}, length {}", idx, arr.len()));
                }
                let mut res = arr.clone();
                let removed = res.remove(idx as usize);
                Ok(removed)
            } else {
                Err("array.remove_at requires an array".into())
            }
        }
        ("array", "insert") => {
            if args.len() < 3 {
                return Err("array.insert requires 3 arguments (arr, index, element)".into());
            }
            if let Value::Array(arr) = &args[0] {
                let idx = as_int(&args[1]);
                if idx < 0 || (idx as usize) > arr.len() {
                    return Err(format!("array.insert index out of bounds: index {}, length {}", idx, arr.len()));
                }
                if arr.len() >= crate::vm::machine::MAX_ARRAY_SIZE {
                    return Err("Array size exceeds maximum limit of 10,000,000".into());
                }
                let mut res = arr.clone();
                res.insert(idx as usize, args[2].clone());
                Ok(Value::Array(res))
            } else {
                Err("array.insert requires an array".into())
            }
        }
        ("array", "fill") => {
            if args.len() < 2 {
                return Err("array.fill requires 2 arguments (count, value)".into());
            }
            let (count, value) = match (&args[0], &args[1]) {
                (Value::Int(c), val) => (*c, val.clone()),
                (val, Value::Int(c)) => (*c, val.clone()),
                _ => return Err("array.fill requires an integer count".into()),
            };
            if count < 0 {
                return Err("array.fill count cannot be negative".into());
            }
            let cnt = count as usize;
            if cnt > crate::vm::machine::MAX_ARRAY_SIZE {
                return Err("Array size exceeds maximum limit of 10,000,000".into());
            }
            Ok(Value::Array(vec![value; cnt]))
        }
        ("array", "unique") => {
            if args.is_empty() {
                return Err("array.unique requires 1 argument".into());
            }
            if let Value::Array(arr) = &args[0] {
                let mut unique = Vec::new();
                for item in arr {
                    if !unique.contains(item) {
                        unique.push(item.clone());
                    }
                }
                Ok(Value::Array(unique))
            } else {
                Err("array.unique requires an array".into())
            }
        }
        ("array", "chunk") => {
            if args.len() < 2 {
                return Err("array.chunk requires 2 arguments (arr, size)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.chunk requires an array as first argument".into());
            };
            let size = as_int(&args[1]);
            if size <= 0 {
                return Err("array.chunk size must be greater than 0".into());
            }
            let chunks = arr.chunks(size as usize).map(|c| Value::Array(c.to_vec())).collect();
            Ok(Value::Array(chunks))
        }
        ("array", "flatten") => {
            if args.is_empty() {
                return Err("array.flatten requires 1 argument".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.flatten requires an array".into());
            };
            let mut flat = Vec::new();
            for item in arr {
                match item {
                    Value::Array(inner) => {
                        if flat.len().saturating_add(inner.len()) > crate::vm::machine::MAX_ARRAY_SIZE {
                            return Err("Array size exceeds maximum limit of 10,000,000".into());
                        }
                        flat.extend(inner.clone());
                    }
                    other => {
                        if flat.len() >= crate::vm::machine::MAX_ARRAY_SIZE {
                            return Err("Array size exceeds maximum limit of 10,000,000".into());
                        }
                        flat.push(other.clone());
                    }
                }
            }
            Ok(Value::Array(flat))
        }
        ("array", "sum") => {
            if args.is_empty() {
                return Err("array.sum requires 1 argument".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.sum requires an array".into());
            };
            let mut is_float = false;
            let mut total_int: i64 = 0;
            let mut total_float: f64 = 0.0;
            for item in arr {
                match item {
                    Value::Int(i) => {
                        total_int = total_int.wrapping_add(*i);
                        total_float += *i as f64;
                    }
                    Value::Float(f) => {
                        is_float = true;
                        total_float += *f;
                    }
                    _ => return Err("array.sum requires an array of numbers".into()),
                }
            }
            if is_float {
                Ok(Value::Float(total_float))
            } else {
                Ok(Value::Int(total_int))
            }
        }
        ("array", "average") => {
            if args.is_empty() {
                return Err("array.average requires 1 argument (arr)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.average requires an array".into());
            };
            if arr.is_empty() {
                return Err("array.average cannot be called on an empty array".into());
            }
            let mut total_float: f64 = 0.0;
            for item in arr {
                match item {
                    Value::Int(i) => total_float += *i as f64,
                    Value::Float(f) => total_float += *f,
                    _ => return Err("array.average requires an array of numbers".into()),
                }
            }
            Ok(Value::Float(total_float / arr.len() as f64))
        }
        ("array", "product") => {
            if args.is_empty() {
                return Err("array.product requires 1 argument (arr)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.product requires an array".into());
            };
            let mut total_int: i64 = 1;
            let mut total_float: f64 = 1.0;
            let mut is_float = false;
            for item in arr {
                match item {
                    Value::Int(i) => {
                        total_int = total_int.wrapping_mul(*i);
                        total_float *= *i as f64;
                    }
                    Value::Float(f) => {
                        is_float = true;
                        total_float *= *f;
                    }
                    _ => return Err("array.product requires an array of numbers".into()),
                }
            }
            if is_float {
                Ok(Value::Float(total_float))
            } else {
                Ok(Value::Int(total_int))
            }
        }
        ("array", "dedup") => {
            if args.is_empty() {
                return Err("array.dedup requires 1 argument (arr)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.dedup requires an array".into());
            };
            let mut deduped: Vec<Value> = Vec::new();
            for v in arr {
                if deduped.last() != Some(v) {
                    deduped.push(v.clone());
                }
            }
            Ok(Value::Array(deduped))
        }
        ("array", "min") => {
            if args.is_empty() {
                return Err("array.min requires 1 argument".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.min requires an array".into());
            };
            if arr.is_empty() {
                return Err("array.min cannot be called on an empty array".into());
            }
            let mut m = arr[0].clone();
            for item in &arr[1..] {
                if item.compare_order(&m) == std::cmp::Ordering::Less {
                    m = item.clone();
                }
            }
            Ok(m)
        }
        ("array", "max") => {
            if args.is_empty() {
                return Err("array.max requires 1 argument".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.max requires an array".into());
            };
            if arr.is_empty() {
                return Err("array.max cannot be called on an empty array".into());
            }
            let mut m = arr[0].clone();
            for item in &arr[1..] {
                if item.compare_order(&m) == std::cmp::Ordering::Greater {
                    m = item.clone();
                }
            }
            Ok(m)
        }
        ("array", "zip") => {
            if args.len() < 2 {
                return Err("array.zip requires 2 arguments (a, b)".into());
            }
            let (Value::Array(a), Value::Array(b)) = (&args[0], &args[1]) else {
                return Err("array.zip requires two arrays".into());
            };
            let zipped = a.iter().zip(b.iter()).map(|(x, y)| Value::Array(vec![x.clone(), y.clone()])).collect();
            Ok(Value::Array(zipped))
        }
        ("array", "binary_search") => {
            if args.len() < 2 {
                return Err("array.binary_search requires 2 arguments (arr, target)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.binary_search requires an array".into());
            };
            let target = &args[1];
            let mut low: isize = 0;
            let mut high: isize = arr.len() as isize - 1;
            let mut found_idx: i64 = -1;
            while low <= high {
                let mid = (low + high) / 2;
                match arr[mid as usize].compare_order(target) {
                    std::cmp::Ordering::Equal => {
                        found_idx = mid as i64;
                        break;
                    }
                    std::cmp::Ordering::Less => {
                        low = mid + 1;
                    }
                    std::cmp::Ordering::Greater => {
                        high = mid - 1;
                    }
                }
            }
            Ok(Value::Int(found_idx))
        }
        ("array", "rotate") => {
            if args.len() < 2 {
                return Err("array.rotate requires 2 arguments (arr, count)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.rotate requires an array".into());
            };
            if arr.is_empty() {
                return Ok(Value::Array(Vec::new()));
            }
            let count = as_int(&args[1]);
            let len = arr.len() as i64;
            let shift = ((count % len) + len) % len;
            let mut rotated = arr.clone();
            rotated.rotate_right(shift as usize);
            Ok(Value::Array(rotated))
        }
        ("array", "repeat") => {
            if args.len() < 2 {
                return Err("array.repeat requires 2 arguments (val, count)".into());
            }
            let count = as_int(&args[1]);
            if count < 0 {
                return Err("array.repeat count cannot be negative".into());
            }
            if (count as usize) > crate::vm::machine::MAX_ARRAY_SIZE {
                return Err(format!("array.repeat size {} exceeds maximum array limit", count));
            }
            let repeated = vec![args[0].clone(); count as usize];
            Ok(Value::Array(repeated))
        }
        ("array", "windows") => {
            if args.len() < 2 {
                return Err("array.windows requires 2 arguments (arr, size)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.windows requires an array".into());
            };
            let size = as_int(&args[1]);
            if size <= 0 || (size as usize) > arr.len() {
                return Ok(Value::Array(Vec::new()));
            }
            let sz = size as usize;
            let mut res = Vec::new();
            for window in arr.windows(sz) {
                res.push(Value::Array(window.to_vec()));
            }
            Ok(Value::Array(res))
        }
        ("array", "frequencies") => {
            if args.is_empty() {
                return Err("array.frequencies requires 1 argument (arr)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.frequencies requires an array".into());
            };
            let mut map = std::collections::HashMap::new();
            for item in arr {
                let key = match item {
                    Value::String(s) => s.clone(),
                    Value::Int(i) => i.to_string(),
                    Value::Bool(b) => b.to_string(),
                    other => value_to_string(other),
                };
                let count = map.entry(key).or_insert(0i64);
                *count += 1;
            }
            let out_map = map.into_iter().map(|(k, v)| (crate::value::MapKey::String(k), Value::Int(v))).collect();
            Ok(Value::Map(out_map))
        }
        ("array", "is_sorted") => {
            if args.is_empty() {
                return Err("array.is_sorted requires 1 argument (arr)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.is_sorted requires an array".into());
            };
            Ok(Value::Bool(is_array_sorted(arr)))
        }
        ("array", "all_unique") => {
            if args.is_empty() {
                return Err("array.all_unique requires 1 argument (arr)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.all_unique requires an array".into());
            };
            Ok(Value::Bool(are_array_elements_unique(arr)))
        }
        ("array", "find_index") => {
            if args.len() < 2 {
                return Err("array.find_index requires 2 arguments (arr, target)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.find_index requires an array".into());
            };
            let target = &args[1];
            let idx = arr.iter().position(|v| v == target).map(|i| i as i64).unwrap_or(-1);
            Ok(Value::Int(idx))
        }
        ("array", "clamp_elements") => {
            if args.len() < 3 {
                return Err("array.clamp_elements requires 3 arguments (arr, min, max)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.clamp_elements requires an array".into());
            };
            let min_f = as_float(&args[1]);
            let max_f = as_float(&args[2]);
            let min_i = as_int(&args[1]);
            let max_i = as_int(&args[2]);
            let clamped = arr.iter().map(|v| match v {
                Value::Int(i) => Value::Int(i.clamp(&min_i, &max_i).clone()),
                Value::Float(f) => Value::Float(f.clamp(min_f, max_f)),
                other => other.clone(),
            }).collect();
            Ok(Value::Array(clamped))
        }
        ("array", "compact") => {
            if args.is_empty() {
                return Err("array.compact requires 1 argument (arr)".into());
            }
            let Value::Array(arr) = &args[0] else {
                return Err("array.compact requires an array".into());
            };
            let compacted: Vec<Value> = arr.iter().filter(|v| !matches!(v, Value::Unit)).cloned().collect();
            Ok(Value::Array(compacted))
        }
        ("array", "diff") => {
            if args.len() < 2 {
                return Err("array.diff requires 2 arguments (a, b)".into());
            }
            let (Value::Array(a), Value::Array(b)) = (&args[0], &args[1]) else {
                return Err("array.diff requires two arrays".into());
            };
            let diff: Vec<Value> = a.iter().filter(|v| !b.contains(v)).cloned().collect();
            Ok(Value::Array(diff))
        }
        ("array", "intersect") => {
            if args.len() < 2 {
                return Err("array.intersect requires 2 arguments (a, b)".into());
            }
            let (Value::Array(a), Value::Array(b)) = (&args[0], &args[1]) else {
                return Err("array.intersect requires two arrays".into());
            };
            let mut res: Vec<Value> = Vec::new();
            for v in a {
                if b.contains(v) && !res.contains(v) {
                    res.push(v.clone());
                }
            }
            Ok(Value::Array(res))
        }

        // --- PATH ---
        ("path", "join") => {
            if args.len() < 2 {
                return Err("path.join requires 2 arguments".into());
            }
            let a = value_to_string(&args[0]);
            let b = value_to_string(&args[1]);
            let path = std::path::Path::new(&a).join(b);
            Ok(Value::String(path.to_string_lossy().to_string()))
        }
        ("path", "dirname") => {
            if args.is_empty() {
                return Err("path.dirname requires 1 argument".into());
            }
            let p = value_to_string(&args[0]);
            let path = std::path::Path::new(&p);
            let dir = path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            Ok(Value::String(dir))
        }
        ("path", "basename") => {
            if args.is_empty() {
                return Err("path.basename requires 1 argument".into());
            }
            let p = value_to_string(&args[0]);
            let path = std::path::Path::new(&p);
            let base = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            Ok(Value::String(base))
        }
        ("path", "extension") => {
            if args.is_empty() {
                return Err("path.extension requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            Ok(Value::String(
                std::path::Path::new(&path)
                    .extension()
                    .map(|extension| extension.to_string_lossy().to_string())
                    .unwrap_or_default(),
            ))
        }
        ("path", "exists") => {
            if args.is_empty() {
                return Err("path.exists requires 1 argument".into());
            }
            let p = value_to_string(&args[0]);
            Ok(Value::Bool(std::path::Path::new(&p).exists()))
        }
        ("path", "is_file") => {
            if args.is_empty() {
                return Err("path.is_file requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            Ok(Value::Bool(std::path::Path::new(&path).is_file()))
        }
        ("path", "is_dir") => {
            if args.is_empty() {
                return Err("path.is_dir requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            Ok(Value::Bool(std::path::Path::new(&path).is_dir()))
        }

        // --- ENV ---
        ("env", "get") => {
            if args.is_empty() {
                return Err("env.get requires 1 argument".into());
            }
            let key = value_to_string(&args[0]);
            Ok(match std::env::var(key) {
                Ok(value) => option_some(Value::String(value)),
                Err(_) => option_none(),
            })
        }
        ("env", "has") => {
            if args.is_empty() {
                return Err("env.has requires 1 argument".into());
            }
            let key = value_to_string(&args[0]);
            Ok(Value::Bool(std::env::var(key).is_ok()))
        }
        ("env", "set") => {
            if args.len() < 2 {
                return Err("env.set requires 2 arguments".into());
            }
            if sandbox_root().is_some() {
                return Err("Sandbox denied env.set; environment mutation is privileged".into());
            }
            let key = value_to_string(&args[0]);
            let val = value_to_string(&args[1]);
            std::env::set_var(key, val);
            Ok(Value::Bool(true))
        }

        // --- PROCESS ---
        ("process", "args") => Ok(Value::Array(PROCESS_ARGS.with(|current| {
            current
                .borrow()
                .iter()
                .cloned()
                .map(Value::String)
                .collect()
        }))),
        ("process", "cwd") => std::env::current_dir()
            .map(|path| Value::String(path.to_string_lossy().to_string()))
            .map_err(|error| format!("cannot read current directory: {}", error)),

        // --- JSON ---
        ("json", "parse") => {
            if args.is_empty() {
                return Err("json.parse requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            parse_json_limited(&s, "json.parse input")
        }
        ("json", "try_parse") => {
            if args.is_empty() {
                return Err("json.try_parse requires 1 argument".into());
            }
            let s = value_to_string(&args[0]);
            Ok(match parse_json_limited(&s, "json.try_parse input") {
                Ok(value) => result_ok(value),
                Err(error) => result_err(error, 2),
            })
        }
        ("json", "stringify") => {
            if args.is_empty() {
                return Err("json.stringify requires 1 argument".into());
            }
            Ok(Value::String(stringify_json_limited(
                &args[0],
                "json.stringify output",
                false,
            )?))
        }

        // --- RESULT / OPTION ---
        ("result", "ok") => {
            if args.len() != 1 {
                return Err("result.ok requires 1 argument".into());
            }
            Ok(result_ok(args.remove(0)))
        }
        ("result", "err") => {
            if args.len() < 2 {
                return Err("result.err requires 2 arguments".into());
            }
            Ok(result_err(value_to_string(&args[0]), as_int(&args[1])))
        }
        ("option", "some") => {
            if args.len() != 1 {
                return Err("option.some requires 1 argument".into());
            }
            Ok(option_some(args.remove(0)))
        }
        ("option", "none") => {
            if !args.is_empty() {
                return Err("option.none requires no arguments".into());
            }
            Ok(option_none())
        }

        // --- CLI ---
        ("cli", "flag") => {
            if args.len() < 2 {
                return Err("cli.flag requires 2 arguments".into());
            }
            let name = value_to_string(&args[1]);
            let found = match &args[0] {
                Value::Array(items) => items.iter().any(|item| value_to_string(item) == name),
                _ => false,
            };
            Ok(Value::Bool(found))
        }
        ("cli", "option") => {
            if args.len() < 2 {
                return Err("cli.option requires 2 arguments".into());
            }
            let name = value_to_string(&args[1]);
            let value = match &args[0] {
                Value::Array(items) => items.windows(2).find_map(|window| {
                    (value_to_string(&window[0]) == name).then(|| value_to_string(&window[1]))
                }),
                _ => None,
            };
            Ok(value
                .map(|value| option_some(Value::String(value)))
                .unwrap_or_else(option_none))
        }

        // --- UI v0.3 ---
        ("ui", "create") => Err("ui.create was removed in UI v0.3; use ui.window(...)".into()),
        ("ui", "open_window") => {
            Err("ui.open_window was removed in UI v0.3; use ui.window(...)".into())
        }
        ("ui", "begin_frame") => {
            Err("ui.begin_frame was removed in UI v0.3; use ui.frame(window)".into())
        }
        ("ui", "rect") => Err("ui.rect was removed in UI v0.3; use ui.draw_rect(...)".into()),
        ("ui", "label") => Err("ui.label was removed in UI v0.3; use ui.text(...)".into()),
        ("ui", "image") => Err("ui.image was removed in UI v0.3; use ui.draw_image(...)".into()),
        ("ui", "sprite") => Err("ui.sprite was removed in UI v0.3; use ui.draw_sprite(...)".into()),
        ("ui", "clear") => {
            Err("ui.clear was removed in UI v0.3; use ui.panel(...) or ui.draw_rect(...)".into())
        }
        ("ui", "window") => {
            let title = arg_string(&args, 0, "Vice App");
            let size = positive_size(&args, 1, [800, 600])?;
            let third = arg_string(&args, 2, "vice");
            let theme = if third.starts_with('#') {
                "vice".to_string()
            } else {
                third
            };
            let color = if args
                .get(2)
                .map(value_to_string)
                .is_some_and(|value| value.starts_with('#'))
            {
                arg_color(&args, 2, [8, 12, 14, 255])?
            } else if args.len() > 3 {
                arg_color(&args, 3, [8, 12, 14, 255])?
            } else if theme == "vice" {
                [8, 12, 14, 255]
            } else {
                [20, 20, 20, 255]
            };
            crate::ui::GLOBAL_UI.open_window(UiWindowConfig {
                title,
                width: size[0],
                height: size[1],
                color,
            });
            Ok(window_value(size[0], size[1], true))
        }
        ("ui", "frame") => {
            if args.len() != 1 {
                return Err("ui.frame requires a Window".into());
            }
            Ok(Value::Bool(crate::ui::GLOBAL_UI.frame()))
        }
        ("ui", "set_theme") => {
            let name = arg_string(&args, 0, "vice");
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Theme { name });
            Ok(Value::Bool(true))
        }
        ("ui", "push_style") => {
            crate::ui::GLOBAL_UI.record(UiRenderCommand::StylePush);
            Ok(Value::Bool(true))
        }
        ("ui", "pop_style") => {
            crate::ui::GLOBAL_UI.record(UiRenderCommand::StylePop);
            Ok(Value::Bool(true))
        }
        ("ui", "panel") => {
            let id = require_id(&args, 0, "panel")?;
            let title = arg_string(&args, 1, "");
            let pos = arg_pair(&args, 2, [0, 0])?;
            let size = positive_size(&args, 3, [320, 240])?;
            crate::ui::GLOBAL_UI.push_panel();
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Panel {
                id,
                title,
                x: pos[0] as i32,
                y: pos[1] as i32,
                width: size[0],
                height: size[1],
            });
            Ok(Value::Bool(true))
        }
        ("ui", "end_panel") => {
            if !crate::ui::GLOBAL_UI.pop_panel() {
                return Err("ui.end_panel called without ui.panel".into());
            }
            crate::ui::GLOBAL_UI.record(UiRenderCommand::EndPanel);
            Ok(Value::Bool(true))
        }
        ("ui", "row" | "column") => {
            let id = require_id(&args, 0, method)?;
            crate::ui::GLOBAL_UI.push_layout();
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Layout {
                kind: method.to_string(),
                id,
            });
            Ok(Value::Bool(true))
        }
        ("ui", "end") => {
            if !crate::ui::GLOBAL_UI.pop_layout() {
                return Err("ui.end called without ui.row or ui.column".into());
            }
            crate::ui::GLOBAL_UI.record(UiRenderCommand::End);
            Ok(Value::Bool(true))
        }
        ("ui", "spacer") => {
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Widget {
                kind: "spacer".to_string(),
                id: arg_string(&args, 0, ""),
            });
            Ok(Value::Bool(true))
        }
        ("ui", "separator") => {
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Widget {
                kind: "separator".to_string(),
                id: arg_string(&args, 0, ""),
            });
            Ok(Value::Bool(true))
        }
        ("ui", "scroll_area") => {
            let id = require_id(&args, 0, "scroll_area")?;
            let pos = arg_pair(&args, 1, [0, 0])?;
            let size = positive_size(&args, 2, [320, 240])?;
            crate::ui::GLOBAL_UI.push_scroll();
            crate::ui::GLOBAL_UI.record(UiRenderCommand::ScrollArea {
                id,
                x: pos[0] as i32,
                y: pos[1] as i32,
                width: size[0],
                height: size[1],
            });
            Ok(Value::Bool(true))
        }
        ("ui", "end_scroll") => {
            if !crate::ui::GLOBAL_UI.pop_scroll() {
                return Err("ui.end_scroll called without ui.scroll_area".into());
            }
            crate::ui::GLOBAL_UI.record(UiRenderCommand::EndScroll);
            Ok(Value::Bool(true))
        }
        ("ui", "text" | "heading" | "mono" | "badge") => {
            let (text, pos, color) = if matches!(args.first(), Some(Value::Array(_))) {
                (
                    arg_string(&args, 1, ""),
                    arg_pair(&args, 0, [0, 0])?,
                    arg_color(&args, 2, [230, 246, 237, 255])?,
                )
            } else {
                (
                    arg_string(&args, 0, ""),
                    arg_pair(&args, 1, [0, 0])?,
                    arg_color(&args, 2, [230, 246, 237, 255])?,
                )
            };
            crate::ui::GLOBAL_UI.add_label(UiLabel {
                text,
                x: pos[0] as i32,
                y: pos[1] as i32,
                color,
            });
            Ok(Value::Bool(true))
        }
        ("ui", "button") => {
            let id = require_id(&args, 0, "button")?;
            let (text, pos, size, color) = if matches!(args.get(1), Some(Value::Array(_))) {
                (
                    arg_string(&args, 3, ""),
                    arg_pair(&args, 1, [0, 0])?,
                    positive_size(&args, 2, [120, 36])?,
                    arg_color(&args, 4, [183, 243, 74, 255])?,
                )
            } else {
                (
                    arg_string(&args, 1, ""),
                    arg_pair(&args, 2, [0, 0])?,
                    positive_size(&args, 3, [120, 36])?,
                    arg_color(&args, 4, [183, 243, 74, 255])?,
                )
            };
            let (mouse_pos, mouse_down, mouse_clicked) = crate::ui::GLOBAL_UI.get_mouse_state();
            let hovered = mouse_pos.0 >= pos[0] as f64
                && mouse_pos.0 <= (pos[0] + size[0] as i64) as f64
                && mouse_pos.1 >= pos[1] as f64
                && mouse_pos.1 <= (pos[1] + size[1] as i64) as f64;
            crate::ui::GLOBAL_UI.add_button(UiButton {
                text,
                x: pos[0] as i32,
                y: pos[1] as i32,
                width: size[0],
                height: size[1],
                color,
            });
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Widget {
                kind: "button".to_string(),
                id,
            });
            Ok(ui_response(
                hovered && mouse_clicked,
                hovered,
                hovered && mouse_down,
                false,
                false,
            ))
        }
        ("ui", "checkbox" | "slider" | "select" | "menu_item") => {
            let id = require_id(&args, 0, method)?;
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Widget {
                kind: method.to_string(),
                id,
            });
            Ok(ui_response(false, false, false, false, false))
        }
        ("ui", "input_text" | "text_area") => {
            let id = require_id(&args, 0, method)?;
            let value = arg_string(&args, 1, "");
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Widget {
                kind: method.to_string(),
                id,
            });
            Ok(text_input_response(value, false, false, false))
        }
        (
            "ui",
            "progress" | "list_item" | "table_row" | "table_cell" | "menu_bar" | "menu"
            | "status_bar" | "tooltip",
        ) => {
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Widget {
                kind: method.to_string(),
                id: arg_string(&args, 0, ""),
            });
            Ok(Value::Bool(true))
        }
        ("ui", "begin_table") => {
            let id = require_id(&args, 0, "begin_table")?;
            let columns = args.get(1).map(as_int).unwrap_or(1).max(1) as u32;
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Table { id, columns });
            Ok(Value::Bool(true))
        }
        ("ui", "end_table") => {
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Widget {
                kind: "end_table".to_string(),
                id: String::new(),
            });
            Ok(Value::Bool(true))
        }
        ("ui", "canvas") => {
            let id = require_id(&args, 0, "canvas")?;
            let size = positive_size(&args, 1, [640, 360])?;
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Widget {
                kind: "canvas".to_string(),
                id: id.clone(),
            });
            Ok(canvas_value(id, size[0], size[1]))
        }
        ("game2d", "begin")
        | ("game2d", "end")
        | ("game2d", "set_layer")
        | ("game2d", "set_camera") => Ok(Value::Bool(true)),
        ("game2d", "tilemap") => Ok(Value::Bool(true)),
        ("ui", "draw_rect") | ("game2d", "rect") => {
            let offset = canvas_arg_offset(&args);
            let pos = arg_pair(&args, offset, [0, 0])?;
            let size = positive_size(&args, offset + 1, [100, 100])?;
            let color = arg_color(&args, offset + 2, [255, 255, 255, 255])?;
            crate::ui::GLOBAL_UI.add_rect(UiRect {
                x: pos[0] as i32,
                y: pos[1] as i32,
                width: size[0],
                height: size[1],
                color,
            });
            Ok(Value::Bool(true))
        }
        ("ui", "draw_line") => {
            let offset = canvas_arg_offset(&args);
            let start = arg_pair(&args, offset, [0, 0])?;
            let end = arg_pair(&args, offset + 1, [0, 0])?;
            let color = arg_color(&args, offset + 2, [255, 255, 255, 255])?;
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Line {
                x1: start[0] as i32,
                y1: start[1] as i32,
                x2: end[0] as i32,
                y2: end[1] as i32,
                color,
            });
            Ok(Value::Bool(true))
        }
        ("ui", "draw_circle") => {
            let offset = canvas_arg_offset(&args);
            let pos = arg_pair(&args, offset, [0, 0])?;
            let radius = args.get(offset + 1).map(as_int).unwrap_or(10);
            if radius <= 0 {
                return Err("ui.draw_circle expects a positive radius".into());
            }
            let color = arg_color(&args, offset + 2, [255, 255, 255, 255])?;
            crate::ui::GLOBAL_UI.record(UiRenderCommand::Circle {
                x: pos[0] as i32,
                y: pos[1] as i32,
                radius: radius as u32,
                color,
            });
            Ok(Value::Bool(true))
        }
        ("game2d", "text_cached") => {
            let text = arg_string(&args, 1, "");
            let pos = arg_pair(&args, 2, [0, 0])?;
            let color = arg_color(&args, 3, [230, 246, 237, 255])?;
            crate::ui::GLOBAL_UI.add_label(UiLabel {
                text,
                x: pos[0] as i32,
                y: pos[1] as i32,
                color,
            });
            Ok(Value::Bool(true))
        }
        ("ui", "draw_text") | ("game2d", "text") => {
            let offset = canvas_arg_offset(&args);
            let (text, pos, color) = if matches!(args.get(offset), Some(Value::Array(_))) {
                (
                    arg_string(&args, offset + 1, ""),
                    arg_pair(&args, offset, [0, 0])?,
                    arg_color(&args, offset + 2, [230, 246, 237, 255])?,
                )
            } else {
                (
                    arg_string(&args, offset, ""),
                    arg_pair(&args, offset + 1, [0, 0])?,
                    arg_color(&args, offset + 2, [230, 246, 237, 255])?,
                )
            };
            crate::ui::GLOBAL_UI.add_label(UiLabel {
                text,
                x: pos[0] as i32,
                y: pos[1] as i32,
                color,
            });
            Ok(Value::Bool(true))
        }
        ("ui", "draw_image") | ("game2d", "image") => {
            let offset = canvas_arg_offset(&args);
            let path = arg_string(&args, offset, "");
            let pos = arg_pair(&args, offset + 1, [0, 0])?;
            let size = positive_size(&args, offset + 2, [100, 100])?;
            crate::ui::GLOBAL_UI.add_image(
                &path,
                pos[0] as i32,
                pos[1] as i32,
                size[0],
                size[1],
                None,
            );
            Ok(Value::Bool(true))
        }
        ("ui", "draw_sprite") | ("game2d", "sprite") => {
            let offset = canvas_arg_offset(&args);
            let path = arg_string(&args, offset, "");
            let pos = arg_pair(&args, offset + 1, [0, 0])?;
            let size = positive_size(&args, offset + 2, [100, 100])?;
            let src_pos = arg_pair(&args, offset + 3, [0, 0])?;
            let src_size = positive_size(&args, offset + 4, [100, 100])?;
            crate::ui::GLOBAL_UI.add_image(
                &path,
                pos[0] as i32,
                pos[1] as i32,
                size[0],
                size[1],
                Some([
                    src_pos[0] as f32,
                    src_pos[1] as f32,
                    src_size[0] as f32,
                    src_size[1] as f32,
                ]),
            );
            Ok(Value::Bool(true))
        }
        ("ui", "is_key_pressed") => {
            let key = arg_string(&args, 0, "");
            Ok(Value::Bool(crate::ui::GLOBAL_UI.is_key_pressed(&key)))
        }
        ("ui", "mouse_pos") => {
            let (pos, _, _) = crate::ui::GLOBAL_UI.get_mouse_state();
            Ok(Value::Array(vec![Value::Float(pos.0), Value::Float(pos.1)]))
        }
        ("engine", "load_scene") => {
            if args.is_empty() {
                return Err("engine.load_scene requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            let path = fs_read_path(&path)?;
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read scene file {}: {}", path.display(), e))?;
            parse_json_limited(&content, &format!("scene file {}", path.display()))
        }
        ("engine", "save_scene" | "save_layout") => {
            if args.len() < 2 {
                return Err(format!("engine.{} requires 2 arguments", method));
            }
            let path = value_to_string(&args[0]);
            let path = fs_write_path(&path)?;
            let body = stringify_json_limited(&args[1], method, true)?;
            std::fs::write(&path, body).map_err(|e| {
                format!("Failed to write {} file {}: {}", method, path.display(), e)
            })?;
            Ok(Value::Bool(true))
        }
        ("engine", "load_layout") => {
            if args.is_empty() {
                return Err("engine.load_layout requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            let path = fs_read_path(&path)?;
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read layout file {}: {}", path.display(), e))?;
            parse_json_limited(&content, &format!("layout file {}", path.display()))
        }

        ("audio", "play_sound") => {
            if args.is_empty() {
                return Err("audio.play_sound requires 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            #[cfg(feature = "audio")]
            {
                crate::audio::play_sound(&path)?;
                Ok(Value::Bool(true))
            }
            #[cfg(not(feature = "audio"))]
            {
                let _ = path;
                Err("Audio support is not compiled; rebuild Vice with --features audio".into())
            }
        }
        ("audio", "play_music") => {
            if args.is_empty() {
                return Err("audio.play_music requires at least 1 argument".into());
            }
            let path = value_to_string(&args[0]);
            let volume = args.get(1).map(as_float).unwrap_or(1.0) as f32;
            #[cfg(feature = "audio")]
            {
                crate::audio::play_music(&path, volume)?;
                Ok(Value::Bool(true))
            }
            #[cfg(not(feature = "audio"))]
            {
                let _ = (path, volume);
                Err("Audio support is not compiled; rebuild Vice with --features audio".into())
            }
        }

        // --- MAP ---
        ("map", "len") => {
            if args.is_empty() {
                return Err("map.len requires 1 argument".into());
            }
            if let Value::Map(m) = &args[0] {
                Ok(Value::Int(m.len() as i64))
            } else {
                Err("map.len requires a map".into())
            }
        }
        ("map", "is_empty") => {
            if args.is_empty() {
                return Err("map.is_empty requires 1 argument".into());
            }
            if let Value::Map(m) = &args[0] {
                Ok(Value::Bool(m.is_empty()))
            } else {
                Err("map.is_empty requires a map".into())
            }
        }
        ("map", "contains_key") => {
            if args.len() < 2 {
                return Err("map.contains_key requires 2 arguments (map, key)".into());
            }
            if let Value::Map(m) = &args[0] {
                let key = MapKey::from_value(&args[1]).ok_or_else(|| "Invalid map key type".to_string())?;
                Ok(Value::Bool(m.contains_key(&key)))
            } else {
                Err("map.contains_key requires a map".into())
            }
        }
        ("map", "keys") => {
            if args.is_empty() {
                return Err("map.keys requires 1 argument".into());
            }
            if let Value::Map(m) = &args[0] {
                let mut keys: Vec<Value> = m.keys().map(|k| k.to_value()).collect();
                keys.sort_by(|a, b| a.compare_order(b));
                Ok(Value::Array(keys))
            } else {
                Err("map.keys requires a map".into())
            }
        }
        ("map", "values") => {
            if args.is_empty() {
                return Err("map.values requires 1 argument".into());
            }
            if let Value::Map(m) = &args[0] {
                let mut pairs: Vec<(&MapKey, &Value)> = m.iter().collect();
                pairs.sort_by(|(k1, _), (k2, _)| k1.to_value().compare_order(&k2.to_value()));
                let values: Vec<Value> = pairs.into_iter().map(|(_, v)| v.clone()).collect();
                Ok(Value::Array(values))
            } else {
                Err("map.values requires a map".into())
            }
        }
        ("map", "get") => {
            if args.len() < 3 {
                return Err("map.get requires 3 arguments (map, key, default)".into());
            }
            if let Value::Map(m) = &args[0] {
                if let Some(key) = MapKey::from_value(&args[1]) {
                    if let Some(val) = m.get(&key) {
                        return Ok(val.clone());
                    }
                }
                Ok(args[2].clone())
            } else {
                Err("map.get requires a map".into())
            }
        }
        ("map", "set") => {
            if args.len() < 3 {
                return Err("map.set requires 3 arguments (map, key, val)".into());
            }
            let key = match MapKey::from_value(&args[1]) {
                Some(k) => k,
                None => return Err("Invalid map key".into()),
            };
            if let Value::Map(m) = &args[0] {
                let mut new_m = m.clone();
                new_m.insert(key, args[2].clone());
                Ok(Value::Map(new_m))
            } else {
                Err("map.set requires a map as first argument".into())
            }
        }
        ("map", "remove") => {
            if args.len() < 2 {
                return Err("map.remove requires 2 arguments (map, key)".into());
            }
            if let Value::Map(m) = &args[0] {
                let mut new_map = m.clone();
                if let Some(key) = MapKey::from_value(&args[1]) {
                    new_map.remove(&key);
                }
                Ok(Value::Map(new_map))
            } else {
                Err("map.remove requires a map".into())
            }
        }
        ("map", "merge") => {
            if args.len() < 2 {
                return Err("map.merge requires 2 arguments (map1, map2)".into());
            }
            match (&args[0], &args[1]) {
                (Value::Map(m1), Value::Map(m2)) => {
                    let mut merged = m1.clone();
                    for (k, v) in m2 {
                        merged.insert(k.clone(), v.clone());
                    }
                    if merged.len() > crate::vm::machine::MAX_MAP_SIZE {
                        return Err("Map size exceeds maximum limit of 10,000,000".into());
                    }
                    Ok(Value::Map(merged))
                }
                _ => Err("map.merge requires two maps".into()),
            }
        }
        ("map", "clear") => {
            if args.is_empty() {
                return Err("map.clear requires 1 argument".into());
            }
            if matches!(&args[0], Value::Map(_)) {
                Ok(Value::Map(HashMap::new()))
            } else {
                Err("map.clear requires a map".into())
            }
        }
        ("map", "has_value") => {
            if args.len() < 2 {
                return Err("map.has_value requires 2 arguments (m, value)".into());
            }
            let Value::Map(m) = &args[0] else {
                return Err("map.has_value requires a map as first argument".into());
            };
            let found = m.values().any(|v| v == &args[1]);
            Ok(Value::Bool(found))
        }
        ("map", "invert") => {
            if args.is_empty() {
                return Err("map.invert requires 1 argument (m)".into());
            }
            let Value::Map(m) = &args[0] else {
                return Err("map.invert requires a map".into());
            };
            let mut inverted = HashMap::new();
            for (k, v) in m {
                let inv_key = MapKey::from_value(v).ok_or_else(|| {
                    "Cannot invert map: value cannot be converted to a map key".to_string()
                })?;
                inverted.insert(inv_key, k.to_value());
            }
            Ok(Value::Map(inverted))
        }
        // --- HASH ---
        ("hash", "fnv1a") => {
            if args.is_empty() {
                return Err("hash.fnv1a requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut hash: u64 = 0xcbf29ce484222325;
            for byte in s.as_bytes() {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Ok(Value::Int(hash as i64))
        }
        ("hash", "djb2") => {
            if args.is_empty() {
                return Err("hash.djb2 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut hash: u64 = 5381;
            for byte in s.as_bytes() {
                hash = hash.wrapping_mul(33).wrapping_add(*byte as u64);
            }
            Ok(Value::Int(hash as i64))
        }
        ("hash", "adler32") => {
            if args.is_empty() {
                return Err("hash.adler32 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut a: u32 = 1;
            let mut b: u32 = 0;
            for byte in s.as_bytes() {
                a = (a + *byte as u32) % 65521;
                b = (b + a) % 65521;
            }
            let adler = ((b << 16) | a) as i64;
            Ok(Value::Int(adler))
        }
        ("hash", "crc32") => {
            if args.is_empty() {
                return Err("hash.crc32 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let mut crc: u32 = 0xFFFFFFFF;
            for byte in s.as_bytes() {
                crc ^= *byte as u32;
                for _ in 0..8 {
                    crc = if (crc & 1) != 0 {
                        (crc >> 1) ^ 0xEDB88320
                    } else {
                        crc >> 1
                    };
                }
            }
            let res = (!crc) as i64;
            Ok(Value::Int(res))
        }
        ("hash", "combine") => {
            if args.len() < 2 {
                return Err("hash.combine requires 2 arguments (h1, h2)".into());
            }
            let h1 = as_int(&args[0]) as u64;
            let h2 = as_int(&args[1]) as u64;
            let combined = h1 ^ (h2.wrapping_add(0x9e3779b97f4a7c15).wrapping_add(h1 << 6).wrapping_add(h1 >> 2));
            Ok(Value::Int(combined as i64))
        }
        ("hash", "murmur3") => {
            if args.is_empty() {
                return Err("hash.murmur3 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let hash = murmur3_32(s.as_bytes(), 0);
            Ok(Value::Int(hash as i64))
        }
        ("hash", "xxhash32") => {
            if args.is_empty() {
                return Err("hash.xxhash32 requires at least 1 argument (s, [seed])".into());
            }
            let s = value_to_string(&args[0]);
            let seed = if args.len() > 1 { as_int(&args[1]) as u32 } else { 0 };
            let hash = xxhash32(s.as_bytes(), seed);
            Ok(Value::Int(hash as i64))
        }
        ("hash", "rot13") => {
            if args.is_empty() {
                return Err("hash.rot13 requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            let rotated: String = s.chars().map(|c| match c {
                'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
                'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
                _ => c,
            }).collect();
            Ok(Value::String(rotated))
        }
        ("hash", "hex_encode") => {
            if args.is_empty() {
                return Err("hash.hex_encode requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(hex_encode(&s)))
        }
        ("hash", "hex_decode") => {
            if args.is_empty() {
                return Err("hash.hex_decode requires 1 argument (s)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(hex_decode(&s)))
        }

        // --- FFI ---
        ("ffi", "load") => {
            if args.is_empty() {
                return Err("ffi.load requires 1 argument (path)".into());
            }
            let path = value_to_string(&args[0]);
            crate::ffi::load(&path)
        }
        ("ffi", "free") => {
            if args.is_empty() {
                return Err("ffi.free requires 1 argument (handle)".into());
            }
            crate::ffi::free(&args[0])
        }
        ("ffi", "call") => {
            if args.len() < 4 {
                return Err("ffi.call requires 4 arguments (handle, symbol, args_array, return_type)".into());
            }
            let symbol = value_to_string(&args[1]);
            let empty = Vec::new();
            let call_args = match &args[2] {
                Value::Array(a) => a.as_slice(),
                _ => &empty,
            };
            let ret_type = value_to_string(&args[3]);
            crate::ffi::call(&args[0], &symbol, call_args, &ret_type)
        }

        // --- HTTP ---
        ("http", "get") => {
            if args.is_empty() {
                return Err("http.get requires at least 1 argument (url)".into());
            }
            let url = value_to_string(&args[0]);
            let headers = if args.len() > 1 {
                match &args[1] {
                    Value::Map(m) => Some(m),
                    _ => None,
                }
            } else {
                None
            };
            crate::net_http::get(&url, headers)
        }
        ("http", "post") => {
            if args.len() < 2 {
                return Err("http.post requires at least 2 arguments (url, body)".into());
            }
            let url = value_to_string(&args[0]);
            let body = value_to_string(&args[1]);
            let headers = if args.len() > 2 {
                match &args[2] {
                    Value::Map(m) => Some(m),
                    _ => None,
                }
            } else {
                None
            };
            crate::net_http::post(&url, &body, headers)
        }

        // --- MATRIX ---
        ("matrix", "zeros") => {
            if args.len() < 2 {
                return Err("matrix.zeros requires 2 arguments (rows, cols)".into());
            }
            crate::matrix::zeros(as_int(&args[0]), as_int(&args[1]))
        }
        ("matrix", "from_array") => {
            if args.len() < 3 {
                return Err("matrix.from_array requires 3 arguments (data, rows, cols)".into());
            }
            let empty = Vec::new();
            let arr = match &args[0] {
                Value::Array(a) => a.as_slice(),
                _ => &empty,
            };
            crate::matrix::from_array(arr, as_int(&args[1]), as_int(&args[2]))
        }
        ("matrix", "to_array") => {
            if args.is_empty() {
                return Err("matrix.to_array requires 1 argument (matrix)".into());
            }
            crate::matrix::to_array(&args[0])
        }
        ("matrix", "shape") => {
            if args.is_empty() {
                return Err("matrix.shape requires 1 argument (matrix)".into());
            }
            crate::matrix::shape(&args[0])
        }
        ("matrix", "add") => {
            if args.len() < 2 {
                return Err("matrix.add requires 2 arguments (m1, m2)".into());
            }
            crate::matrix::add(&args[0], &args[1])
        }
        ("matrix", "sub") => {
            if args.len() < 2 {
                return Err("matrix.sub requires 2 arguments (m1, m2)".into());
            }
            crate::matrix::sub(&args[0], &args[1])
        }
        ("matrix", "mul") => {
            if args.len() < 2 {
                return Err("matrix.mul requires 2 arguments (m1, m2)".into());
            }
            crate::matrix::mul(&args[0], &args[1])
        }
        ("matrix", "scale") => {
            if args.len() < 2 {
                return Err("matrix.scale requires 2 arguments (matrix, scalar)".into());
            }
            crate::matrix::scale(&args[0], as_float(&args[1]))
        }
        ("matrix", "transpose") => {
            if args.is_empty() {
                return Err("matrix.transpose requires 1 argument (matrix)".into());
            }
            crate::matrix::transpose(&args[0])
        }
        ("matrix", "dot") => {
            if args.len() < 2 {
                return Err("matrix.dot requires 2 arguments (v1, v2)".into());
            }
            let empty = Vec::new();
            let v1 = match &args[0] {
                Value::Array(a) => a.as_slice(),
                _ => &empty,
            };
            let v2 = match &args[1] {
                Value::Array(a) => a.as_slice(),
                _ => &empty,
            };
            crate::matrix::dot(v1, v2).map(Value::Float)
        }
        ("matrix", "sum") => {
            if args.is_empty() {
                return Err("matrix.sum requires 1 argument (matrix)".into());
            }
            crate::matrix::sum(&args[0]).map(Value::Float)
        }

        // --- CRYPTO ---
        ("crypto", "sha256") => {
            if args.is_empty() {
                return Err("crypto.sha256 requires 1 argument (text)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(crate::crypto::sha256_hex(&s)))
        }
        ("crypto", "sha512") => {
            if args.is_empty() {
                return Err("crypto.sha512 requires 1 argument (text)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(crate::crypto::sha512_hex(&s)))
        }
        ("crypto", "base64_encode") => {
            if args.is_empty() {
                return Err("crypto.base64_encode requires 1 argument (text)".into());
            }
            let s = value_to_string(&args[0]);
            Ok(Value::String(crate::crypto::base64_encode_str(&s)))
        }
        ("crypto", "base64_decode") => {
            if args.is_empty() {
                return Err("crypto.base64_decode requires 1 argument (b64_string)".into());
            }
            let s = value_to_string(&args[0]);
            crate::crypto::base64_decode_str(&s).map(Value::String)
        }
        ("crypto", "random_bytes") => {
            if args.is_empty() {
                return Err("crypto.random_bytes requires 1 argument (count)".into());
            }
            let count = as_int(&args[0]);
            if count <= 0 {
                return Err("crypto.random_bytes count must be positive".into());
            }
            crate::crypto::secure_random_bytes(count as usize).map(Value::Array)
        }

        // --- REGEX ---
        ("regex", "is_match") => {
            if args.len() < 2 {
                return Err("regex.is_match requires 2 arguments (pattern, text)".into());
            }
            let pattern = value_to_string(&args[0]);
            let text = value_to_string(&args[1]);
            crate::reg_engine::is_match(&pattern, &text).map(Value::Bool)
        }
        ("regex", "find") => {
            if args.len() < 2 {
                return Err("regex.find requires 2 arguments (pattern, text)".into());
            }
            let pattern = value_to_string(&args[0]);
            let text = value_to_string(&args[1]);
            crate::reg_engine::find(&pattern, &text).map(|opt| match opt {
                Some(s) => Value::String(s),
                None => Value::Unit,
            })
        }
        ("regex", "find_all") => {
            if args.len() < 2 {
                return Err("regex.find_all requires 2 arguments (pattern, text)".into());
            }
            let pattern = value_to_string(&args[0]);
            let text = value_to_string(&args[1]);
            crate::reg_engine::find_all(&pattern, &text).map(Value::Array)
        }
        ("regex", "replace") => {
            if args.len() < 3 {
                return Err("regex.replace requires 3 arguments (pattern, text, replacement)".into());
            }
            let pattern = value_to_string(&args[0]);
            let text = value_to_string(&args[1]);
            let repl = value_to_string(&args[2]);
            crate::reg_engine::replace(&pattern, &text, &repl).map(Value::String)
        }
        ("regex", "split") => {
            if args.len() < 2 {
                return Err("regex.split requires 2 arguments (pattern, text)".into());
            }
            let pattern = value_to_string(&args[0]);
            let text = value_to_string(&args[1]);
            crate::reg_engine::split(&pattern, &text).map(Value::Array)
        }

        // --- SQLITE ---
        ("sqlite", "open") => {
            if args.is_empty() {
                return Err("sqlite.open requires 1 argument (path)".into());
            }
            let path = value_to_string(&args[0]);
            crate::db_sqlite::open(&path)
        }
        ("sqlite", "close") => {
            if args.is_empty() {
                return Err("sqlite.close requires 1 argument (handle)".into());
            }
            crate::db_sqlite::close(&args[0])
        }
        ("sqlite", "execute") => {
            if args.len() < 2 {
                return Err("sqlite.execute requires at least 2 arguments (handle, sql)".into());
            }
            let sql = value_to_string(&args[1]);
            let empty = Vec::new();
            let p = if args.len() > 2 {
                match &args[2] {
                    Value::Array(a) => a.as_slice(),
                    _ => &empty,
                }
            } else {
                &empty
            };
            crate::db_sqlite::execute(&args[0], &sql, p)
        }
        ("sqlite", "query") => {
            if args.len() < 2 {
                return Err("sqlite.query requires at least 2 arguments (handle, sql)".into());
            }
            let sql = value_to_string(&args[1]);
            let empty = Vec::new();
            let p = if args.len() > 2 {
                match &args[2] {
                    Value::Array(a) => a.as_slice(),
                    _ => &empty,
                }
            } else {
                &empty
            };
            crate::db_sqlite::query(&args[0], &sql, p)
        }
        ("sqlite", "last_insert_rowid") => {
            if args.is_empty() {
                return Err("sqlite.last_insert_rowid requires 1 argument (handle)".into());
            }
            crate::db_sqlite::last_insert_rowid(&args[0])
        }
        _ => Err(format!(
            "Native method not implemented in VM: {}.{}",
            module, method
        )),
    }
}

pub fn murmur3_32(data: &[u8], seed: u32) -> u32 {
    let c1: u32 = 0xcc9e2d51;
    let c2: u32 = 0x1b873593;
    let mut h1 = seed;
    let len = data.len();
    let nblocks = len / 4;

    for i in 0..nblocks {
        let chunk = &data[i * 4..(i + 1) * 4];
        let mut k1 = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(c2);

        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5).wrapping_add(0xe6546b64);
    }

    let tail = &data[nblocks * 4..];
    let mut k1: u32 = 0;
    match tail.len() {
        3 => {
            k1 ^= (tail[2] as u32) << 16;
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        2 => {
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        1 => {
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        _ => {}
    }

    h1 ^= len as u32;
    h1 ^= h1 >> 16;
    h1 = h1.wrapping_mul(0x85ebca6b);
    h1 ^= h1 >> 13;
    h1 = h1.wrapping_mul(0xc2b2ae35);
    h1 ^= h1 >> 16;

    h1
}

pub fn xxhash32(input: &[u8], seed: u32) -> u32 {
    const PRIME32_1: u32 = 0x9E3779B1;
    const PRIME32_2: u32 = 0x85EBCA77;
    const PRIME32_3: u32 = 0xC2B2AE3D;
    const PRIME32_4: u32 = 0x27D4EB2F;
    const PRIME32_5: u32 = 0x165667B1;

    let len = input.len();
    let mut h32: u32;

    if len >= 16 {
        let mut v1 = seed.wrapping_add(PRIME32_1).wrapping_add(PRIME32_2);
        let mut v2 = seed.wrapping_add(PRIME32_2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME32_1);

        let limit = len - 16;
        let mut offset = 0;
        while offset <= limit {
            let chunk = &input[offset..offset + 16];
            let read32 = |o: usize| u32::from_le_bytes([chunk[o], chunk[o + 1], chunk[o + 2], chunk[o + 3]]);
            v1 = v1.wrapping_add(read32(0).wrapping_mul(PRIME32_2)).rotate_left(13).wrapping_mul(PRIME32_1);
            v2 = v2.wrapping_add(read32(4).wrapping_mul(PRIME32_2)).rotate_left(13).wrapping_mul(PRIME32_1);
            v3 = v3.wrapping_add(read32(8).wrapping_mul(PRIME32_2)).rotate_left(13).wrapping_mul(PRIME32_1);
            v4 = v4.wrapping_add(read32(12).wrapping_mul(PRIME32_2)).rotate_left(13).wrapping_mul(PRIME32_1);
            offset += 16;
        }

        h32 = v1.rotate_left(1).wrapping_add(v2.rotate_left(7)).wrapping_add(v3.rotate_left(12)).wrapping_add(v4.rotate_left(18));
    } else {
        h32 = seed.wrapping_add(PRIME32_5);
    }

    h32 = h32.wrapping_add(len as u32);

    let mut remaining = &input[(len / 16) * 16..];
    while remaining.len() >= 4 {
        let val = u32::from_le_bytes([remaining[0], remaining[1], remaining[2], remaining[3]]);
        h32 = h32.wrapping_add(val.wrapping_mul(PRIME32_3)).rotate_left(17).wrapping_mul(PRIME32_4);
        remaining = &remaining[4..];
    }
    for &byte in remaining {
        h32 = h32.wrapping_add((byte as u32).wrapping_mul(PRIME32_5)).rotate_left(11).wrapping_mul(PRIME32_1);
    }

    h32 ^= h32 >> 15;
    h32 = h32.wrapping_mul(PRIME32_2);
    h32 ^= h32 >> 13;
    h32 = h32.wrapping_mul(PRIME32_3);
    h32 ^= h32 >> 16;
    h32
}

pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row = vec![0; len2 + 1];

    for i in 0..len1 {
        curr_row[0] = i + 1;
        for j in 0..len2 {
            let cost = if s1_chars[i] == s2_chars[j] { 0 } else { 1 };
            curr_row[j + 1] = std::cmp::min(
                std::cmp::min(curr_row[j] + 1, prev_row[j + 1] + 1),
                prev_row[j] + cost,
            );
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[len2]
}

pub fn hamming_distance(s1: &str, s2: &str) -> i64 {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    if s1_chars.len() != s2_chars.len() {
        return -1;
    }
    let mut dist = 0i64;
    for i in 0..s1_chars.len() {
        if s1_chars[i] != s2_chars[i] {
            dist += 1;
        }
    }
    dist
}

pub fn slugify(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_dash = true;

    for c in s.chars() {
        if c.is_alphanumeric() {
            for lower in c.to_lowercase() {
                result.push(lower);
            }
            last_was_dash = false;
        } else if (c == ' ' || c == '-' || c == '_' || c == '.') && !last_was_dash {
            result.push('-');
            last_was_dash = true;
        }
    }
    if result.ends_with('-') {
        result.pop();
    }
    result
}

pub fn truncate_string(s: &str, max_len: usize, ellipsis: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        s.to_string()
    } else {
        let el_chars: Vec<char> = ellipsis.chars().collect();
        if max_len <= el_chars.len() {
            chars.into_iter().take(max_len).collect()
        } else {
            let keep_len = max_len - el_chars.len();
            let mut res: String = chars.into_iter().take(keep_len).collect();
            res.push_str(ellipsis);
            res
        }
    }
}

pub fn hex_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.as_bytes() {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

pub fn hex_decode(s: &str) -> String {
    if s.len() % 2 != 0 {
        return String::new();
    }
    let bytes: Option<Vec<u8>> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect();
    match bytes {
        Some(b) => String::from_utf8(b).unwrap_or_default(),
        None => String::new(),
    }
}

pub fn clamp01_f64(val: f64) -> f64 {
    if val.is_nan() {
        0.0
    } else {
        val.clamp(0.0, 1.0)
    }
}

pub fn lerp_inv_f64(a: f64, b: f64, v: f64) -> f64 {
    if (b - a).abs() < f64::EPSILON {
        0.0
    } else {
        (v - a) / (b - a)
    }
}

pub fn remap_f64(v: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    let t = lerp_inv_f64(in_min, in_max, v);
    out_min + t * (out_max - out_min)
}

pub fn midpoint_i64(a: i64, b: i64) -> i64 {
    (a & b) + ((a ^ b) >> 1)
}

pub fn approx_eq_f64(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps.abs()
}

pub fn is_array_sorted(arr: &[Value]) -> bool {
    if arr.len() <= 1 {
        return true;
    }
    for window in arr.windows(2) {
        if window[0].compare_order(&window[1]) == std::cmp::Ordering::Greater {
            return false;
        }
    }
    true
}

pub fn are_array_elements_unique(arr: &[Value]) -> bool {
    for i in 0..arr.len() {
        for j in (i + 1)..arr.len() {
            if arr[i] == arr[j] {
                return false;
            }
        }
    }
    true
}

pub fn is_string_ascii(s: &str) -> bool {
    s.is_ascii()
}

pub fn is_string_alphanumeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric())
}

pub fn count_string_occurrences(s: &str, sub: &str) -> i64 {
    if sub.is_empty() {
        return 0;
    }
    s.matches(sub).count() as i64
}

pub fn pad_zero_string(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= width {
        s.to_string()
    } else {
        let diff = width - char_count;
        format!("{}{}", "0".repeat(diff), s)
    }
}

pub fn hypot3_f64(x: f64, y: f64, z: f64) -> f64 {
    (x * x + y * y + z * z).sqrt()
}

pub fn clamp_int_i64(v: i64, min: i64, max: i64) -> i64 {
    v.clamp(min, max)
}

pub fn is_integer_square(n: i64) -> bool {
    if n < 0 {
        return false;
    }
    if n == 0 || n == 1 {
        return true;
    }
    let s = (n as f64).sqrt().round() as i64;
    s * s == n
}

pub fn string_lines_count(s: &str) -> i64 {
    if s.is_empty() {
        0
    } else {
        s.lines().count() as i64
    }
}

pub fn string_words_count(s: &str) -> i64 {
    s.split_whitespace().count() as i64
}



