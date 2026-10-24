use vice::error::ViceErrorType;
use vice::vm::machine::Vm;

fn run_source_full(source: &str) -> Result<vice::vm::machine::ExecutionResult, vice::error::ViceError> {
    let program = vice::parse_source(source)?;
    let mut semantic = vice::semantic::SemanticAnalyzer::new();
    semantic.analyze(&program)?;
    let compiled = vice::vm::compiler::Compiler::new().compile(&program)?;
    let mut vm = Vm::new();
    vm.interpret(compiled)
}

fn check_source(source: &str) -> Result<(), vice::error::ViceError> {
    let program = vice::parse_source(source)?;
    let mut semantic = vice::semantic::SemanticAnalyzer::new();
    semantic.analyze(&program)
}

fn compile_source(source: &str) -> Result<(), vice::error::ViceError> {
    check_source(source)
}

fn run_source(source: &str) -> Result<Vec<String>, vice::error::ViceError> {
    run_source_full(source).map(|res| res.output)
}

// ==========================================
// 1. @hot, @cold, @unroll Attributes & Conflicts
// ==========================================

#[test]
fn test_hot_attribute_execution() {
    let code = r#"
@hot
fn fast_calc(x: int) -> int {
    return x * 10;
}

@entry
fn main() {
    output <- to_str(fast_calc(4));
}
"#;
    let res = run_source_full(code).expect("Should compile and run @hot function");
    assert_eq!(res.output, vec!["40"]);
}

#[test]
fn test_hot_and_cold_conflict() {
    let code = r#"
@hot
@cold
fn conflicting() -> int {
    return 1;
}

@entry
fn main() {
    conflicting();
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @hot and @cold"));
}

#[test]
fn test_unroll_and_cold_conflict() {
    let code = r#"
@unroll
@cold
fn conflicting() -> int {
    return 1;
}

@entry
fn main() {
    conflicting();
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @unroll and @cold"));
}

// ==========================================
// 2. @strict Attribute
// ==========================================

#[test]
fn test_strict_valid_function() {
    let code = r#"
@strict
fn safe_multiply(a: int, b: int) -> int {
    let factor: int => a;
    return factor * b;
}

@entry
fn main() {
    output <- to_str(safe_multiply(6, 7));
}
"#;
    let res = run_source_full(code).expect("Valid @strict function should compile and run");
    assert_eq!(res.output, vec!["42"]);
}

#[test]
fn test_strict_rejects_missing_return_type() {
    let code = r#"
@strict
fn unannotated(x: int) {
    output <- to_str(x);
}

@entry
fn main() {
    unannotated(1);
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("@strict function 'unannotated' must specify an explicit return type"));
}

#[test]
fn test_strict_rejects_unannotated_parameter() {
    let code = r#"
@strict
fn bad_param(x) -> int {
    return 1;
}

@entry
fn main() {
    bad_param(1);
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Parameter 'x' in @strict function 'bad_param' must specify an explicit type annotation"));
}

#[test]
fn test_strict_rejects_unannotated_variable() {
    let code = r#"
@strict
fn bad_local(x: int) -> int {
    let local => x + 1;
    return local;
}

@entry
fn main() {
    bad_local(1);
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Variable 'local' in @strict function 'bad_local' must specify an explicit type annotation"));
}

// ==========================================
// 3. @nodiscard Attribute
// ==========================================

#[test]
fn test_nodiscard_allows_used_value() {
    let code = r#"
@nodiscard(reason => "computation result should not be lost")
fn compute() -> int {
    return 100;
}

@entry
fn main() {
    let val => compute();
    output <- to_str(val);
}
"#;
    let res = run_source_full(code).expect("Should succeed when return value is captured");
    assert_eq!(res.output, vec!["100"]);
}

#[test]
fn test_nodiscard_rejects_discarded_call() {
    let code = r#"
@nodiscard(reason => "important result")
fn compute() -> int {
    return 100;
}

@entry
fn main() {
    compute();
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Unused return value from function 'compute' marked with @must_use: important result"));
}

// ==========================================
// 4. @immutable Data Attribute
// ==========================================

#[test]
fn test_immutable_data_rejects_field_mutation() {
    let code = r#"
@immutable
data Coordinate {
    x: int => 0;
    y: int => 0;
}

@entry
fn main() {
    mut pt => Coordinate(x => 10, y => 20);
    pt.x = 99;
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot mutate field of read-only data type: Coordinate"));
}

// ==========================================
// 5. @budget Execution Protection
// ==========================================

#[test]
fn test_budget_within_limit_passes() {
    let code = r#"
@budget(instructions => 500)
fn quick_sum(n: int) -> int {
    return n * (n + 1) / 2;
}

@entry
fn main() {
    output <- to_str(quick_sum(10));
}
"#;
    let res = run_source_full(code).expect("Should succeed within budget");
    assert_eq!(res.output, vec!["55"]);
}

#[test]
fn test_budget_exceeded_raises_runtime_error() {
    let code = r#"
@budget(instructions => 30)
fn infinite_loop() -> int {
    mut counter => 0;
    while (true) {
        counter += 1;
    }
    return counter;
}

@entry
fn main() {
    infinite_loop();
}
"#;
    let err = run_source_full(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Runtime);
    assert!(err.message.contains("Instruction budget of 30 exceeded in function 'infinite_loop'"));
}

// ==========================================
// 6. Map Instance Methods
// ==========================================

#[test]
fn test_map_instance_methods() {
    let code = r#"
@entry
fn main() {
    mut m => {"b": 2, "a": 1, "c": 3};
    output <- to_str(m.len());
    output <- to_str(m.is_empty());
    output <- to_str(m.contains("b"));
    output <- to_str(m.contains("z"));

    let keys => m.keys();
    output <- keys.join(",");

    let vals => m.values();
    output <- vals.join(",");

    output <- to_str(m.get("a", 999));
    output <- to_str(m.get("missing", 999));

    let removed => m.remove("b");
    output <- to_str(removed);
    output <- to_str(m.len());
    output <- to_str(m.contains("b"));

    m.clear();
    output <- to_str(m.len());
    output <- to_str(m.is_empty());
}
"#;
    let res = run_source_full(code).expect("Map methods should execute successfully");
    assert_eq!(res.output, vec![
        "3",
        "false",
        "true",
        "false",
        "a,b,c",
        "1,2,3",
        "1",
        "999",
        "true",
        "2",
        "false",
        "0",
        "true"
    ]);
}

// ==========================================
// 7. Array Instance Methods (slice, index_of, insert, remove_at, clone)
// ==========================================

#[test]
fn test_array_advanced_methods() {
    let code = r#"
@entry
fn main() {
    mut arr: int[] => [10, 20, 30, 40, 50];
    let sliced => arr.slice(1, 4);
    output <- sliced.join("-");

    output <- to_str(arr.index_of(30));
    output <- to_str(arr.find(99));

    arr.insert(2, 25);
    output <- arr.join("-");

    let removed => arr.remove_at(2);
    output <- to_str(removed);
    output <- arr.join("-");

    let copied => arr.clone();
    output <- copied.join("-");
}
"#;
    let res = run_source_full(code).expect("Array advanced methods should execute");
    assert_eq!(res.output, vec![
        "20-30-40",
        "2",
        "-1",
        "10-20-25-30-40-50",
        "25",
        "10-20-30-40-50",
        "10-20-30-40-50"
    ]);
}

#[test]
fn test_array_bounds_check_safety() {
    let code = r#"
@entry
fn main() {
    mut arr => [1, 2];
    arr.remove_at(10);
}
"#;
    let err = run_source_full(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Runtime);
    assert!(err.message.contains("remove_at index out of bounds"));
}

// ==========================================
// 8. String Instance Methods (slice, index_of, repeat, chars, pad_start, pad_end)
// ==========================================

#[test]
fn test_string_advanced_methods() {
    let code = r#"
@entry
fn main() {
    let s => "Vice Language";
    output <- s.slice(0, 4);
    output <- to_str(s.index_of("Lang"));
    output <- to_str(s.index_of("Rust"));

    let stars => "*".repeat(5);
    output <- stars;

    let letters => "ABC".chars();
    output <- letters.join("|");

    let padded_left => "42".pad_start(6, "0");
    output <- padded_left;

    let padded_right => "data".pad_end(8, ".");
    output <- padded_right;
}
"#;
    let res = run_source_full(code).expect("String advanced methods should execute");
    assert_eq!(res.output, vec![
        "Vice",
        "5",
        "-1",
        "*****",
        "A|B|C",
        "000042",
        "data...."
    ]);
}

#[test]
fn test_string_repeat_negative_error() {
    let code = r#"
@entry
fn main() {
    let s => "abc";
    output <- s.repeat(-2);
}
"#;
    let err = run_source_full(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Runtime);
    assert!(err.message.contains("repeat count cannot be negative"));
}

// ==========================================
// 9. Math Standard Library Functions
// ==========================================

#[test]
fn test_math_extensions() {
    let code = r#"
@entry
fn main() {
    output <- to_str(math.clamp(15, 0, 10));
    output <- to_str(math.clamp(-5, 0, 10));
    output <- to_str(math.clamp(7, 0, 10));

    output <- to_str(math.sign(42));
    output <- to_str(math.sign(-99));
    output <- to_str(math.sign(0));

    output <- to_str(math.hypot(3.0, 4.0));
    output <- to_str(math.log2(8.0));
}
"#;
    let res = run_source_full(code).expect("Math extensions should execute");
    assert_eq!(res.output, vec![
        "10",
        "0",
        "7",
        "1",
        "-1",
        "0",
        "5",
        "3"
    ]);
}

#[test]
fn test_math_clamp_invalid_bounds() {
    let code = r#"
@entry
fn main() {
    output <- to_str(math.clamp(5, 10, 0));
}
"#;
    let err = run_source_full(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Runtime);
    assert!(err.message.contains("math.clamp: min cannot be greater than max"));
}

#[test]
fn test_math_log_domain_error() {
    let code = r#"
@entry
fn main() {
    output <- to_str(math.log(-1.0));
}
"#;
    let err = run_source_full(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Runtime);
    assert!(err.message.contains("math.log domain error"));
}

// ==========================================
// 10. Advanced Constant Folding & Zero-Runtime DCE
// ==========================================

#[test]
fn test_extended_constant_folding() {
    let code = r#"
@entry
fn main() {
    let str_add1 => "Value: " + 42;
    let str_add2 => 100 + " meters";
    let str_add3 => "Active: " + true;
    let cmp1 => "apple" < "banana";
    let cmp2 => "zebra" >= "ant";
    let folded_math => math.clamp(50, 0, 10) + math.min(10, 20) + math.max(5, 15);
    let folded_len => "hello world".len();

    output <- str_add1;
    output <- str_add2;
    output <- str_add3;
    output <- to_str(cmp1);
    output <- to_str(cmp2);
    output <- to_str(folded_math);
    output <- to_str(folded_len);
}
"#;
    let res = run_source_full(code).expect("Extended constant folding should succeed");
    assert_eq!(res.output, vec![
        "Value: 42",
        "100 meters",
        "Active: true",
        "true",
        "true",
        "35",
        "11"
    ]);
}

#[test]
fn test_data_must_use_rejects_discarded_constructor() {
    let code = r#"
@must_use("TransactionResult must be handled")
data TransactionResult {
    success: bool => true;
    code: int => 0;
}

@entry
fn main() {
    TransactionResult(success => true, code => 200);
}
"#;
    let err = compile_source(code).expect_err("Discarded constructor of @must_use data must fail");
    assert!(
        err.message.contains("TransactionResult") && err.message.contains("@must_use"),
        "Unexpected error: {:?}",
        err
    );
}

#[test]
fn test_data_must_use_rejects_discarded_function_returning_data() {
    let code = r#"
@must_use
data StatusPacket {
    id: int => 1;
}

fn fetch_packet() -> StatusPacket {
    return StatusPacket(id => 42);
}

@entry
fn main() {
    fetch_packet();
}
"#;
    let err = compile_source(code).expect_err("Discarded call returning @must_use data must fail");
    assert!(
        err.message.contains("StatusPacket") && err.message.contains("@must_use"),
        "Unexpected error: {:?}",
        err
    );
}

#[test]
fn test_data_must_use_allows_bound_variable() {
    let code = r#"
@must_use
data Token {
    value: string => "secret";
}

@entry
fn main() {
    let t => Token(value => "auth_123");
    output <- t.value;
}
"#;
    let res = run_source(code).expect("Bound @must_use data must succeed");
    assert_eq!(res, vec!["auth_123"]);
}

#[test]
fn test_runtime_nonreentrant_protection() {
    let code = r#"
@nonreentrant
fn worker(n: int) -> int {
    if (n <= 0) {
        return 0;
    }
    return delegate(n - 1);
}

fn delegate(n: int) -> int {
    return worker(n);
}

@entry
fn main() {
    let res => worker(2);
    output <- to_str(res);
}
"#;
    let err = run_source(code).expect_err("Mutual reentrancy into @nonreentrant function must fail at runtime");
    assert!(
        err.message.contains("Reentrancy violation") && err.message.contains("worker"),
        "Expected reentrancy violation, got: {:?}",
        err
    );
}

#[test]
fn test_array_sort_and_concat() {
    let code = r#"
@entry
fn main() {
    mut nums => [40, 10, 50, 20, 30];
    nums.sort();
    output <- nums.join(", ");

    mut words => ["banana", "apple", "cherry"];
    words.sort();
    output <- words.join(", ");

    let combined => [1, 2].concat([3, 4, 5]);
    output <- combined.join("-");

    let std_sorted => array.sort([9, 3, 7, 1]);
    output <- std_sorted.join(" ");

    let std_concat => array.concat(["a", "b"], ["c", "d"]);
    output <- std_concat.join("");
}
"#;
    let res = run_source(code).expect("Array sort and concat should succeed");
    assert_eq!(res, vec![
        "10, 20, 30, 40, 50",
        "apple, banana, cherry",
        "1-2-3-4-5",
        "1 3 7 9",
        "abcd"
    ]);
}

#[test]
fn test_string_char_at_reverse_lines() {
    let code = r#"
@entry
fn main() {
    let text => "ViceLang";
    output <- text.char_at(0);
    output <- text.char_at(4);

    let rev => text.reverse();
    output <- rev;

    let multiline => "line1\nline2\nline3";
    let lines_arr => multiline.lines();
    output <- lines_arr.join(" | ");

    let std_rev => string.reverse("racecar");
    output <- std_rev;

    let std_char => string.char_at("hello", 1);
    output <- std_char;
}
"#;
    let res = run_source(code).expect("String char_at, reverse, lines should succeed");
    assert_eq!(res, vec![
        "V",
        "L",
        "gnaLeciV",
        "line1 | line2 | line3",
        "racecar",
        "e"
    ]);
}

#[test]
fn test_math_trig_and_constants() {
    let code = r#"
@entry
fn main() {
    let pi_val => math.pi();
    let e_val => math.e();
    let atan_val => math.atan2(1.0, 1.0);
    let deg_val => math.deg(3.141592653589793);
    let rad_val => math.rad(180.0);
    let asin_zero => math.asin(0.0);
    let acos_one => math.acos(1.0);

    output <- to_str(pi_val > 3.14 && pi_val < 3.15);
    output <- to_str(e_val > 2.71 && e_val < 2.72);
    output <- to_str(atan_val > 0.78 && atan_val < 0.79);
    output <- to_str(deg_val >= 179.9 && deg_val <= 180.1);
    output <- to_str(rad_val >= 3.14 && rad_val <= 3.15);
    output <- to_str(asin_zero);
    output <- to_str(acos_one);
}
"#;
    let res = run_source(code).expect("Trig and math constants should succeed");
    assert_eq!(res, vec![
        "true",
        "true",
        "true",
        "true",
        "true",
        "0",
        "0"
    ]);
}

#[test]
fn test_math_asin_domain_error() {
    let code = r#"
@entry
fn main() {
    let val => math.asin(2.5);
    output <- to_str(val);
}
"#;
    let err = run_source(code).expect_err("math.asin out of domain must fail");
    assert!(
        err.message.contains("domain error") && err.message.contains("asin"),
        "Unexpected error: {:?}",
        err
    );
}

#[test]
fn test_math_acos_domain_error() {
    let code = r#"
@entry
fn main() {
    let val => math.acos(-2.5);
    output <- to_str(val);
}
"#;
    let err = run_source(code).expect_err("math.acos out of domain must fail");
    assert!(
        err.message.contains("domain error") && err.message.contains("acos"),
        "Unexpected error: {:?}",
        err
    );
}

// ==========================================
// 12. Bytecode Optimizer & DCE Tests
// ==========================================

#[test]
fn test_bytecode_optimizer_dce_and_jump_threading() {
    let code = r#"
fn compute_value(flag: bool) -> int {
    if (flag) {
        return 100;
        let dead_var => 999;
        return dead_var;
    } else {
        return 200;
        let another_dead => 888;
        return another_dead;
    }
}

@entry
fn main() {
    let a => compute_value(true);
    let b => compute_value(false);
    output <- to_str(a);
    output <- to_str(b);
}
"#;
    let res = run_source(code).expect("Optimizer with dead code should compile and run");
    assert_eq!(res, vec!["100", "200"]);
}

#[test]
fn test_bytecode_optimizer_jump_to_return_threading() {
    let code = r#"
fn early_exit(x: int) -> int {
    while (x > 0) {
        if (x == 5) {
            return 42;
        }
        return 99;
    }
    return 0;
}

@entry
fn main() {
    output <- to_str(early_exit(5));
    output <- to_str(early_exit(10));
    output <- to_str(early_exit(-1));
}
"#;
    let res = run_source(code).expect("Jump to return threading should execute correctly");
    assert_eq!(res, vec!["42", "99", "0"]);
}

// ==========================================
// 13. Map Methods (entries, clone, merge)
// ==========================================

#[test]
fn test_map_entries_and_clone() {
    let code = r#"
@entry
fn main() {
    let mut m => { "alpha": 1, "beta": 2 };
    let cloned => m.clone();
    output <- to_str(cloned.len());
    output <- to_str(cloned.contains("alpha"));

    let entries => m.entries();
    output <- to_str(entries.len());
    let first => entries[0];
    output <- first[0];
    output <- to_str(first[1]);
}
"#;
    let res = run_source(code).expect("Map entries and clone should succeed");
    assert_eq!(res, vec!["2", "true", "2", "alpha", "1"]);
}

#[test]
fn test_map_merge_and_extend() {
    let code = r#"
@entry
fn main() {
    let mut base => { "x": 10, "y": 20 };
    let additions => { "y": 99, "z": 30 };
    base.merge(additions);
    output <- to_str(base.len());
    output <- to_str(base.get("x", 0));
    output <- to_str(base.get("y", 0));
    output <- to_str(base.get("z", 0));
}
"#;
    let res = run_source(code).expect("Map merge should succeed");
    assert_eq!(res, vec!["3", "10", "99", "30"]);
}

// ==========================================
// 14. Advanced Math (sqrt domain, signum, trunc, fract, cbrt, gcd, lcm)
// ==========================================

#[test]
fn test_math_sqrt_domain_error() {
    let code = r#"
@entry
fn main() {
    let val => math.sqrt(-4.0);
    output <- to_str(val);
}
"#;
    let err = run_source(code).expect_err("math.sqrt negative must return domain error");
    assert!(
        err.message.contains("domain error") && err.message.contains("sqrt"),
        "Unexpected error: {:?}",
        err
    );
}

#[test]
fn test_math_signum_trunc_fract_cbrt() {
    let code = r#"
@entry
fn main() {
    output <- to_str(math.signum(50));
    output <- to_str(math.signum(-50));
    output <- to_str(math.signum(0));
    output <- to_str(math.trunc(4.75));
    output <- to_str(math.fract(4.75));
    output <- to_str(math.cbrt(27.0));
}
"#;
    let res = run_source(code).expect("Advanced math operations should execute");
    assert_eq!(res, vec!["1", "-1", "0", "4", "0.75", "3"]);
}

#[test]
fn test_math_gcd_and_lcm() {
    let code = r#"
@entry
fn main() {
    let g1 => math.gcd(48, 18);
    let g2 => math.gcd(101, 10);
    let l1 => math.lcm(12, 18);
    let l2 => math.lcm(7, 5);

    output <- to_str(g1);
    output <- to_str(g2);
    output <- to_str(l1);
    output <- to_str(l2);
}
"#;
    let res = run_source(code).expect("GCD and LCM should compute correctly");
    assert_eq!(res, vec!["6", "1", "36", "35"]);
}

#[test]
fn test_math_constant_folding_extensions() {
    let code = r#"
@entry
fn main() {
    // These should be folded directly into constants by the compiler
    let g => math.gcd(120, 25);
    let l => math.lcm(4, 6);
    let c => math.cbrt(8.0);
    let t => math.trunc(9.99);

    output <- to_str(g);
    output <- to_str(l);
    output <- to_str(c);
    output <- to_str(t);
}
"#;
    let res = run_source(code).expect("Constant folding math extensions should work");
    assert_eq!(res, vec!["5", "12", "2", "9"]);
}


