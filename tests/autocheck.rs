#[test]
fn autocheck_infers_literal_types() {
    let src = r#"
@autocheck
@entry
fn main() {
    let a => 5;
    let b => 5.5;
    output <- a;
    output <- b;
}
"#;
    let (_res, out) = vice::run_source(src, vec![]).expect("run_source failed");
    assert!(out.iter().any(|s| s.contains("5")));
    assert!(out.iter().any(|s| s.contains("5.5")));
}

#[test]
fn autocheck_infers_expressions_params_and_returns() {
    let src = r#"
@autocheck
fn add(a: int, b: int) -> int {
    let total => a + b;
    return total;
}

@entry
fn main() {
    let value => add(2, 3);
    output <- value;
}
"#;
    let (_res, out) = vice::run_source(src, vec![]).expect("run_source failed");
    assert_eq!(out, vec!["5"]);
}

#[test]
fn highlevel_requires_explicit_float_to_int_conversion() {
    let src = r#"
@highlevel
@entry
fn main() {
    let value: int => 5.9;
    output <- value;
}
"#;
    let err = vice::run_source(src, vec![]).expect_err("highlevel should reject implicit truncate");
    assert!(err.message.contains("expected int, got float"));
}
