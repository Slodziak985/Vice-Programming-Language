use vice::parse_source;
use vice::semantic::SemanticAnalyzer;

fn run_vice_code(code: &str) -> String {
    let program = parse_source(code).expect("Parser failed");
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).expect("Semantic check failed");
    let output = vice::execute_program_vm(&program).expect("Interpreter failed");
    output.join("\n") + "\n"
}

fn run_vice_code_err(code: &str) -> String {
    let program = parse_source(code).expect("Parser failed");
    let mut semantic = SemanticAnalyzer::new();
    if let Err(e) = semantic.analyze(&program) {
        return format!("{:?}", e);
    }
    match vice::execute_program_vm(&program) {
        Ok(_) => "Success".to_string(),
        Err(e) => e.message.clone(),
    }
}

#[test]
fn math_div_by_zero_int() {
    let code = r#"
@entry
fn main() {
    let a => 10;
    let b => 0;
    output <- a / b;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Division by zero"), "Got: {}", err);
}

#[test]
fn math_div_by_zero_float() {
    let code = r#"
@entry
fn main() {
    let a => 10.0;
    let b => 0.0;
    output <- a / b;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Division by zero"), "Got: {}", err);
}

#[test]
fn math_modulo_by_zero() {
    let code = r#"
@entry
fn main() {
    let a => 10;
    let b => 0;
    output <- a % b;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Division by zero") || err.contains("Modulo by zero"));
}

#[test]
fn math_large_int_overflow_wrapping() {
    // i64 max is 9223372036854775807
    // In Rust, overflow panics in debug, but wrap in release unless explicitly checked.
    // Assuming Vice relies on standard Rust math operators.
    let code = r#"
@entry
fn main() {
    let a: int => 9223372036854775800;
    let b: int => 100;
    output <- a + b;
}
"#;
    let out = run_vice_code(code);
    assert!(out.contains("-"), "Got: {}", out);
}

#[test]
fn math_max_min_limits() {
    let code = r#"
@entry
fn main() {
    output <- math.max(10, 20);
    output <- math.min(-50, -100);
    output <- math.max(10.5, 10.6);
    output <- math.min(10.5, 10.6);
}
"#;
    assert_eq!(run_vice_code(code), "20\n-100\n10.6\n10.5\n");
}

#[test]
fn math_nested_operations() {
    let code = r#"
@entry
fn main() {
    let a => (((10 + 20) * 3) / 2) - 5;
    output <- a;
}
"#;
    assert_eq!(run_vice_code(code), "40\n");
}

#[test]
fn math_large_float_precision() {
    let code = r#"
@entry
fn main() {
    let a => 123456789.123456789;
    let b => 0.000000001;
    output <- a + b;
}
"#;
    let out = run_vice_code(code);
    assert!(out.contains("123456789.1234567"));
}

#[test]
fn math_nan_handling() {
    let code = r#"
@entry
fn main() {
    let a => 0.0;
    let b => 0.0;
    let c => a / b;
    output <- c;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Division by zero"));
}

#[test]
fn math_heavy_loop_operations() {
    let code = r#"
@entry
fn main() {
    mut val => 0;
    for i in 0..1000 {
        val += i;
    }
    output <- val;
}
"#;
    assert_eq!(run_vice_code(code), "500500\n");
}

#[test]
fn math_heavy_power_simulation() {
    // Since math.pow isn't natively exposed? We simulate power.
    let code = r#"
@entry
fn main() {
    mut base => 2;
    mut result => 1;
    for i in 0..10 {
        result = result * base;
    }
    output <- result;
}
"#;
    assert_eq!(run_vice_code(code), "2048\n");
}
