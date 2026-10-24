use crate::value::{MapKey, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

fn instance(data_name: &str, fields: HashMap<String, Value>) -> Value {
    Value::Instance {
        data_name: data_name.to_string(),
        field_types: HashMap::new(),
        fields: Rc::new(RefCell::new(fields)),
    }
}

fn build_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .timeout_write(Duration::from_secs(30))
        .build()
}

fn check_sandbox() -> Result<(), String> {
    if crate::stdlib::sandbox_root().is_some() {
        return Err("HTTP networking is disabled in sandbox mode".to_string());
    }
    Ok(())
}

fn make_response(status: u16, body: String) -> Value {
    let ok = status >= 200 && status < 300;
    instance(
        "HttpResponse",
        HashMap::from([
            ("status".to_string(), Value::Int(status as i64)),
            ("body".to_string(), Value::String(body)),
            ("ok".to_string(), Value::Bool(ok)),
        ]),
    )
}

fn apply_headers(
    mut req: ureq::Request,
    headers: Option<&HashMap<MapKey, Value>>,
) -> ureq::Request {
    if let Some(map) = headers {
        for (k, v) in map {
            let header_name = match k {
                MapKey::String(s) => s.as_str(),
                _ => continue,
            };
            let header_val = match v {
                Value::String(s) => s.as_str(),
                _ => continue,
            };
            req = req.set(header_name, header_val);
        }
    }
    req
}

pub fn get(url: &str, headers: Option<&HashMap<MapKey, Value>>) -> Result<Value, String> {
    check_sandbox()?;
    let agent = build_agent();
    let req = apply_headers(agent.get(url), headers);
    
    match req.call() {
        Ok(res) => {
            let status = res.status();
            let body = res
                .into_string()
                .map_err(|e| format!("Failed to read HTTP response body: {}", e))?;
            Ok(make_response(status, body))
        }
        Err(ureq::Error::Status(code, res)) => {
            let body = res
                .into_string()
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            Ok(make_response(code, body))
        }
        Err(e) => Err(format!("HTTP GET error: {}", e)),
    }
}

pub fn post(
    url: &str,
    body_data: &str,
    headers: Option<&HashMap<MapKey, Value>>,
) -> Result<Value, String> {
    check_sandbox()?;
    let agent = build_agent();
    let mut req = apply_headers(agent.post(url), headers);
    if req.header("Content-Type").is_none() {
        req = req.set("Content-Type", "application/json");
    }

    match req.send_string(body_data) {
        Ok(res) => {
            let status = res.status();
            let body = res
                .into_string()
                .map_err(|e| format!("Failed to read HTTP response body: {}", e))?;
            Ok(make_response(status, body))
        }
        Err(ureq::Error::Status(code, res)) => {
            let body = res
                .into_string()
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            Ok(make_response(code, body))
        }
        Err(e) => Err(format!("HTTP POST error: {}", e)),
    }
}
