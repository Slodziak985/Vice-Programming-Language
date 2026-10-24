#[test]
fn read_only_function_prevents_parameter_mutation() {
    let src = r##"
data Point {
    x: int => 0;
    y: int => 0;
}

@read
fn mutate_param(p: Point) {
    p.x = 10;
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject parameter member write");
    assert!(err
        .message
        .contains("Cannot mutate field of parameter or global variable 'p'"));
}

#[test]
fn read_only_function_allows_local_mutation() {
    let src = r##"
data Point {
    x: int => 0;
    y: int => 0;
}

@read
fn local_mutation() -> int {
    mut p => Point(x => 1, y => 2);
    p.x = 10;
    return p.x;
}

@entry
fn main() {
    let val => local_mutation();
    output <- val;
}
"##;
    let (_res, out) =
        vice::run_source(src, vec![]).expect("should allow local mutation in read function");
    assert_eq!(out, vec!["10"]);
}

#[test]
fn read_only_function_prevents_non_read_only_calls() {
    let src = r##"
fn impure() {}

@read
fn pure_caller() {
    impure();
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject calling impure function");
    assert!(err
        .message
        .contains("Cannot call non-read-only function 'impure'"));
}

#[test]
fn read_only_function_allows_read_only_calls() {
    let src = r##"
@read
fn pure_helper() -> int {
    return 42;
}

@read
fn pure_caller() -> int {
    return pure_helper();
}

@entry
fn main() {
    let val => pure_caller();
    output <- val;
}
"##;
    let (_res, out) = vice::run_source(src, vec![]).expect("should allow calling read-only helper");
    assert_eq!(out, vec!["42"]);
}

#[test]
fn read_only_function_prevents_side_effects() {
    let src = r##"
@read
fn do_ui() {
    ui.draw_rect(pos => [0, 0], size => [100, 100], color => "#ffffff");
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject UI call in read function");
    assert!(err
        .message
        .contains("Cannot call side-effecting UI method 'ui.draw_rect'"));
}

#[test]
fn read_only_function_prevents_array_mutation() {
    let src = r##"
@read
fn mutate_array(arr: int[]) {
    arr.push(1);
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject push on parameter array");
    assert!(err
        .message
        .contains("Cannot mutate array/map of parameter or global variable 'arr'"));
}

#[test]
fn read_only_data_prevents_mutation_everywhere() {
    let src = r##"
@read
data ConstPoint {
    x: int => 0;
    y: int => 0;
}

@entry
fn main() {
    mut p => ConstPoint(x => 1, y => 2);
    p.x = 10;
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject mutation of read-only data");
    assert!(err
        .message
        .contains("Cannot mutate field of read-only data type: ConstPoint"));
}
