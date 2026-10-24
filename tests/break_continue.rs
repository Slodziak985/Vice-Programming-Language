use vice::error::ViceErrorType;
use vice::parse_source;
use vice::run_source;
use vice::semantic::SemanticAnalyzer;

#[test]
fn runtime_break_continue() {
    let (_, output) = run_source(
        r#"@entry
fn main() {
    let mut sum: int => 0;
    for j in 1..5 {
        if (j == 3) {
            continue;
        }
        sum = sum + j;
        if (j == 4) {
            break;
        }
    }
    output <- sum;
}
"#,
        vec![],
    )
    .unwrap();

    assert_eq!(output, vec!["7"]);
}

#[test]
fn semantic_break_outside_loop() {
    let error = parse_source(
        r#"@entry
fn main() {
    break;
}
"#,
    )
    .and_then(|program| SemanticAnalyzer::new().analyze(&program))
    .unwrap_err();

    assert_eq!(error.error_type, ViceErrorType::Semantic);
    assert!(error.message.contains("break"));
}

#[test]
fn semantic_continue_outside_loop() {
    let error = parse_source(
        r#"@entry
fn main() {
    continue;
}
"#,
    )
    .and_then(|program| SemanticAnalyzer::new().analyze(&program))
    .unwrap_err();

    assert_eq!(error.error_type, ViceErrorType::Semantic);
    assert!(error.message.contains("continue"));
}
