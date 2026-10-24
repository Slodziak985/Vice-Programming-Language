use std::fs;
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
fn path_extreme_join() {
    // Joining many paths to check limits and memory leaks
    let code = r#"
@entry
fn main() {
    mut p => "/root";
    for i in 0..100 {
        p = path.join(p, "dir");
    }
    output <- path.basename(p);
}
"#;
    assert_eq!(run_vice_code(code), "dir\n");
}

#[test]
fn fs_read_non_existent_file() {
    let code = r#"
@entry
fn main() {
    let file => "this_file_definitely_does_not_exist_in_the_multiverse.txt";
    let content => fs.read_text(file);
    output <- content;
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("Failed to read"), "Got: {}", err);
}

#[test]
fn path_empty_operations() {
    let code = r#"
@entry
fn main() {
    let p => "";
    output <- path.basename(p);
    output <- path.dirname(p);
    output <- path.exists(p);
}
"#;
    let out = run_vice_code(code);
    assert!(out.contains("false")); // path.exists("") is false usually
}

#[test]
fn fs_write_and_read_huge_file() {
    let dir = std::env::temp_dir().join("vice_test_limits");
    let _ = fs::create_dir_all(&dir);
    let target = dir.join("huge.txt");
    let target_str = target.to_str().unwrap().replace("\\", "\\\\"); // Windows backslash escape

    let code = format!(
        r#"
@entry
fn main() {{
    let target => "{}";
    mut content => "";
    for i in 0..1000 {{
        content = content + "ViceEngine!";
    }}
    fs.write_text(target, content);
    let read_back => fs.read_text(target);
    output <- string.contains(read_back, "ViceEngine!ViceEngine!");
}}
"#,
        target_str
    );

    assert_eq!(run_vice_code(&code), "true\n");
    let _ = fs::remove_file(target);
}
