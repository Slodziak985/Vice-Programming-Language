use vice::error::ViceError;
use vice::semantic::SemanticAnalyzer;

fn analyze(source: &str) -> Result<(), ViceError> {
    let program = vice::parse_source(source)?;
    SemanticAnalyzer::new().analyze(&program)
}

fn semantic_error(source: &str) -> String {
    analyze(source).unwrap_err().message
}

#[test]
fn non_void_function_must_return() {
    let error = semantic_error(
        r#"
        fn answer() -> int {
            let value => 42;
        }
        "#,
    );
    assert!(error.contains("Function 'answer' can complete without returning int"));
}

#[test]
fn one_sided_if_does_not_guarantee_a_return() {
    let error = semantic_error(
        r#"
        fn answer(enabled: bool) -> int {
            if (enabled) {
                return 42;
            }
        }
        "#,
    );
    assert!(error.contains("can complete without returning int"));
}

#[test]
fn if_with_two_returning_branches_is_accepted() {
    analyze(
        r#"
        fn answer(enabled: bool) -> int {
            if (enabled) {
                return 42;
            } else {
                return 0;
            }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn exhaustive_boolean_match_guarantees_a_return() {
    analyze(
        r#"
        fn answer(enabled: bool) -> int {
            match enabled {
                true => { return 1; }
                false => { return 0; }
            }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn exhaustive_enum_match_guarantees_a_return() {
    analyze(
        r#"
        enum State { Ready, Done }
        fn code(state: State) -> int {
            match state {
                State.Ready => { return 1; }
                State.Done => { return 2; }
            }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn infinite_loop_without_break_is_a_diverging_path() {
    analyze(
        r#"
        fn wait_forever() -> int {
            while (true) {
                continue;
            }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn match_pattern_must_match_condition_type() {
    let error = semantic_error(
        r#"
        fn check(value: int) {
            match value {
                "wrong" => { return; }
            }
        }
        "#,
    );
    assert!(error.contains("Match pattern type string is incompatible with int"));
}

#[test]
fn wildcard_arm_must_be_last() {
    let error = semantic_error(
        r#"
        fn check(value: int) {
            match value {
                _ => { return; }
                1 => { return; }
            }
        }
        "#,
    );
    assert!(error.contains("Wildcard match arm must be last"));
}

#[test]
fn non_exhaustive_match_expression_raises_runtime_error() {
    let error = vice::run_source(
        r#"
        @entry
        fn main() {
            let value => match 9 {
                1 => "one"
            };
            output <- value;
        }
        "#,
        vec![],
    )
    .unwrap_err();
    assert_eq!(error.message, "Non-exhaustive match expression");
}
