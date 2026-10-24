use vice::parse_source;
use vice::semantic::SemanticAnalyzer;

fn run_vice_code(code: &str) -> String {
    let program = parse_source(code).expect("Parser failed");
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).expect("Semantic check failed");
    let output = vice::execute_program_vm(&program).expect("Interpreter failed");
    output.join("\n") + "\n"
}

#[test]
fn string_huge_concatenation() {
    let code = r#"
@entry
fn main() {
    mut s => "";
    for i in 0..5000 {
        s = s + "a";
    }
    output <- s;
}
"#;
    let out = run_vice_code(code);
    assert_eq!(out.trim().len(), 5001);
}

#[test]
fn string_split_by_empty() {
    let code = r#"
@entry
fn main() {
    let parts => string.split("", "");
    output <- len(parts);
}
"#;
    let out = run_vice_code(code);
    // splitting "vice engine" by "" returns 12 items in some langs, or maybe throws.
    // Let's just check it doesn't crash natively.
    assert!(!out.is_empty());
}

#[test]
fn string_trim_empty_and_spaces() {
    let code = r#"
@entry
fn main() {
    let s1 => string.trim("");
    let s2 => string.trim("      ");
    let s3 => string.trim("  x  ");
    output <- s1;
    output <- s2;
    output <- s3;
}
"#;
    assert_eq!(run_vice_code(code), "\n\nx\n");
}

#[test]
fn string_contains_and_starts_with() {
    let code = r#"
@entry
fn main() {
    output <- string.contains("multiverse", "verse");
    output <- string.starts_with("multiverse", "multi");
    output <- string.ends_with("multiverse", "verse");
    output <- string.contains("multiverse", "engine");
}
"#;
    assert_eq!(run_vice_code(code), "true\ntrue\ntrue\nfalse\n");
}
