use vice::semantic::SemanticAnalyzer;

#[test]
fn pure_function_accepts_valid_computation() {
    let src = r##"
@pure
fn add(a: int, b: int) -> int {
    let sum => a + b;
    return sum;
}

@entry
fn main() {
    let result => add(10, 32);
    output <- result;
}
"##;
    let (_res, out) = vice::run_source(src, vec![]).expect("should succeed");
    assert_eq!(out, vec!["42"]);
}

#[test]
fn pure_function_rejects_output_and_emit() {
    let src = r##"
@pure
fn impure_output(x: int) -> int {
    output <- x;
    return x;
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject output");
    assert!(err.message.contains("Cannot perform I/O (output/emit) inside pure function"));

    let src_emit = r##"
@pure
fn impure_emit(x: int) -> int {
    emit <- x;
    return x;
}
"##;
    let err_emit = vice::run_source(src_emit, vec![]).expect_err("should reject emit");
    assert!(err_emit.message.contains("Cannot perform I/O (output/emit) inside pure function"));
}

#[test]
fn pure_function_rejects_input() {
    let src = r##"
@pure
fn impure_input() -> int {
    input -> name;
    return 1;
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject input");
    assert!(err.message.contains("Cannot read input inside pure function"));
}

#[test]
fn pure_function_rejects_fail_and_error() {
    let src_fail = r##"
@pure
fn impure_fail(x: int) -> int {
    if (x < 0) {
        fail <- "negative";
    }
    return x;
}
"##;
    let err_fail = vice::run_source(src_fail, vec![]).expect_err("should reject fail");
    assert!(err_fail.message.contains("Cannot fail inside pure function"));

    let src_error = r##"
@pure
fn impure_error(x: int) -> int {
    if (x < 0) {
        error(code => 1) <- "bad";
    }
    return x;
}
"##;
    let err_error = vice::run_source(src_error, vec![]).expect_err("should reject error");
    assert!(err_error.message.contains("Cannot raise runtime error inside pure function"));
}

#[test]
fn pure_function_rejects_parameter_and_global_mutation() {
    let src_param = r##"
data Point {
    x: int => 0;
    y: int => 0;
}

@pure
fn mutate_param(p: Point) -> int {
    p.x = 99;
    return p.x;
}
"##;
    let err_param = vice::run_source(src_param, vec![]).expect_err("should reject field mutation");
    assert!(err_param.message.contains("Cannot mutate field of parameter or global variable 'p' inside pure function"));

    let src_assign_param = r##"
@pure
fn mutate_param_var(mut x: int) -> int {
    x = x + 1;
    return x;
}
"##;
    let err_assign = vice::run_source(src_assign_param, vec![]).expect_err("should reject parameter reassignment");
    assert!(err_assign.message.contains("Cannot mutate parameter 'x' inside pure function"));
}

#[test]
fn pure_function_rejects_calling_impure_function() {
    let src = r##"
fn side_effect() -> int {
    return 100;
}

@pure
fn caller() -> int {
    return side_effect();
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject call to impure function");
    assert!(err.message.contains("Cannot call non-pure function 'side_effect' inside a pure function"));
}

#[test]
fn pure_function_allows_calling_other_pure_function() {
    let src = r##"
@pure
fn helper(x: int) -> int {
    return x * 2;
}

@pure
fn calc(a: int) -> int {
    return helper(a) + 1;
}

@entry
fn main() {
    output <- calc(5);
}
"##;
    let (_res, out) = vice::run_source(src, vec![]).expect("should succeed");
    assert_eq!(out, vec!["11"]);
}

#[test]
fn pure_function_rejects_impure_module_calls() {
    let src_fs = r##"
@pure
fn read_file() -> string {
    return fs.read_text("test.txt");
}
"##;
    let err_fs = vice::run_source(src_fs, vec![]).expect_err("should reject fs");
    assert!(err_fs.message.contains("Cannot call impure module method 'fs.read_text' inside a pure function"));

    let src_time = r##"
@pure
fn get_time() -> int {
    return time.now_ms();
}
"##;
    let err_time = vice::run_source(src_time, vec![]).expect_err("should reject time");
    assert!(err_time.message.contains("Cannot call impure module method 'time.now_ms' inside a pure function"));
}

#[test]
fn pure_function_must_return_value() {
    let src = r##"
@pure
fn no_return(x: int) {
    let y => x + 1;
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject pure function without return");
    assert!(err.message.contains("@pure function 'no_return' must return a value"));
}

#[test]
fn must_use_attribute_prevents_discarding_return_value() {
    let src_discard = r##"
@must_use
fn compute() -> int {
    return 42;
}

@entry
fn main() {
    compute();
}
"##;
    let err = vice::run_source(src_discard, vec![]).expect_err("should reject unused return value");
    assert!(err.message.contains("Unused return value from function 'compute' marked with @must_use"));
}

#[test]
fn must_use_attribute_with_custom_message() {
    let src_msg = r##"
@must_use(message => "Return code must be checked")
fn execute() -> int {
    return 0;
}

@entry
fn main() {
    execute();
}
"##;
    let err = vice::run_source(src_msg, vec![]).expect_err("should report custom message");
    assert!(err.message.contains("Return code must be checked"));
}

#[test]
fn must_use_attribute_allows_used_value() {
    let src = r##"
@must_use
fn compute() -> int {
    return 42;
}

@entry
fn main() {
    let val => compute();
    output <- val;
}
"##;
    let (_res, out) = vice::run_source(src, vec![]).expect("should succeed when assigned");
    assert_eq!(out, vec!["42"]);
}

#[test]
fn must_use_attribute_rejects_void_function() {
    let src = r##"
@must_use
fn do_nothing() {
    let x => 1;
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject void must_use");
    assert!(err.message.contains("Function 'do_nothing' marked with @must_use must have a return value"));
}

#[test]
fn deprecated_attribute_records_warnings() {
    let src = r##"
@deprecated(message => "Use safe_add instead")
fn old_add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    let result => old_add(1, 2);
    output <- result;
}
"##;
    let program = vice::parse_source(src).expect("should parse");
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program).expect("semantic check should pass");
    let warnings = analyzer.get_warnings();
    assert!(!warnings.is_empty(), "should record warning");
    assert!(warnings.iter().any(|w| w.contains("Item 'old_add' is deprecated: Use safe_add instead")));
}

#[test]
fn tailrec_attribute_accepts_valid_tail_recursion() {
    let src = r##"
@tailrec
fn sum_acc(n: int, acc: int) -> int {
    if (n <= 0) {
        return acc;
    }
    return sum_acc(n - 1, acc + n);
}

@entry
fn main() {
    let total => sum_acc(10, 0);
    output <- total;
}
"##;
    let (_res, out) = vice::run_source(src, vec![]).expect("should compile and run tail recursion");
    assert_eq!(out, vec!["55"]);
}

#[test]
fn tailrec_attribute_rejects_non_tail_recursion() {
    let src = r##"
@tailrec
fn factorial(n: int) -> int {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject non-tail call");
    assert!(err.message.contains("Function 'factorial' is marked with @tailrec but recursive call is not in tail position"));
}

#[test]
fn tailrec_attribute_rejects_non_recursive_function() {
    let src = r##"
@tailrec
fn non_rec(x: int) -> int {
    return x + 1;
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject non-recursive function");
    assert!(err.message.contains("Function 'non_rec' is marked with @tailrec but contains no recursive calls"));
}

#[test]
fn inline_attribute_rejects_recursive_function() {
    let src = r##"
@inline
fn fib(n: int) -> int {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
"##;
    let err = vice::run_source(src, vec![]).expect_err("should reject recursive inline");
    assert!(err.message.contains("Function 'fib' cannot be marked with @inline because it is recursive"));
}

#[test]
fn memoize_attribute_requires_pure_or_read_and_params() {
    let src_no_params = r##"
@memoize
@pure
fn zero_param() -> int {
    return 42;
}
"##;
    let err_no_params = vice::run_source(src_no_params, vec![]).expect_err("should reject no params");
    assert!(err_no_params.message.contains("Function 'zero_param' marked with @memoize must take at least one parameter"));

    let src_not_pure = r##"
@memoize
fn impure_memo(x: int) -> int {
    return x * 2;
}
"##;
    let err_not_pure = vice::run_source(src_not_pure, vec![]).expect_err("should reject impure memoize");
    assert!(err_not_pure.message.contains("Function 'impure_memo' marked with @memoize must also be marked @pure or @read"));

    let src_valid = r##"
@memoize
@pure
fn square(x: int) -> int {
    return x * x;
}

@entry
fn main() {
    output <- square(9);
}
"##;
    let (_res, out) = vice::run_source(src_valid, vec![]).expect("should accept valid memoize");
    assert_eq!(out, vec!["81"]);
}

#[test]
fn tailrec_avoids_call_depth_exhaustion_on_deep_recursion() {
    let src = r##"
@tailrec
fn loop_count(n: int) -> int {
    if (n <= 0) {
        return 0;
    }
    return loop_count(n - 1);
}

@entry
fn main() {
    // 500 exceeds max_call_depth of 64
    let res => loop_count(500);
    output <- res;
}
"##;
    let (_res, out) = vice::run_source(src, vec![]).expect("should not exhaust call depth with TCO");
    assert_eq!(out, vec!["0"]);
}

#[test]
fn compiler_constant_folding_optimizes_arithmetic() {
    let src = r##"
@entry
fn main() {
    let x => 2 + 3 * 4;
    output <- x;
}
"##;
    let program = vice::parse_source(src).expect("should parse");
    let mut analyzer = vice::semantic::SemanticAnalyzer::new();
    analyzer.analyze(&program).expect("should analyze");
    let compiled = vice::vm::compiler::Compiler::new().compile(&program).expect("should compile");
    let main_fn = compiled.functions.get("main").expect("main exists");

    // Check that Add and Mul opcodes were folded away
    let has_add = main_fn.chunk.code.iter().any(|op| matches!(op, vice::vm::opcodes::Opcode::Add));
    let has_mul = main_fn.chunk.code.iter().any(|op| matches!(op, vice::vm::opcodes::Opcode::Mul));
    assert!(!has_add, "Add opcode should be constant folded");
    assert!(!has_mul, "Mul opcode should be constant folded");

    // And verify execution outputs 14
    let (_res, out) = vice::run_source(src, vec![]).expect("should run");
    assert_eq!(out, vec!["14"]);
}


