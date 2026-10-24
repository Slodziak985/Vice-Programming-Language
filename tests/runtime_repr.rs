#[test]
fn division_modes() {
    let src_low = r#"
@lowlevel
@entry
fn main() {
    let a: int => 5;
    let b: int => 2;
    output <- a / b;
}
"#;
    let (_res_low, out_low) = vice::run_source(src_low, vec![]).expect("run_low failed");
    assert_eq!(out_low, vec!["2"]);

    let src_high = r#"
@highlevel
@entry
fn main() {
    let a: int => 5;
    let b: int => 2;
    output <- a / b;
}
"#;
    let (_res_high, out_high) = vice::run_source(src_high, vec![]).expect("run_high failed");
    assert_eq!(out_high, vec!["2.5"]);
}

#[test]
fn lowlevel_truncates_float_to_int_in_typed_contexts() {
    let src = r#"
@lowlevel
@entry
fn main() {
    let a: int => 5.9;
    output <- a;
}
"#;
    let (_res, out) = vice::run_source(src, vec![]).expect("run_lowlevel_truncate failed");
    assert_eq!(out, vec!["5"]);
}

#[test]
fn stdlib_numeric_conversions_and_return_types_work() {
    let src = r#"
@entry
fn main() {
    output <- len([1, 2, 3]);
    output <- time.now_ms() > 0;
    output <- math.floor(3.9), math.ceil(3.1), math.round(3.5);
    output <- math.min(2, 5), math.max(2.5, 5);
    output <- to_num("42"), to_int(7.9), to_float(3);
}
"#;
    let (_res, out) = vice::run_source(src, vec![]).expect("run_stdlib_numeric failed");
    assert_eq!(out[0], "3");
    assert_eq!(out[1], "true");
    assert_eq!(out[2], "3 4 4");
    assert_eq!(out[3], "2 5");
    assert_eq!(out[4], "42 7 3");
}
