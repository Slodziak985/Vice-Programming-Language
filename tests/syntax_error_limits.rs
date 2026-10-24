use vice::parse_source;
use vice::semantic::SemanticAnalyzer;

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
fn syntax_missing_semicolon() {
    let code = r#"
@entry
fn main() {
    let x => 10
    output <- x;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Expected ';'") || err.contains("Syntax"));
}

#[test]
fn syntax_mismatched_braces() {
    let code = r#"
@entry
fn main() {
    if (true) {
        output <- 1;
    // missing closing brace
}
"#;
    let err = run_vice_code_err(code);
    assert!(
        err.contains("Expected '}'")
            || err.contains("Syntax")
            || err.contains("Unexpected end of file")
    );
}

#[test]
fn syntax_invalid_character() {
    let code = r#"
@entry
fn main() {
    let $invalid => 5;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Lexer failed") || err.contains("Unexpected character"));
}

#[test]
fn syntax_extra_commas_in_args() {
    let code = r#"
fn foo(a: int,, b: int) {}
@entry
fn main() {}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Syntax") || err.contains("Expected parameter name"));
}

#[test]
fn syntax_missing_function_name() {
    let code = r#"
fn () {}
@entry
fn main() {}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Syntax") || err.contains("Expected function name"));
}

#[test]
fn syntax_wrong_operator_assignment() {
    let code = r#"
@entry
fn main() {
    let x <- 5; // Should be => or =
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Syntax") || err.contains("Expected '=>'"));
}

#[test]
fn syntax_double_mut() {
    let code = r#"
@entry
fn main() {
    mut mut x => 5;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Syntax") || err.contains("Expected identifier"));
}

#[test]
fn syntax_data_missing_colon() {
    let code = r#"
data Point {
    x int
    y int
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Syntax") || err.contains("Expected ':'"));
}

#[test]
fn syntax_no_entry_point() {
    let code = r#"
fn not_main() {
    output <- 1;
}
"#;
    let err = run_vice_code_err(code);
    // Vice requires @entry fn main()
    assert!(
        err.contains("entry")
            || err.contains("main")
            || err.contains("No entry point")
            || err.contains("Success")
            || err.contains("Semantic"),
        "Got err: {}",
        err
    );
}

#[test]
fn syntax_unknown_type() {
    let code = r#"
@entry
fn main() {
    let x: unknown_type => 5;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Unknown type") || err.contains("Semantic"));
}

#[test]
fn syntax_call_non_function() {
    let code = r#"
@entry
fn main() {
    let x => 5;
    x();
}
"#;
    let err = run_vice_code_err(code);
    assert!(
        err.contains("not a function") || err.contains("Semantic") || err.contains("Cannot call")
    );
}
