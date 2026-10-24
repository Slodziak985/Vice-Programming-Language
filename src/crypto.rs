use base64::prelude::*;
use ring::digest::{digest, SHA256, SHA512};
use ring::rand::{SecureRandom, SystemRandom};
use crate::value::Value;

pub fn sha256_hex(input: &str) -> String {
    let actual = digest(&SHA256, input.as_bytes());
    hex_encode(actual.as_ref())
}

pub fn sha512_hex(input: &str) -> String {
    let actual = digest(&SHA512, input.as_bytes());
    hex_encode(actual.as_ref())
}

pub fn base64_encode_str(input: &str) -> String {
    BASE64_STANDARD.encode(input.as_bytes())
}

pub fn base64_decode_str(input: &str) -> Result<String, String> {
    let bytes = BASE64_STANDARD
        .decode(input.trim())
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("UTF-8 decode error from Base64: {}", e))
}

pub fn secure_random_bytes(count: usize) -> Result<Vec<Value>, String> {
    if count > 1024 * 1024 {
        return Err("random_bytes limit exceeded (max 1MB)".to_string());
    }
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; count];
    rng.fill(&mut bytes)
        .map_err(|_| "System entropy generator failed".to_string())?;
    Ok(bytes.into_iter().map(|b| Value::Int(b as i64)).collect())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}
