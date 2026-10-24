use crate::value::Value;
use regex::Regex;

pub fn is_match(pattern: &str, text: &str) -> Result<bool, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
    Ok(re.is_match(text))
}

pub fn find(pattern: &str, text: &str) -> Result<Option<String>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
    Ok(re.find(text).map(|m| m.as_str().to_string()))
}

pub fn find_all(pattern: &str, text: &str) -> Result<Vec<Value>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
    let matches = re
        .find_iter(text)
        .map(|m| Value::String(m.as_str().to_string()))
        .collect();
    Ok(matches)
}

pub fn replace(pattern: &str, text: &str, replacement: &str) -> Result<String, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
    Ok(re.replace_all(text, replacement).to_string())
}

pub fn replace_one(pattern: &str, text: &str, replacement: &str) -> Result<String, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
    Ok(re.replace(text, replacement).to_string())
}

pub fn split(pattern: &str, text: &str) -> Result<Vec<Value>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
    let parts = re
        .split(text)
        .map(|s| Value::String(s.to_string()))
        .collect();
    Ok(parts)
}
