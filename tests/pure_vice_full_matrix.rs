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

// =========================================================================
// 1. ATTRIBUTE TESTS: @fallible
// =========================================================================

#[test]
fn test_attribute_fallible_success() {
    let src = r#"
        data Result {
            is_ok: bool,
            value: int,
            error: string
        }

        @fallible
        fn compute(x: int) -> Result {
            if x > 0 {
                return Result { is_ok: true, value: x * 2, error: "" };
            }
            return Result { is_ok: false, value: 0, error: "negative" };
        }

        @entry
        fn main() -> int {
            let res = compute(10);
            return res.value;
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(20));
}

#[test]
fn test_attribute_fallible_requires_result_return() {
    let src = r#"
        @fallible
        fn compute(x: int) -> int {
            return x * 2;
        }

        @entry
        fn main() -> int {
            return compute(5);
        }
    "#;
    expect_semantic_error(src, "must return a Result type");
}

#[test]
fn test_attribute_fallible_and_infallible_conflict() {
    let src = r#"
        data Result {
            is_ok: bool,
            value: int,
            error: string
        }

        @fallible
        @infallible
        fn compute(x: int) -> Result {
            return Result { is_ok: true, value: x, error: "" };
        }
    "#;
    expect_semantic_error(src, "cannot be marked with both @fallible and @infallible");
}

// =========================================================================
// 2. ATTRIBUTE TESTS: @infallible
// =========================================================================

#[test]
fn test_attribute_infallible_success() {
    let src = r#"
        @infallible
        fn safe_math(a: int, b: int) -> int {
            return a + b;
        }

        @entry
        fn main() -> int {
            return safe_math(15, 25);
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(40));
}

#[test]
fn test_attribute_infallible_cannot_return_result() {
    let src = r#"
        data Result {
            is_ok: bool,
            value: int,
            error: string
        }

        @infallible
        fn safe_task() -> Result {
            return Result { is_ok: true, value: 1, error: "" };
        }
    "#;
    expect_semantic_error(src, "cannot return a Result or Option type");
}

#[test]
fn test_attribute_infallible_cannot_call_fallible() {
    let src = r#"
        data Result {
            is_ok: bool,
            value: int,
            error: string
        }

        @fallible
        fn might_fail() -> Result {
            return Result { is_ok: true, value: 42, error: "" };
        }

        @infallible
        fn caller() -> int {
            let r = might_fail();
            return 1;
        }
    "#;
    expect_semantic_error(src, "Cannot call @fallible function 'might_fail' directly inside @infallible function");
}

// =========================================================================
// 3. ATTRIBUTE TESTS: @deterministic
// =========================================================================

#[test]
fn test_attribute_deterministic_success() {
    let src = r#"
        @deterministic
        fn pure_calc(x: int) -> int {
            return math.abs(x) + math.bitwise_and(x, 7);
        }

        @entry
        fn main() -> int {
            return pure_calc(13);
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(13 + (13 & 7)));
}

#[test]
fn test_attribute_deterministic_blocks_random() {
    let src = r#"
        @deterministic
        fn nondet() -> float {
            return math.random();
        }
    "#;
    expect_semantic_error(src, "Cannot call non-deterministic module method 'math.random' inside @deterministic function");
}

#[test]
fn test_attribute_deterministic_blocks_time() {
    let src = r#"
        @deterministic
        fn get_time() -> int {
            return time.now_ms();
        }
    "#;
    expect_semantic_error(src, "Cannot call non-deterministic module method 'time.now_ms' inside @deterministic function");
}

// =========================================================================
// 4. ATTRIBUTE TESTS: @idempotent
// =========================================================================

#[test]
fn test_attribute_idempotent_success() {
    let src = r#"
        @idempotent
        fn abs_val(x: int) -> int {
            return math.abs(x);
        }

        @entry
        fn main() -> int {
            return abs_val(abs_val(-42));
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(42));
}

#[test]
fn test_attribute_idempotent_requires_param() {
    let src = r#"
        @idempotent
        fn constant_val() -> int {
            return 100;
        }
    "#;
    expect_semantic_error(src, "must take at least one parameter");
}

#[test]
fn test_attribute_idempotent_cannot_be_entry() {
    let src = r#"
        @idempotent
        @entry
        fn main(x: int) -> int {
            return x;
        }
    "#;
    expect_semantic_error(src, "cannot be marked with @entry");
}

// =========================================================================
// 5. ATTRIBUTE TESTS: @max_depth
// =========================================================================

#[test]
fn test_attribute_max_depth_success_within_limit() {
    let src = r#"
        @max_depth(5)
        fn countdown(n: int) -> int {
            if n <= 1 {
                return 1;
            }
            return n + countdown(n - 1);
        }

        @entry
        fn main() -> int {
            return countdown(3);
        }
    "#;
    let res = run_source(src).expect("Execution should succeed within recursion depth limit");
    assert_eq!(res, Value::Int(6));
}

#[test]
fn test_attribute_max_depth_exceeded_error() {
    let src = r#"
        @max_depth(3)
        fn runaway(n: int) -> int {
            return runaway(n + 1);
        }

        @entry
        fn main() -> int {
            return runaway(1);
        }
    "#;
    let outcome = run_source(src);
    assert!(outcome.is_err());
    let err_msg = outcome.unwrap_err();
    assert!(err_msg.contains("Recursion depth limit"), "Error should indicate depth limit, got: {}", err_msg);
}

// =========================================================================
// 6. ATTRIBUTE TESTS: @pure_data
// =========================================================================

#[test]
fn test_attribute_pure_data_success() {
    let src = r#"
        @pure_data
        data Coordinate {
            x: int,
            y: int,
            label: string
        }

        @entry
        fn main() -> int {
            let c = Coordinate { x: 10, y: 20, label: "origin" };
            return c.x + c.y;
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(30));
}

// =========================================================================
// 7. COMPILER OPTIMIZATIONS
// =========================================================================

#[test]
fn test_compiler_algebraic_identities() {
    let src = r#"
        @entry
        fn main() -> int {
            let x = 50;
            let a = x + 0;
            let b = a - 0;
            let c = b * 1;
            let t = (c > 10) && true;
            let f = (c < 10) || false;
            if t && !f {
                return c;
            }
            return 0;
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(50));
}

#[test]
fn test_compiler_peephole_not_branch_inversion() {
    let src = r#"
        @entry
        fn main() -> int {
            let cond = false;
            let mut result = 0;
            if !cond {
                result = 100;
            } else {
                result = 200;
            }
            return result;
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(100));
}

// =========================================================================
// 8. STDLIB EXTENSIONS: string
// =========================================================================

#[test]
fn test_stdlib_string_new_functions() {
    let src = r#"
        @entry
        fn main() -> int {
            let s = "hello world world";
            let c = string.count(s, "world");
            let cap = string.capitalize("vice");
            let idx = string.index_of(s, "world");
            let last_idx = string.last_index_of(s, "world");
            let pad = string.pad_start("42", 5, "0");
            let pad_e = string.pad_end("42", 5, "x");

            if c == 2 && cap == "Vice" && idx == 6 && last_idx == 12 && pad == "00042" && pad_e == "42xxx" {
                return 1;
            }
            return 0;
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(1));
}

// =========================================================================
// 9. STDLIB EXTENSIONS: array
// =========================================================================

#[test]
fn test_stdlib_array_new_functions() {
    let src = r#"
        @entry
        fn main() -> int {
            let arr = [10, 20, 30, 20, 10];
            let rem = array.remove_at(arr, 1);
            let ins = array.insert([1, 3], 1, 2);
            let fil = array.fill(4, 9);
            let u = array.unique(arr);

            if rem == 20 && ins[1] == 2 && fil[0] == 9 && array.len(fil) == 4 && array.len(u) == 3 {
                return 1;
            }
            return 0;
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(1));
}

// =========================================================================
// 10. STDLIB EXTENSIONS: math
// =========================================================================

#[test]
fn test_stdlib_math_new_functions() {
    let src = r#"
        @entry
        fn main() -> int {
            let l = math.lerp(0.0, 100.0, 0.5);
            let e = math.exp(0.0);
            let even = math.is_even(42);
            let odd = math.is_odd(43);
            let b_and = math.bitwise_and(6, 3);
            let b_or = math.bitwise_or(4, 2);
            let b_xor = math.bitwise_xor(7, 3);
            let b_not = math.bitwise_not(0);
            let b_shl = math.bitwise_shl(1, 4);
            let b_shr = math.bitwise_shr(16, 2);

            if l == 50.0 && e == 1.0 && even && odd && b_and == 2 && b_or == 6 && b_xor == 4 && b_not == -1 && b_shl == 16 && b_shr == 4 {
                return 1;
            }
            return 0;
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(1));
}

// =========================================================================
// 11. STDLIB EXTENSIONS: map module
// =========================================================================

#[test]
fn test_stdlib_map_module() {
    let src = r#"
        @entry
        fn main() -> int {
            let mut m = {"a": 1, "b": 2};
            let l = map.len(m);
            let empty = map.is_empty(m);
            let has_a = map.contains_key(m, "a");
            let v_a = map.get(m, "a", 0);
            let v_missing = map.get(m, "c", 999);
            let keys = map.keys(m);
            let vals = map.values(m);

            let m2 = {"c": 3};
            let merged = map.merge(m, m2);
            let merged_has_c = map.contains_key(merged, "c");

            let removed = m.remove("a");
            let now_has_a = map.contains_key(m, "a");

            if l == 2 && !empty && has_a && v_a == 1 && v_missing == 999 && array.len(keys) == 2 && array.len(vals) == 2 && merged_has_c && removed && !now_has_a {
                return 1;
            }
            return 0;
        }
    "#;
    let res = run_source(src).expect("Execution should succeed");
    assert_eq!(res, Value::Int(1));
}
