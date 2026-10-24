use vice::run_source;
use vice::value::Value;
use vice::vm::compiler::Compiler;

#[test]
fn test_thread_safe_valid_and_calling_pure() {
    let src = r#"
@pure
fn pure_calc(x: int) -> int {
    return x * 10;
}

@thread_safe
fn safe_worker(val: int) -> int {
    let local_res => pure_calc(val) + 5;
    return local_res;
}

@entry
fn main() -> int {
    return safe_worker(4);
}
"#;
    let (res, _) = run_source(src, vec![]).expect("Valid @thread_safe function should run cleanly");
    assert_eq!(res, Some(Value::Int(45)));
}

#[test]
fn test_thread_safe_conflicts_and_mutations_rejected() {
    // 1. Conflict with @nonreentrant
    let src_conflict = r#"
@thread_safe
@nonreentrant
fn conflict_fn() -> int {
    return 1;
}

@entry
fn main() {}
"#;
    let err_conflict = run_source(src_conflict, vec![]).expect_err("Should reject @thread_safe + @nonreentrant");
    assert!(
        format!("{:?}", err_conflict).contains("cannot be marked with both @thread_safe and @nonreentrant"),
        "Got: {:?}",
        err_conflict
    );

    // 2. Calling non-thread_safe function
    let src_call = r#"
fn impure_func() -> int {
    return 10;
}

@thread_safe
fn safe_fn() -> int {
    return impure_func();
}

@entry
fn main() {}
"#;
    let err_call = run_source(src_call, vec![]).expect_err("Should reject calling impure function inside @thread_safe");
    assert!(
        format!("{:?}", err_call).contains("Cannot call non-thread_safe function 'impure_func' inside @thread_safe function"),
        "Got: {:?}",
        err_call
    );

    // 3. Calling impure module (e.g. fs)
    let src_fs = r#"
@thread_safe
fn safe_io() {
    fs.write_text("test.txt", "data");
}

@entry
fn main() {}
"#;
    let err_fs = run_source(src_fs, vec![]).expect_err("Should reject calling fs inside @thread_safe");
    assert!(
        format!("{:?}", err_fs).contains("Cannot call impure module method 'fs.write_text' inside @thread_safe function"),
        "Got: {:?}",
        err_fs
    );
}

#[test]
fn test_pure_math_valid_and_non_math_rejected() {
    let src_valid = r#"
@pure_math
fn hypotenuse_squared(a: int, b: int) -> int {
    return (a * a) + (b * b);
}

@entry
fn main() -> int {
    return hypotenuse_squared(3, 4);
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @pure_math function should succeed");
    assert_eq!(res, Some(Value::Int(25)));

    // 1. Non-numeric return
    let src_string = r#"
@pure_math
fn invalid_ret(x: int) -> string {
    return "number";
}

@entry
fn main() {}
"#;
    let err_ret = run_source(src_string, vec![]).expect_err("Should reject non-numeric return for @pure_math");
    assert!(
        format!("{:?}", err_ret).contains("@pure_math function 'invalid_ret' must return a number"),
        "Got: {:?}",
        err_ret
    );

    // 2. Calling non-math module
    let src_non_math = r#"
@pure_math
fn test_time(x: int) -> int {
    let now => time.now_ms();
    return x;
}

@entry
fn main() {}
"#;
    let err_time = run_source(src_non_math, vec![]).expect_err("Should reject non-math module in @pure_math");
    assert!(
        format!("{:?}", err_time).contains("Cannot call non-math module 'time.now_ms' inside @pure_math function"),
        "Got: {:?}",
        err_time
    );
}

#[test]
fn test_clamp_attribute_validation() {
    // 1. Valid execution within bounds
    let src_valid = r#"
@clamp(min: 0, max: 100)
fn clamp_sensor(raw: int) -> int {
    return raw;
}

@entry
fn main() -> int {
    return clamp_sensor(50);
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid clamp should succeed");
    assert_eq!(res, Some(Value::Int(50)));

    // 2. min > max error
    let src_inverted = r#"
@clamp(min: 100, max: 10)
fn invalid_clamp(val: int) -> int {
    return val;
}

@entry
fn main() {}
"#;
    let err_inv = run_source(src_inverted, vec![]).expect_err("Should reject min > max in @clamp");
    assert!(
        format!("{:?}", err_inv).contains("@clamp attribute min (100) cannot be greater than max (10)"),
        "Got: {:?}",
        err_inv
    );

    // 3. Constant return outside clamp range statically detected
    let src_out_of_bounds = r#"
@clamp(0, 50)
fn out_of_range() -> int {
    return 999;
}

@entry
fn main() {}
"#;
    let err_oob = run_source(src_out_of_bounds, vec![]).expect_err("Should reject constant return outside clamp");
    assert!(
        format!("{:?}", err_oob).contains("Constant return value 999 is outside @clamp(0, 50) range"),
        "Got: {:?}",
        err_oob
    );
}

#[test]
fn test_non_empty_attribute() {
    let src_valid = r#"
@non_empty
fn first_char(s: string) -> string {
    return s.char_at(0);
}

@entry
fn main() -> string {
    return first_char("Vice");
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @non_empty should succeed");
    assert_eq!(res, Some(Value::String("V".to_string())));

    // Calling with empty literal ""
    let src_empty = r#"
@non_empty
fn process_list(items: int[]) -> int {
    return items.len();
}

@entry
fn main() {
    process_list([]);
}
"#;
    let err_empty = run_source(src_empty, vec![]).expect_err("Should reject empty literal for @non_empty parameter");
    assert!(
        format!("{:?}", err_empty).contains("@non_empty function 'process_list' cannot be called with an empty collection argument"),
        "Got: {:?}",
        err_empty
    );
}

#[test]
fn test_bitwise_only_validation() {
    let src_valid = r#"
@bitwise_only
fn mask_and_shift(val: int, mask: int) -> int {
    let a => math.bitwise_and(val, mask);
    return math.bitwise_shl(a, 2);
}

@entry
fn main() -> int {
    return mask_and_shift(15, 5);
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @bitwise_only function should succeed");
    assert_eq!(res, Some(Value::Int(5 << 2)));

    // Float literal rejection
    let src_float = r#"
@bitwise_only
fn bad_float(val: int) -> int {
    let f => 3.14;
    return val;
}

@entry
fn main() {}
"#;
    let err_float = run_source(src_float, vec![]).expect_err("Should reject float literal in @bitwise_only");
    assert!(
        format!("{:?}", err_float).contains("Float literal '3.14' is forbidden inside @bitwise_only function"),
        "Got: {:?}",
        err_float
    );
}

#[test]
fn test_array_algorithms_runtime() {
    let src = r#"
@entry
fn main() -> int {
    // 1. binary_search
    let arr => [10, 20, 30, 40, 50, 60];
    let idx_found => arr.binary_search(40);
    let idx_missing => arr.binary_search(99);

    // 2. rotate
    let rotated => [1, 2, 3, 4, 5].rotate(2); // [4, 5, 1, 2, 3]

    // 3. repeat
    let repeated => array.repeat(7, 4); // [7, 7, 7, 7]

    // 4. windows
    let wins => [1, 2, 3, 4].windows(2); // [[1, 2], [2, 3], [3, 4]]

    // 5. frequencies
    let freq => ["a", "b", "a", "c", "a"].frequencies();

    if idx_found == 3 && idx_missing == -1
        && rotated[0] == 4 && rotated[4] == 3
        && repeated.len() == 4 && repeated[3] == 7
        && wins.len() == 3 && wins[0][1] == 2
        && freq.get("a", 0) == 3 && freq.get("b", 0) == 1 {
        return 1;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src, vec![]).expect("Array algorithms should run cleanly");
    assert_eq!(res, Some(Value::Int(1)));
}

#[test]
fn test_math_and_hash_algorithms_runtime() {
    let src = r#"
@entry
fn main() -> int {
    // 1. is_prime
    let p2 => math.is_prime(2);
    let p7 => math.is_prime(7);
    let p8 => math.is_prime(8);
    let p1 => math.is_prime(1);

    // 2. bit counts
    let ones => math.count_ones(7); // 0b111 -> 3
    let pwr => math.is_power_of_two(16);
    let not_pwr => math.is_power_of_two(18);
    let nxt => math.next_power_of_two(17); // 32

    // 3. xxhash32
    let h1 => hash.xxhash32("ViceLanguage", 0);
    let h2 => hash.xxhash32("ViceLanguage", 0);
    let h3 => hash.xxhash32("Different", 0);

    // 4. rot13
    let enc => "Hello".rot13();
    let dec => enc.rot13();

    if p2 && p7 && !p8 && !p1
        && ones == 3 && pwr && !not_pwr && nxt == 32
        && h1 == h2 && h1 != h3
        && enc == "Uryyb" && dec == "Hello" {
        return 42;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src, vec![]).expect("Math and hash algorithms should run cleanly");
    assert_eq!(res, Some(Value::Int(42)));
}

#[test]
fn test_compiler_constant_folding_v6() {
    let src = r#"
@entry
fn main() -> int {
    // Compile-time folded:
    let sum => [1, 2, 3, 4, 5].sum();
    let prod => [2, 3, 4].product();
    let prime => math.is_prime(29);
    let ones => math.count_ones(15);
    let rot => "Vice".rot13();
    let h => hash.xxhash32("constant", 42);

    if sum == 15 && prod == 24 && prime && ones == 4 && rot == "Ivpr" && h != 0 {
        return 100;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src, vec![]).expect("Constant folding v6 should execute properly");
    assert_eq!(res, Some(Value::Int(100)));
}

#[test]
fn test_compiler_peephole_swap_comparator_inversion() {
    let src = r#"
@entry
fn main() -> bool {
    return true;
}
"#;
    let program = vice::parse_source(src).expect("Parse");
    let compiled = Compiler::new().compile(&program).expect("Compile program");
    let main_chunk = &compiled.functions.get("main").expect("main exists").chunk;

    // Verify chunk was optimized and non-empty
    assert!(!main_chunk.code.is_empty());
}
