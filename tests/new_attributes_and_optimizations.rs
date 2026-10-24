use vice::error::ViceErrorType;
use vice::value::Value;
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

#[test]
fn test_no_inline_and_cold_attributes_work() {
    let code = r#"
@no_inline
fn heavy_calc(x: int) -> int {
    return x * 2;
}

@cold
fn slow_path() -> string {
    return "unlikely";
}

@entry
fn main() {
    output <- to_str(heavy_calc(21));
    output <- slow_path();
}
"#;
    let res = run_source_full(code).expect("Should compile and run");
    assert_eq!(res.output, vec!["42", "unlikely"]);
}

#[test]
fn test_no_inline_and_inline_conflict() {
    let code = r#"
@inline
@no_inline
fn bad() -> int {
    return 1;
}

@entry
fn main() {
    output <- to_str(bad());
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @inline and @no_inline"));
}

#[test]
fn test_cold_and_inline_conflict() {
    let code = r#"
@inline
@cold
fn bad() -> int {
    return 1;
}

@entry
fn main() {
    output <- to_str(bad());
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @cold and @inline"));
}

#[test]
fn test_nonreentrant_attribute_rejects_recursion() {
    let code = r#"
@nonreentrant
fn countdown(n: int) -> int {
    if (n <= 0) {
        return 0;
    }
    return countdown(n - 1);
}

@entry
fn main() {
    output <- to_str(countdown(5));
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("marked with @nonreentrant cannot call itself"));
}

#[test]
fn test_nonreentrant_and_tailrec_conflict() {
    let code = r#"
@nonreentrant
@tailrec
fn bad(n: int) -> int {
    return n;
}

@entry
fn main() {
    output <- to_str(bad(1));
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @tailrec and @nonreentrant"));
}

#[test]
fn test_safe_attribute_valid_function() {
    let code = r#"
@safe
fn add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    output <- to_str(add(10, 20));
}
"#;
    let res = run_source_full(code).expect("Safe function should pass");
    assert_eq!(res.output, vec!["30"]);
}

#[test]
fn test_safe_attribute_rejects_fail_and_error() {
    let code_fail = r#"
@safe
fn bad_fail() {
    fail <- "forbidden";
}

@entry
fn main() {
    bad_fail();
}
"#;
    let err_fail = check_source(code_fail).unwrap_err();
    assert_eq!(err_fail.error_type, ViceErrorType::Semantic);
    assert!(err_fail.message.contains("inside @safe function"));

    let code_error = r#"
@safe
fn bad_error() {
    error() <- "forbidden";
}

@entry
fn main() {
    bad_error();
}
"#;
    let err_error = check_source(code_error).unwrap_err();
    assert_eq!(err_error.error_type, ViceErrorType::Semantic);
    assert!(err_error.message.contains("inside @safe function"));
}

#[test]
fn test_safe_attribute_rejects_unwrap_operator() {
    let code = r#"
@safe
fn bad_unwrap(opt: Option<int>) -> int {
    return opt!;
}

@entry
fn main() {
    output <- to_str(bad_unwrap(some(10)));
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("inside @safe function"));
}

#[test]
fn test_safe_attribute_rejects_impure_module_calls() {
    let code = r#"
@safe
fn bad_fs() {
    fs.read("test.txt");
}

@entry
fn main() {
    bad_fs();
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot call impure module method 'fs.read' inside a @safe function"));
}

#[test]
fn test_memoize_attribute_execution_and_caching() {
    let code = r#"
@memoize
@pure
fn fib(n: int) -> int {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

@entry
fn main() {
    output <- to_str(fib(20));
}
"#;
    let res = run_source_full(code).expect("Memoized fibonacci should succeed");
    assert_eq!(res.output, vec!["6765"]);
    // Without memoization, fib(20) would take ~21,891 calls.
    // With memoization, instructions should be very small.
    assert!(res.instructions < 2000, "Instruction count was {}", res.instructions);
}

#[test]
fn test_dce_optimizations() {
    let code = r#"
@entry
fn main() {
    if (false) {
        output <- "unreachable branch";
    } else {
        output <- "reachable else";
    }

    if (true) {
        output <- "reachable then";
    } else {
        output <- "unreachable else";
    }

    while (false) {
        output <- "unreachable while";
    }
}
"#;
    let res = run_source_full(code).expect("DCE code runs");
    assert_eq!(res.output, vec!["reachable else", "reachable then"]);

    // Check that compiler didn't even emit the unreachable strings into constants!
    let program = vice::parse_source(code).unwrap();
    let compiled = vice::vm::compiler::Compiler::new().compile(&program).unwrap();
    let main_chunk = &compiled.functions.get("main").unwrap().chunk;
    for c in &main_chunk.constants {
        if let Value::String(s) = c {
            assert_ne!(s, "unreachable branch");
            assert_ne!(s, "unreachable else");
            assert_ne!(s, "unreachable while");
        }
    }
}

#[test]
fn test_mixed_number_and_interpolated_string_constant_folding() {
    let code = r#"
@entry
fn main() {
    let a => 10 + 2.5;
    let b => 4 * 0.5;
    let c => 5.0 - 2;
    let s => "folded value: ${40 + 2}";
    output <- to_str(a);
    output <- to_str(b);
    output <- to_str(c);
    output <- s;
}
"#;
    let res = run_source_full(code).expect("Mixed constant folding works");
    assert_eq!(res.output, vec!["12.5", "2", "3", "folded value: 42"]);
}

#[test]
fn test_array_instance_methods() {
    let code = r#"
@entry
fn main() {
    mut arr: int[] => [1, 2, 3];
    output <- to_str(arr.len());
    output <- to_str(arr.is_empty());
    output <- to_str(arr.first());
    output <- to_str(arr.last());
    output <- to_str(arr.contains(2));
    output <- to_str(arr.contains(99));

    arr.reverse();
    output <- arr.join("-");

    arr.clear();
    output <- to_str(arr.len());
    output <- to_str(arr.is_empty());
}
"#;
    let res = run_source_full(code).expect("Array methods execute");
    assert_eq!(res.output, vec![
        "3",
        "false",
        "1",
        "3",
        "true",
        "false",
        "3-2-1",
        "0",
        "true"
    ]);
}

#[test]
fn test_string_instance_methods() {
    let code = r#"
@entry
fn main() {
    let text => "  Hello, World!  ";
    let trimmed => text.trim();
    output <- trimmed;
    output <- trimmed.lower();
    output <- trimmed.upper();
    output <- to_str(trimmed.len());
    output <- to_str(trimmed.is_empty());
    output <- to_str(trimmed.contains("World"));
    output <- to_str(trimmed.starts_with("Hello"));
    output <- to_str(trimmed.ends_with("!"));
    output <- trimmed.replace("World", "Vice");

    let parts => trimmed.split(",");
    output <- parts.join(" | ");
}
"#;
    let res = run_source_full(code).expect("String methods execute");
    assert_eq!(res.output, vec![
        "Hello, World!",
        "hello, world!",
        "HELLO, WORLD!",
        "13",
        "false",
        "true",
        "true",
        "true",
        "Hello, Vice!",
        "Hello |  World!"
    ]);
}

#[test]
fn test_security_bounds_string_limit() {
    // Repeated string doubling until limit is hit
    let code = r#"
@entry
fn main() {
    mut s => "A";
    mut i => 0;
    while (i < 30) {
        s = s + s;
        i += 1;
    }
}
"#;
    let err = run_source_full(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Runtime);
    assert!(err.message.contains("String size exceeds maximum limit"));
}

