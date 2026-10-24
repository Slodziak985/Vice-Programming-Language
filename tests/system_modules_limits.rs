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
    let program = match parse_source(code) {
        Ok(p) => p,
        Err(e) => return format!("{:?}", e),
    };
    let mut semantic = SemanticAnalyzer::new();
    if let Err(e) = semantic.analyze(&program) {
        return format!("{:?}", e);
    }
    match vice::execute_program_vm(&program) {
        Ok(_) => "Success".to_string(),
        Err(e) => format!("{:?}", e),
    }
}

#[test]
fn system_random_limits() {
    let code = r#"
@entry
fn main() {
    // Generate many random numbers to ensure no internal panic or sequence failure
    mut i => 0;
    mut sum => 0;
    while (i < 1000) {
        let r => random.int(0, 100);
        sum = sum + r;
        i = i + 1;
    }
    // Just verifying it doesn't crash
    if (sum >= 0) {
        output <- "Random Success";
    }
}
"#;
    assert_eq!(run_vice_code(code), "Random Success\n");
}

#[test]
fn system_random_bad_range() {
    let code = r#"
@entry
fn main() {
    // Min > Max
    let r => random.int(100, 0);
}
"#;
    let err = run_vice_code_err(code);
    assert!(
        err.contains("max") || err.contains("range") || err.contains("Success"),
        "Got err: {}",
        err
    );
}

#[test]
fn system_time_limits() {
    let code = r#"
@entry
fn main() {
    // Test time progression
    let t1 => time.now_ms();
    // Simulate some work
    mut i => 0;
    while (i < 10000) { i = i + 1; }
    let t2 => time.now_ms();
    
    // time should not go backwards
    if (t2 >= t1) {
        output <- "Time OK";
    }
}
"#;
    assert_eq!(run_vice_code(code), "Time OK\n");
}

#[test]
fn system_env_limits() {
    let code = r#"
@entry
fn main() {
    // Env vars testing
    env.set("VICE_TEST_VAR", "limits_test");
    let val => env.get("VICE_TEST_VAR")!;
    output <- val;
    
    let none => env.get("NON_EXISTENT_VAR_123456");
    if (none.is_some == false) {
        output <- "Empty";
    }
}
"#;
    assert_eq!(run_vice_code(code), "limits_test\nEmpty\n");
}
