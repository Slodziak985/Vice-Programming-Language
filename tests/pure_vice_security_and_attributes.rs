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

fn run_source(source: &str) -> Result<Vec<String>, vice::error::ViceError> {
    run_source_full(source).map(|res| res.output)
}

#[test]
fn test_rate_limit_success_within_limit() {
    let source = r#"
@rate_limit(3)
fn ping() -> int {
    return 42;
}

@entry
fn main() {
    let a => ping();
    let b => ping();
    let c => ping();
    output <- a + b + c;
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["126"]);
}

#[test]
fn test_rate_limit_exceeded_raises_error() {
    let source = r#"
@rate_limit(calls => 2)
fn ping() -> int {
    return 1;
}

@entry
fn main() {
    let a => ping();
    let b => ping();
    let c => ping();
    output <- a + b + c;
}
"#;
    let err = run_source(source).unwrap_err();
    assert!(err.message.contains("Rate limit of 2 calls exceeded for function 'ping'"));
}

#[test]
fn test_rate_limit_semantic_validation_zero() {
    let source = r#"
@rate_limit(0)
fn ping() -> int {
    return 1;
}

@entry
fn main() {
    output <- ping();
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("requires a positive integer limit"));
}

#[test]
fn test_rate_limit_conflict_with_unreachable() {
    let source = r#"
@rate_limit(5)
@unreachable
fn dead() {}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @rate_limit and @unreachable"));
}

#[test]
fn test_timeout_fast_execution_succeeds() {
    let source = r#"
@timeout(ms => 1000)
fn quick_math() -> int {
    mut sum => 0;
    for i in 1..100 {
        sum = sum + i;
    }
    return sum;
}

@entry
fn main() {
    output <- quick_math();
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["5050"]);
}

#[test]
fn test_timeout_exceeded_raises_error() {
    let source = r#"
@timeout(ms => 1)
fn slow_loop() -> int {
    mut count => 0;
    for i in 0..10000000 {
        count = count + 1;
    }
    return count;
}

@entry
fn main() {
    output <- slow_loop();
}
"#;
    let err = run_source(source).unwrap_err();
    assert!(err.message.contains("Execution timeout of 1ms exceeded in function 'slow_loop'"));
}

#[test]
fn test_timeout_semantic_validation_zero() {
    let source = r#"
@timeout(0)
fn foo() {}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("requires a positive integer millisecond limit"));
}

#[test]
fn test_isolated_valid_local_computation() {
    let source = r#"
@isolated
fn compute(x: int) -> int {
    mut local_val => x * 2;
    local_val = local_val + 5;
    return local_val;
}

@entry
fn main() {
    output <- compute(10);
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["25"]);
}

#[test]
fn test_isolated_rejects_io() {
    let source = r#"
@isolated
fn do_io() {
    output <- "forbidden";
}

@entry
fn main() {
    do_io();
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot perform I/O (output/emit) inside @isolated function"));
}

#[test]
fn test_isolated_rejects_non_isolated_call() {
    let source = r#"
fn impure_worker() -> int {
    return 42;
}

@isolated
fn isolated_caller() -> int {
    return impure_worker();
}

@entry
fn main() {
    output <- isolated_caller();
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot call non-isolated function 'impure_worker' inside an @isolated function"));
}

#[test]
fn test_isolated_rejects_impure_module() {
    let source = r#"
@isolated
fn measure() {
    time.now();
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot call impure module method 'time.now' inside an @isolated function"));
}

#[test]
fn test_no_float_valid_integer_math() {
    let source = r#"
@no_float
fn factorial(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

@entry
fn main() {
    output <- factorial(5);
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["120"]);
}

#[test]
fn test_no_float_rejects_float_param() {
    let source = r#"
@no_float
fn calc(rate: float) -> int {
    return 10;
}

@entry
fn main() {
    output <- calc(1.5);
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Parameter 'rate' in @no_float function 'calc' cannot have float type"));
}

#[test]
fn test_no_float_rejects_float_return() {
    let source = r#"
@no_float
fn ratio() -> float {
    return 3.14;
}

@entry
fn main() {
    output <- ratio();
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("@no_float function 'ratio' cannot return float"));
}

#[test]
fn test_no_float_rejects_float_literal() {
    let source = r#"
@no_float
fn test_const() -> int {
    let pi => 3.14;
    return 3;
}

@entry
fn main() {
    output <- test_const();
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Float literal '3.14' is not allowed inside @no_float function"));
}

#[test]
fn test_audit_records_calls_and_returns() {
    let source = r#"
@audit
fn add_nums(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    let res => add_nums(10, 20);
    output <- res;
}
"#;
    let result = run_source_full(source).unwrap();
    assert_eq!(result.output, vec!["30"]);
    assert!(result.audit_trail.iter().any(|entry| entry.contains("CALL add_nums(10, 20)")));
    assert!(result.audit_trail.iter().any(|entry| entry.contains("RET add_nums => 30")));
}

#[test]
fn test_immutable_data_prevents_field_mutation() {
    let source = r#"
@immutable
data UserRecord {
    id: int => 0;
    name: string => "";
}

@entry
fn main() {
    mut u => UserRecord(id => 1, name => "Alice");
    u.id = 2;
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot mutate field of read-only data type: UserRecord"));
}

#[test]
fn test_constant_folding_string_and_array_helpers() {
    let source = r#"
@entry
fn main() {
    let s => "hello";
    let p1 => s.pad_start(8, "*");
    let p2 => s.pad_end(8, "!");
    let c => s.char_at(1);
    let joined => ["a", "b", "c"].join("-");
    output <- p1, p2, c, joined;
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["***hello hello!!! e a-b-c"]);
}
