use vice::run_source;
use vice::value::Value;

#[test]
fn test_no_alias_valid_and_aliasing_rejected() {
    // 1. Valid execution with different reference targets
    let src_valid = r#"
@no_alias
fn swap_vals(a: &mut int, b: &mut int) {
    let tmp => *a;
    *a = *b;
    *b = tmp;
}

@entry
fn main() -> int {
    let mut x => 10;
    let mut y => 20;
    swap_vals(&x, &y);
    if x == 20 && y == 10 {
        return 1;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @no_alias function should run cleanly");
    assert_eq!(res, Some(Value::Int(1)));

    // 2. Reject function definition with fewer than 2 reference params
    let src_one_ref = r#"
@no_alias
fn single_ref(a: &mut int, b: int) {
    *a = b;
}

@entry
fn main() {}
"#;
    let err_one = run_source(src_one_ref, vec![]).expect_err("Should reject @no_alias with fewer than 2 ref params");
    assert!(
        format!("{:?}", err_one).contains("@no_alias function 'single_ref' requires at least two reference parameters"),
        "Got: {:?}",
        err_one
    );

    // 3. Reject call site passing conflicting reference to the same root variable
    let src_conflict = r#"
@no_alias
fn test_alias(a: &mut int, b: &mut int) {
    *a = *a + *b;
}

@entry
fn main() {
    let mut x => 5;
    test_alias(&x, &x);
}
"#;
    let err_conflict = run_source(src_conflict, vec![]).expect_err("Should reject aliased reference call");
    assert!(
        format!("{:?}", err_conflict).contains("Call to @no_alias function 'test_alias' passes conflicting reference to variable 'x' in multiple arguments"),
        "Got: {:?}",
        err_conflict
    );
}

#[test]
fn test_pure_string_validation() {
    // 1. Valid pure string function
    let src_valid = r#"
@pure_string
fn format_item(name: string, count: int) -> string {
    let slug => name.slugify();
    let hex => slug.hex_encode();
    return hex;
}

@entry
fn main() -> string {
    return format_item("Hello World", 42);
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @pure_string should succeed");
    assert_eq!(res, Some(Value::String("68656c6c6f2d776f726c64".to_string())));

    // 2. Reject @entry on @pure_string
    let src_entry = r#"
@pure_string
@entry
fn main() -> string {
    return "test";
}
"#;
    let err_entry = run_source(src_entry, vec![]).expect_err("Should reject @entry on @pure_string");
    assert!(
        format!("{:?}", err_entry).contains("@pure_string function 'main' cannot be marked with @entry"),
        "Got: {:?}",
        err_entry
    );

    // 3. Reject calling impure module (e.g. fs)
    let src_fs = r#"
@pure_string
fn read_file(name: string) -> string {
    return fs.read_text(name);
}

@entry
fn main() {}
"#;
    let err_fs = run_source(src_fs, vec![]).expect_err("Should reject non-string module inside @pure_string");
    assert!(
        format!("{:?}", err_fs).contains("Cannot call non-string module 'fs.read_text' inside @pure_string function"),
        "Got: {:?}",
        err_fs
    );
}

#[test]
fn test_non_negative_validation() {
    // 1. Valid execution with positive constant
    let src_valid = r#"
@non_negative
fn calculate_area(width: int, height: int) -> int {
    return width * height;
}

@entry
fn main() -> int {
    return calculate_area(10, 5);
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @non_negative call should run cleanly");
    assert_eq!(res, Some(Value::Int(50)));

    // 2. Statically reject negative literal call argument
    let src_neg = r#"
@non_negative
fn set_timer(delay_ms: int) -> int {
    return delay_ms;
}

@entry
fn main() {
    set_timer(-50);
}
"#;
    let err_neg = run_source(src_neg, vec![]).expect_err("Should reject negative constant argument");
    assert!(
        format!("{:?}", err_neg).contains("Constant argument to @non_negative function 'set_timer' at parameter 1 must be non-negative (>= 0)"),
        "Got: {:?}",
        err_neg
    );
}

#[test]
fn test_string_algorithms_runtime() {
    let src = r#"
@entry
fn main() -> int {
    // 1. levenshtein
    let d1 => "kitten".levenshtein("sitting");
    let d2 => string.levenshtein("rosettacode", "raisethysword");

    // 2. hamming
    let h1 => "karolin".hamming("kathrin");
    let h2 => string.hamming("1011101", "1001001");
    let h_diff => "abc".hamming("abcd");

    // 3. slugify
    let slug => "Pure Vice 2026! Rocks_Hard".slugify();

    // 4. truncate
    let trunc => "Antigravity AI Engine".truncate(15, "...");

    // 5. hex_encode & hex_decode
    let enc => "Vice".hex_encode();
    let dec => enc.hex_decode();

    if d1 == 3 && d2 == 8
        && h1 == 3 && h2 == 2 && h_diff == -1
        && slug == "pure-vice-2026-rocks-hard"
        && trunc == "Antigravity ..."
        && enc == "56696365" && dec == "Vice" {
        return 100;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src, vec![]).expect("String algorithms should run cleanly");
    assert_eq!(res, Some(Value::Int(100)));
}

#[test]
fn test_math_algorithms_runtime() {
    let src = r#"
@entry
fn main() -> int {
    // 1. clamp01
    let c1 => math.clamp01(1.5);
    let c2 => math.clamp01(-0.25);
    let c3 => math.clamp01(0.75);

    // 2. lerp_inv
    let inv => math.lerp_inv(10.0, 20.0, 15.0); // 0.5

    // 3. remap
    let remapped => math.remap(5.0, 0.0, 10.0, 0.0, 100.0); // 50.0

    // 4. midpoint
    let mid_int => math.midpoint(10, 20); // 15
    let mid_flt => math.midpoint(10.0, 20.0); // 15.0

    // 5. approx_eq
    let eq1 => math.approx_eq(1.0000001, 1.0, 0.0001);
    let eq2 => math.approx_eq(1.5, 2.0, 0.1);

    if c1 == 1.0 && c2 == 0.0 && c3 == 0.75
        && inv == 0.5 && remapped == 50.0
        && mid_int == 15 && mid_flt == 15.0
        && eq1 && !eq2 {
        return 200;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src, vec![]).expect("Math algorithms should run cleanly");
    assert_eq!(res, Some(Value::Int(200)));
}

#[test]
fn test_compiler_constant_folding_v7() {
    let src = r#"
@entry
fn main() -> int {
    // Compile-time folded receiver methods and module calls
    let d => "kitten".levenshtein("sitting");
    let slug => "Hello World!".slugify();
    let hex => "Vice".hex_encode();
    let c => math.clamp01(2.5);
    let mid => math.midpoint(100, 200);

    if d == 3 && slug == "hello-world" && hex == "56696365" && c == 1.0 && mid == 150 {
        return 300;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src, vec![]).expect("Constant folding v7 should execute properly");
    assert_eq!(res, Some(Value::Int(300)));
}
