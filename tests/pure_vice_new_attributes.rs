use vice::value::Value;
use vice::vm::machine::Vm;

fn run_source(source: &str) -> Result<Value, String> {
    let program = vice::parse_source(source).map_err(|e| format!("{:?}", e))?;
    let mut semantic = vice::semantic::SemanticAnalyzer::new();
    semantic.analyze(&program).map_err(|e| format!("{:?}", e))?;

    let compiled = vice::vm::compiler::Compiler::new()
        .compile(&program)
        .map_err(|e| format!("{:?}", e))?;

    let mut vm = Vm::new();
    let res = vm.interpret(compiled).map_err(|e| format!("{:?}", e))?;
    Ok(res.value)
}

fn expect_semantic_error(source: &str, expected_substring: &str) {
    let program = match vice::parse_source(source) {
        Ok(p) => p,
        Err(e) => panic!("Parse unexpectedly failed: {:?}", e),
    };

    let mut semantic = vice::semantic::SemanticAnalyzer::new();
    match semantic.analyze(&program) {
        Ok(_) => panic!(
            "Expected semantic error containing '{}', but analysis succeeded!",
            expected_substring
        ),
        Err(err) => {
            let msg = format!("{:?}", err);
            assert!(
                msg.contains(expected_substring),
                "Expected error message to contain '{}', but got: {}",
                expected_substring,
                msg
            );
        }
    }
}

// 1. @checked
#[test]
fn test_attribute_checked_normal_operations() {
    let src = r#"
@entry
@checked
fn main() -> int {
    let a: int = 100;
    let b: int = 200;
    let c: int = a + b;
    let d: int = c * 2;
    return d - 100;
}
"#;
    let res = run_source(src);
    assert!(res.is_ok(), "Expected success, got: {:?}", res);
    assert_eq!(res.unwrap(), Value::Int(500));
}

#[test]
fn test_attribute_checked_addition_overflow() {
    let src = r#"
@entry
@checked
fn main() -> int {
    let max_val: int = 9223372036854775807;
    let overflowed: int = max_val + 1;
    return overflowed;
}
"#;
    let res = run_source(src);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Integer overflow in @checked function"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_checked_multiplication_overflow() {
    let src = r#"
@entry
@checked
fn main() -> int {
    let big: int = 5000000000000000000;
    let overflowed: int = big * 2;
    return overflowed;
}
"#;
    let res = run_source(src);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Integer overflow in @checked function"),
        "Unexpected error: {}",
        err
    );
}

// 2. @saturating
#[test]
fn test_attribute_saturating_addition_clamp() {
    let src = r#"
@entry
@saturating
fn main() -> int {
    let max_val: int = 9223372036854775807;
    let sat: int = max_val + 1000;
    if sat == 9223372036854775807 {
        return 42;
    }
    return 0;
}
"#;
    let val = run_source(src).expect("Saturating addition should succeed");
    assert_eq!(val, Value::Int(42));
}

#[test]
fn test_attribute_saturating_subtraction_clamp() {
    let src = r#"
@entry
@saturating
fn main() -> int {
    let min_val: int = -9223372036854775807;
    let sat: int = min_val - 1000;
    if sat <= min_val {
        return 99;
    }
    return 0;
}
"#;
    let val = run_source(src).expect("Saturating subtraction should succeed");
    assert_eq!(val, Value::Int(99));
}

// 3. Conflict between @checked and @saturating
#[test]
fn test_attribute_checked_and_saturating_conflict() {
    let src = r#"
@entry
@checked
@saturating
fn main() -> int {
    return 1;
}
"#;
    expect_semantic_error(src, "cannot be marked with both @checked and @saturating");
}

// 4. @benchmark
#[test]
fn test_attribute_benchmark_valid() {
    let src = r#"
@benchmark
fn bench_something() -> int {
    return 123;
}

@entry
fn main() -> int {
    return 0;
}
"#;
    let val = run_source(src).expect("Valid @benchmark should succeed");
    assert_eq!(val, Value::Int(0));
}

#[test]
fn test_attribute_benchmark_rejects_parameters() {
    let src = r#"
@benchmark
fn bench_with_args(x: int) -> int {
    return x;
}

@entry
fn main() -> int {
    return 0;
}
"#;
    expect_semantic_error(
        src,
        "@benchmark function 'bench_with_args' must take 0 parameters",
    );
}

#[test]
fn test_attribute_benchmark_cannot_be_entry() {
    let src = r#"
@entry
@benchmark
fn main() -> int {
    return 0;
}
"#;
    expect_semantic_error(src, "cannot be marked with both @benchmark and @entry");
}

// 5. @unreachable
#[test]
fn test_attribute_unreachable_declaration_ok() {
    let src = r#"
@unreachable
fn dead_code() -> void {
    let x: int = 1;
}

@entry
fn main() -> int {
    return 42;
}
"#;
    let val = run_source(src).expect("Uncalled @unreachable function should succeed");
    assert_eq!(val, Value::Int(42));
}

#[test]
fn test_attribute_unreachable_call_rejected_statically() {
    let src = r#"
@unreachable
fn panic_handler() -> void {
}

@entry
fn main() -> int {
    panic_handler();
    return 0;
}
"#;
    expect_semantic_error(src, "Cannot call @unreachable function 'panic_handler'");
}

#[test]
fn test_attribute_unreachable_cannot_be_entry() {
    let src = r#"
@entry
@unreachable
fn main() -> int {
    return 0;
}
"#;
    expect_semantic_error(src, "cannot be marked with both @unreachable and @entry");
}

// 6. @final
#[test]
fn test_attribute_final_instantiation_ok() {
    let src = r#"
@final
data ImmutableRecord {
    id: int,
    name: string
}

@entry
fn main() -> int {
    let rec = ImmutableRecord { id: 7, name: "test" };
    return rec.id;
}
"#;
    let val = run_source(src).expect("Final data instantiation should succeed");
    assert_eq!(val, Value::Int(7));
}

#[test]
fn test_attribute_final_rejects_impl() {
    let src = r#"
@final
data FrozenStruct {
    value: int
}

impl FrozenStruct {
    fn get(self) -> int {
        return self.value;
    }
}

@entry
fn main() -> int {
    return 0;
}
"#;
    expect_semantic_error(
        src,
        "Cannot define impl block for @final data 'FrozenStruct'",
    );
}

// 7. Bitwise and math constant folding
#[test]
fn test_constant_folding_bitwise_operations() {
    let src = r#"
@entry
fn main() -> int {
    let and_val: int = math.bitwise_and(12, 10);
    let or_val: int = math.bitwise_or(12, 10);
    let xor_val: int = math.bitwise_xor(12, 10);
    let shl_val: int = math.bitwise_shl(1, 4);
    let shr_val: int = math.bitwise_shr(16, 2);
    let not_val: int = math.bitwise_not(0);
    return and_val + or_val + xor_val + shl_val + shr_val + not_val;
}
"#;
    let val = run_source(src).expect("Constant folding bitwise should succeed");
    // 8 + 14 + 6 + 16 + 4 + (-1) = 47
    assert_eq!(val, Value::Int(47));
}

#[test]
fn test_constant_folding_math_helpers() {
    let src = r#"
@entry
fn main() -> int {
    let even: bool = math.is_even(42);
    let odd: bool = math.is_odd(43);
    let lerp_val: float = math.lerp(0.0, 100.0, 0.5);
    if even && odd && lerp_val == 50.0 {
        return 1;
    }
    return 0;
}
"#;
    let val = run_source(src).expect("Constant folding math helpers should succeed");
    assert_eq!(val, Value::Int(1));
}

#[test]
fn test_constant_folding_string_module_functions() {
    let src = r#"
@entry
fn main() -> int {
    let c: bool = string.contains("vice-lang", "lang");
    let sw: bool = string.starts_with("compiler", "comp");
    let ew: bool = string.ends_with("engine", "ine");
    let trimmed: string = string.trim("  pure  ");
    let up: string = string.upper("hi");
    let rep: string = string.repeat("v", 3);
    let idx: int = string.index_of("banana", "na");

    if c && sw && ew && trimmed == "pure" && up == "HI" && rep == "vvv" && idx == 2 {
        return 100;
    }
    return 0;
}
"#;
    let val = run_source(src).expect("String module folds should succeed");
    assert_eq!(val, Value::Int(100));
}

#[test]
fn test_constant_folding_array_module_functions() {
    let src = r#"
@entry
fn main() -> int {
    let empty: bool = array.is_empty([]);
    let not_empty: bool = array.is_empty([1, 2]);
    let has: bool = array.contains([10, 20, 30], 20);
    let f: int = array.first([7, 8, 9]);
    let l: int = array.last([7, 8, 9]);
    let joined: string = array.join(["a", "b", "c"], "-");

    if empty && !not_empty && has && f == 7 && l == 9 && joined == "a-b-c" {
        return 200;
    }
    return 0;
}
"#;
    let val = run_source(src).expect("Array module folds should succeed");
    assert_eq!(val, Value::Int(200));
}
