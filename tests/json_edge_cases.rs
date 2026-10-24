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
fn json_parse_basic() {
    let code = r#"
@entry
fn main() {
    let obj => json.parse("{\"name\": \"vice\", \"version\": 1.0}");
    output <- obj["name"];
    output <- obj["version"];
}
"#;
    assert_eq!(run_vice_code(code), "vice\n1\n");
}

#[test]
fn json_parse_nested() {
    let code = r#"
@entry
fn main() {
    let obj => json.parse("{\"root\": {\"items\": [10, 20, {\"deep\": true}]}}");
    output <- obj["root"]["items"][0];
    output <- obj["root"]["items"][2]["deep"];
}
"#;
    assert_eq!(run_vice_code(code), "10\ntrue\n");
}

#[test]
fn json_parse_invalid_syntax() {
    let code = r#"
@entry
fn main() {
    let obj => json.parse("{invalid}");
}
"#;
    let err = run_vice_code_err(code);
    assert!(
        err.contains("key must be a string"),
        "Expected JSON error, got: {}",
        err
    );
}

#[test]
fn json_parse_empty_string() {
    let code = r#"
@entry
fn main() {
    let obj => json.parse("");
}
"#;
    let err = run_vice_code_err(code);
    assert!(
        err.contains("EOF while parsing a value"),
        "Expected JSON EOF error, got: {}",
        err
    );
}

#[test]
fn json_parse_whitespace() {
    let code = r#"
@entry
fn main() {
    let obj => json.parse("   {   \"key\" :   \"value\"  }   ");
    output <- obj["key"];
}
"#;
    assert_eq!(run_vice_code(code), "value\n");
}

#[test]
fn json_parse_null() {
    let code = r#"
@entry
fn main() {
    let obj => json.parse("null");
    output <- to_str(obj);
}
"#;
    // Vice has no null, so it becomes an empty map as per `interpreter.rs` line 1777
    let out = run_vice_code(code);
    assert!(out.contains("{}"));
}

#[test]
fn json_parse_arrays() {
    let code = r#"
@entry
fn main() {
    let obj => json.parse("[1, \"two\", false, [3, 4]]");
    output <- obj[0];
    output <- obj[1];
    output <- obj[2];
    output <- obj[3][1];
}
"#;
    assert_eq!(run_vice_code(code), "1\ntwo\nfalse\n4\n");
}

#[test]
fn json_parse_instance_mapping() {
    let code = r#"
@entry
fn main() {
    let obj => json.parse("{\"__type__\": \"MyType\", \"health\": 100}");
    output <- to_str(obj);
}
"#;
    let out = run_vice_code(code);
    assert!(out.contains("MyType(health: 100)"));
}
