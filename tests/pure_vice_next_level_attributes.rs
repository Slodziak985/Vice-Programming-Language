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

// ---------------------------------------------------------
// @leaf tests
// ---------------------------------------------------------

#[test]
fn test_leaf_function_success_local_computation() {
    let source = r#"
@leaf
fn compute(a: int, b: int) -> int {
    let mut sum => a + b;
    if (sum > 10) {
        sum = sum * 2;
    } else {
        sum = sum + 5;
    }
    return sum;
}

@entry
fn main() {
    let res => compute(6, 7);
    emit <- res;
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["26"]);
}

#[test]
fn test_leaf_function_rejects_function_call() {
    let source = r#"
fn helper() -> int {
    return 1;
}

@leaf
fn leaf_bad() -> int {
    return helper();
}

@entry
fn main() {
    emit <- leaf_bad();
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot call function 'helper' inside @leaf function"));
}

#[test]
fn test_leaf_function_rejects_method_call() {
    let source = r#"
@leaf
fn leaf_method_bad() -> int {
    let mut arr => [1, 2, 3];
    arr.push(4);
    return arr.len();
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot call method 'push' inside @leaf function"));
}

#[test]
fn test_leaf_function_conflict_with_tailrec() {
    let source = r#"
@leaf
@tailrec
fn bad(n: int) -> int {
    return n;
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @leaf and @tailrec"));
}

#[test]
fn test_leaf_function_conflict_with_retry() {
    let source = r#"
@leaf
@retry(3)
fn bad() -> Result {
    return Result.Ok(1);
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @leaf and @retry"));
}

#[test]
fn test_leaf_function_conflict_with_max_depth() {
    let source = r#"
@leaf
@max_depth(5)
fn bad() -> int {
    return 1;
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @leaf and @max_depth"));
}

// ---------------------------------------------------------
// @wrapping tests
// ---------------------------------------------------------

#[test]
fn test_wrapping_attribute_behavior() {
    let source = r#"
@wrapping
fn add_wrap(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    let max_val => 9223372036854775807;
    let wrapped => add_wrap(max_val, 1);
    emit <- wrapped;
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["-9223372036854775808"]);
}

#[test]
fn test_wrapping_attribute_conflict_with_checked() {
    let source = r#"
@wrapping
@checked
fn bad(a: int) -> int {
    return a + 1;
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @wrapping and @checked"));
}

#[test]
fn test_wrapping_attribute_conflict_with_saturating() {
    let source = r#"
@wrapping
@saturating
fn bad(a: int) -> int {
    return a + 1;
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @wrapping and @saturating"));
}

// ---------------------------------------------------------
// @nothrow / @no_fail tests
// ---------------------------------------------------------

#[test]
fn test_nothrow_function_success() {
    let source = r#"
@nothrow
fn safe_calc(x: int) -> int {
    return x * 10;
}

@entry
fn main() {
    emit <- safe_calc(7);
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["70"]);
}

#[test]
fn test_nothrow_function_rejects_fail() {
    let source = r#"
@nothrow
fn bad() {
    fail <- "boom";
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot fail inside @nothrow function"));
}

#[test]
fn test_nothrow_function_conflict_with_fallible() {
    let source = r#"
@nothrow
@fallible
fn bad() -> Result<int, string> {
    return ok(1);
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @nothrow and @fallible"));
}

#[test]
fn test_nothrow_function_rejects_calling_fallible() {
    let source = r#"
@fallible
fn might_fail() -> Result<int, string> {
    return ok(1);
}

@nothrow
fn calls_fallible() {
    let _r => might_fail();
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot call @fallible function 'might_fail' directly inside @nothrow function"));
}

#[test]
fn test_nothrow_function_rejects_unwrap() {
    let source = r#"
@nothrow
fn bad_unwrap(r: Result<int, string>) -> int {
    return r!;
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot use unwrap operator '!' inside @nothrow function"));
}

// ---------------------------------------------------------
// @sealed enum tests
// ---------------------------------------------------------

#[test]
fn test_sealed_enum_same_file_impl_allowed() {
    let source = r#"
@sealed
enum Color {
    Red,
    Green,
    Blue
}

impl Color {
    fn is_red(self) -> bool {
        return match (self) {
            Color.Red => true,
            _ => false
        };
    }
}

@entry
fn main() {
    let c => Color.Red;
    emit <- c.is_red();
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["true"]);
}

#[test]
fn test_sealed_enum_external_file_impl_rejected() {
    let file1 = r#"
@sealed
enum TrafficLight {
    Red,
    Yellow,
    Green
}
"#;
    let file2 = r#"
use "file1.vice";

impl TrafficLight {
    fn code(self) -> int {
        return 0;
    }
}

@entry
fn main() {}
"#;
    let mut ast1 = vice::parse_source(file1).unwrap();
    ast1.assign_source(0);
    let mut ast2 = vice::parse_source(file2).unwrap();
    ast2.assign_source(1);

    let mut merged = ast1;
    merged.functions.extend(ast2.functions);
    merged.impls.extend(ast2.impls);
    merged.top_level.extend(ast2.top_level);

    let mut semantic = vice::semantic::SemanticAnalyzer::new();
    merged.sources = vec!["file1.vice".into(), "file2.vice".into()];
    let err = semantic.analyze(&merged).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Enum 'TrafficLight' is @sealed and cannot be implemented outside its defining file"));
}

// ---------------------------------------------------------
// @no_io tests
// ---------------------------------------------------------

#[test]
fn test_no_io_function_success() {
    let source = r#"
@no_io
fn add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    emit <- add(10, 20);
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["30"]);
}

#[test]
fn test_no_io_function_rejects_emit() {
    let source = r#"
@no_io
fn bad() {
    emit <- "leak";
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot perform I/O (output/emit) inside @no_io function"));
}

#[test]
fn test_no_io_function_rejects_input() {
    let source = r#"
@no_io
fn bad() {
    input -> user_input;
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot read input inside @no_io function"));
}

#[test]
fn test_no_io_function_rejects_print() {
    let source = r#"
@no_io
fn bad() {
    println("hello");
}

@entry
fn main() {}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot call I/O function 'println' inside @no_io function"));
}

// ---------------------------------------------------------
// Peephole constant folding tests
// ---------------------------------------------------------

#[test]
fn test_peephole_bytecode_constant_folding() {
    use vice::vm::opcodes::Opcode;
    let mut chunk = vice::vm::compiler::Chunk::new();
    let idx1 = chunk.add_constant(vice::value::Value::Int(100));
    let idx2 = chunk.add_constant(vice::value::Value::Int(50));
    chunk.write(Opcode::LoadConst(idx1), 1);
    chunk.write(Opcode::LoadConst(idx2), 1);
    chunk.write(Opcode::Add, 1);
    chunk.write(Opcode::Return, 1);

    vice::vm::compiler::optimize_chunk(&mut chunk);

    // After optimization: LoadConst(100), LoadConst(50), Add should be folded to LoadConst(150), Return!
    assert_eq!(chunk.code.len(), 2);
    if let Opcode::LoadConst(idx) = chunk.code[0] {
        assert_eq!(chunk.constants[idx], vice::value::Value::Int(150));
    } else {
        panic!("Expected LoadConst opcode");
    }
}

#[test]
fn test_peephole_bytecode_unary_folding() {
    use vice::vm::opcodes::Opcode;
    let mut chunk = vice::vm::compiler::Chunk::new();
    let idx = chunk.add_constant(vice::value::Value::Bool(false));
    chunk.write(Opcode::LoadConst(idx), 1);
    chunk.write(Opcode::Not, 1);
    chunk.write(Opcode::Return, 1);

    vice::vm::compiler::optimize_chunk(&mut chunk);

    // After optimization: LoadConst(false), Not should fold to LoadConst(true), Return!
    assert_eq!(chunk.code.len(), 2);
    if let Opcode::LoadConst(new_idx) = chunk.code[0] {
        assert_eq!(chunk.constants[new_idx], vice::value::Value::Bool(true));
    } else {
        panic!("Expected LoadConst opcode");
    }
}

#[test]
fn test_peephole_bytecode_div_mod_pow_folding() {
    use vice::vm::opcodes::Opcode;
    let mut chunk = vice::vm::compiler::Chunk::new();
    let idx1 = chunk.add_constant(vice::value::Value::Int(2));
    let idx2 = chunk.add_constant(vice::value::Value::Int(10));
    chunk.write(Opcode::LoadConst(idx1), 1);
    chunk.write(Opcode::LoadConst(idx2), 1);
    chunk.write(Opcode::Pow, 1);
    chunk.write(Opcode::Return, 1);

    vice::vm::compiler::optimize_chunk(&mut chunk);

    // 2^10 should fold to 1024
    assert_eq!(chunk.code.len(), 2);
    if let Opcode::LoadConst(idx) = chunk.code[0] {
        assert_eq!(chunk.constants[idx], vice::value::Value::Int(1024));
    } else {
        panic!("Expected LoadConst opcode");
    }
}

#[test]
fn test_peephole_bytecode_logic_folding() {
    use vice::vm::opcodes::Opcode;
    let mut chunk = vice::vm::compiler::Chunk::new();
    let idx1 = chunk.add_constant(vice::value::Value::Bool(true));
    let idx2 = chunk.add_constant(vice::value::Value::Bool(false));
    chunk.write(Opcode::LoadConst(idx1), 1);
    chunk.write(Opcode::LoadConst(idx2), 1);
    chunk.write(Opcode::And, 1);
    chunk.write(Opcode::Return, 1);

    vice::vm::compiler::optimize_chunk(&mut chunk);

    // true && false should fold to false
    assert_eq!(chunk.code.len(), 2);
    if let Opcode::LoadConst(idx) = chunk.code[0] {
        assert_eq!(chunk.constants[idx], vice::value::Value::Bool(false));
    } else {
        panic!("Expected LoadConst opcode");
    }
}

// ---------------------------------------------------------
// @private / @internal tests
// ---------------------------------------------------------

#[test]
fn test_private_function_same_file_allowed() {
    let source = r#"
@private
fn helper(x: int) -> int {
    return x * 3;
}

@entry
fn main() {
    emit <- helper(4);
}
"#;
    let out = run_source(source).unwrap();
    assert_eq!(out, vec!["12"]);
}

#[test]
fn test_private_function_cross_file_rejected() {
    let file1 = r#"
@private
fn secret_algo() -> int {
    return 42;
}
"#;
    let file2 = r#"
use "file1.vice";

@entry
fn main() {
    let _x => secret_algo();
}
"#;
    let mut ast1 = vice::parse_source(file1).unwrap();
    ast1.assign_source(0);
    let mut ast2 = vice::parse_source(file2).unwrap();
    ast2.assign_source(1);

    let mut merged = ast1;
    merged.functions.extend(ast2.functions);
    merged.top_level.extend(ast2.top_level);
    merged.sources = vec!["file1.vice".into(), "file2.vice".into()];

    let mut analyzer = vice::semantic::SemanticAnalyzer::new();
    let err = analyzer.analyze(&merged).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Function 'secret_algo' is marked @private and cannot be called from another file"));
}

#[test]
fn test_private_function_entry_rejected() {
    let source = r#"
@private
@entry
fn main() {
    emit <- 1;
}
"#;
    let err = check_source(source).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @private and @entry"));
}

#[test]
fn test_private_data_cross_file_rejected() {
    let file1 = r#"
@private
data SecretConfig {
    key: int
}
"#;
    let file2 = r#"
use "file1.vice";

@entry
fn main() {
    let _c => SecretConfig(key => 123);
}
"#;
    let mut ast1 = vice::parse_source(file1).unwrap();
    ast1.assign_source(0);
    let mut ast2 = vice::parse_source(file2).unwrap();
    ast2.assign_source(1);

    let mut merged = ast1;
    merged.data_declarations.extend(ast2.data_declarations);
    merged.functions.extend(ast2.functions);
    merged.top_level.extend(ast2.top_level);
    merged.sources = vec!["file1.vice".into(), "file2.vice".into()];

    let mut analyzer = vice::semantic::SemanticAnalyzer::new();
    let err = analyzer.analyze(&merged).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Data type 'SecretConfig' is marked @private and cannot be constructed from another file"));
}

#[test]
fn test_private_enum_cross_file_rejected() {
    let file1 = r#"
@private
enum SecretStatus {
    Locked,
    Unlocked
}
"#;
    let file2 = r#"
use "file1.vice";

@entry
fn main() {
    let _s => SecretStatus.Locked;
}
"#;
    let mut ast1 = vice::parse_source(file1).unwrap();
    ast1.assign_source(0);
    let mut ast2 = vice::parse_source(file2).unwrap();
    ast2.assign_source(1);

    let mut merged = ast1;
    merged.top_level.extend(ast2.top_level);
    merged.functions.extend(ast2.functions);
    merged.sources = vec!["file1.vice".into(), "file2.vice".into()];

    let mut analyzer = vice::semantic::SemanticAnalyzer::new();
    let err = analyzer.analyze(&merged).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Enum 'SecretStatus' is marked @private and cannot be accessed from another file"));
}

