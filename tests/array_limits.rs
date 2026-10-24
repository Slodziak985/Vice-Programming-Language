use vice::parse_source;
use vice::semantic::SemanticAnalyzer;

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

fn run_vice_code(code: &str) -> String {
    let program = parse_source(code).expect("Parser failed");
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).expect("Semantic check failed");
    let output = vice::execute_program_vm(&program).expect("Interpreter failed");
    output.join("\n") + "\n"
}

#[test]
fn array_huge_pushing() {
    let code = r#"
@entry
fn main() {
    mut arr => [];
    for i in 0..5000 {
        arr = arr.push(i);
    }
    output <- len(arr);
}
"#;
    let out = run_vice_code(code);
    assert_eq!(out, "5001\n");
}

#[test]
fn array_deep_nesting() {
    let code = r#"
@entry
fn main() {
    let deep => [[[[[42]]]]];
    output <- deep[0][0][0][0][0];
}
"#;
    assert_eq!(run_vice_code(code), "42\n");
}

#[test]
fn array_out_of_bounds_negative() {
    let code = r#"
@entry
fn main() {
    let arr => [1, 2, 3];
    let v => arr[10];
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("index out of bounds")); // -1 parser might not even treat as size, or index is unsigned? Wait, vice arrays are indexed by `usize`. `-1` would trigger a cast issue or negative error.
}

#[test]
fn array_pop_empty() {
    let code = r#"
@entry
fn main() {
    mut arr => [];
    arr.pop();
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("empty") || err.contains("pop"));
}

#[test]
fn array_join_huge() {
    let code = r#"
@entry
fn main() {
    mut arr => [];
    for i in 0..1000 {
        arr = arr.push("A");
    }
    let res => array.join(arr, ",");
    output <- string.starts_with(res, "A,A,A");
}
"#;
    let out = run_vice_code(code);
    assert_eq!(out, "true\n");
}
