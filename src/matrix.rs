use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn instance(data_name: &str, fields: HashMap<String, Value>) -> Value {
    Value::Instance {
        data_name: data_name.to_string(),
        field_types: HashMap::new(),
        fields: Rc::new(RefCell::new(fields)),
    }
}

pub fn make_matrix(rows: usize, cols: usize, data: Vec<f64>) -> Value {
    let array_values = data.into_iter().map(Value::Float).collect();
    instance(
        "Matrix",
        HashMap::from([
            ("rows".to_string(), Value::Int(rows as i64)),
            ("cols".to_string(), Value::Int(cols as i64)),
            ("data".to_string(), Value::Array(array_values)),
        ]),
    )
}

pub fn extract_matrix(val: &Value) -> Result<(usize, usize, Vec<f64>), String> {
    match val {
        Value::Instance { data_name, fields, .. } if data_name == "Matrix" => {
            let f = fields.borrow();
            let rows = match f.get("rows") {
                Some(Value::Int(r)) if *r > 0 => *r as usize,
                _ => return Err("Invalid matrix rows".to_string()),
            };
            let cols = match f.get("cols") {
                Some(Value::Int(c)) if *c > 0 => *c as usize,
                _ => return Err("Invalid matrix cols".to_string()),
            };
            let data = match f.get("data") {
                Some(Value::Array(arr)) => {
                    let mut nums = Vec::with_capacity(arr.len());
                    for item in arr {
                        match item {
                            Value::Float(n) => nums.push(*n),
                            Value::Int(n) => nums.push(*n as f64),
                            _ => return Err("Matrix data must contain numbers".to_string()),
                        }
                    }
                    nums
                }
                _ => return Err("Invalid matrix data".to_string()),
            };
            if data.len() != rows * cols {
                return Err(format!(
                    "Matrix shape mismatch: expected {} elements, got {}",
                    rows * cols,
                    data.len()
                ));
            }
            Ok((rows, cols, data))
        }
        _ => Err("Expected Matrix instance".to_string()),
    }
}

pub fn zeros(rows: i64, cols: i64) -> Result<Value, String> {
    if rows <= 0 || cols <= 0 {
        return Err("Matrix dimensions must be positive".to_string());
    }
    let count = (rows * cols) as usize;
    Ok(make_matrix(rows as usize, cols as usize, vec![0.0; count]))
}

pub fn from_array(arr: &[Value], rows: i64, cols: i64) -> Result<Value, String> {
    if rows <= 0 || cols <= 0 {
        return Err("Matrix dimensions must be positive".to_string());
    }
    let expected = (rows * cols) as usize;
    if arr.len() != expected {
        return Err(format!(
            "Array length {} does not match matrix dimensions {}x{} (total {})",
            arr.len(),
            rows,
            cols,
            expected
        ));
    }
    let mut data = Vec::with_capacity(expected);
    for item in arr {
        match item {
            Value::Float(f) => data.push(*f),
            Value::Int(i) => data.push(*i as f64),
            _ => return Err("Matrix elements must be numbers".to_string()),
        }
    }
    Ok(make_matrix(rows as usize, cols as usize, data))
}

pub fn shape(mat: &Value) -> Result<Value, String> {
    let (rows, cols, _) = extract_matrix(mat)?;
    Ok(Value::Array(vec![Value::Int(rows as i64), Value::Int(cols as i64)]))
}

pub fn to_array(mat: &Value) -> Result<Value, String> {
    let (_, _, data) = extract_matrix(mat)?;
    Ok(Value::Array(data.into_iter().map(Value::Float).collect()))
}

pub fn add(m1: &Value, m2: &Value) -> Result<Value, String> {
    let (r1, c1, d1) = extract_matrix(m1)?;
    let (r2, c2, d2) = extract_matrix(m2)?;
    if r1 != r2 || c1 != c2 {
        return Err(format!(
            "Matrix addition shape mismatch: {}x{} vs {}x{}",
            r1, c1, r2, c2
        ));
    }
    let mut result = Vec::with_capacity(d1.len());
    for i in 0..d1.len() {
        result.push(d1[i] + d2[i]);
    }
    Ok(make_matrix(r1, c1, result))
}

pub fn sub(m1: &Value, m2: &Value) -> Result<Value, String> {
    let (r1, c1, d1) = extract_matrix(m1)?;
    let (r2, c2, d2) = extract_matrix(m2)?;
    if r1 != r2 || c1 != c2 {
        return Err(format!(
            "Matrix subtraction shape mismatch: {}x{} vs {}x{}",
            r1, c1, r2, c2
        ));
    }
    let mut result = Vec::with_capacity(d1.len());
    for i in 0..d1.len() {
        result.push(d1[i] - d2[i]);
    }
    Ok(make_matrix(r1, c1, result))
}

pub fn mul(m1: &Value, m2: &Value) -> Result<Value, String> {
    let (r1, c1, d1) = extract_matrix(m1)?;
    let (r2, c2, d2) = extract_matrix(m2)?;
    if c1 != r2 {
        return Err(format!(
            "Matrix multiplication dimension mismatch: left cols ({}) != right rows ({})",
            c1, r2
        ));
    }
    let mut result = vec![0.0; r1 * c2];
    for i in 0..r1 {
        for k in 0..c1 {
            let left_val = d1[i * c1 + k];
            for j in 0..c2 {
                result[i * c2 + j] += left_val * d2[k * c2 + j];
            }
        }
    }
    Ok(make_matrix(r1, c2, result))
}

pub fn scale(m: &Value, factor: f64) -> Result<Value, String> {
    let (r, c, data) = extract_matrix(m)?;
    let scaled = data.into_iter().map(|v| v * factor).collect();
    Ok(make_matrix(r, c, scaled))
}

pub fn transpose(m: &Value) -> Result<Value, String> {
    let (r, c, data) = extract_matrix(m)?;
    let mut result = vec![0.0; r * c];
    for i in 0..r {
        for j in 0..c {
            result[j * r + i] = data[i * c + j];
        }
    }
    Ok(make_matrix(c, r, result))
}

pub fn dot(v1: &[Value], v2: &[Value]) -> Result<f64, String> {
    if v1.len() != v2.len() {
        return Err(format!(
            "Vector dot product length mismatch: {} vs {}",
            v1.len(),
            v2.len()
        ));
    }
    let mut sum = 0.0;
    for (a, b) in v1.iter().zip(v2.iter()) {
        let val_a = match a {
            Value::Float(f) => *f,
            Value::Int(i) => *i as f64,
            _ => return Err("Vector elements must be numbers".to_string()),
        };
        let val_b = match b {
            Value::Float(f) => *f,
            Value::Int(i) => *i as f64,
            _ => return Err("Vector elements must be numbers".to_string()),
        };
        sum += val_a * val_b;
    }
    Ok(sum)
}

pub fn sum(m: &Value) -> Result<f64, String> {
    let (_, _, data) = extract_matrix(m)?;
    Ok(data.into_iter().sum())
}
