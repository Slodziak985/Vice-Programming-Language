use vice::error::ViceErrorType;
use vice::vm::machine::Vm;
use vice::vm::opcodes::Opcode;

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

// ==========================================
// 1. @test Attribute
// ==========================================

#[test]
fn test_attribute_test_valid_zero_params() {
    let code = r#"
@test
fn verify_basic_addition() {
    let a => 10;
    let b => 20;
}

@entry
fn main() {
    output <- "ready";
}
"#;
    assert!(check_source(code).is_ok());
    let out = run_source(code).expect("Should run main");
    assert_eq!(out, vec!["ready"]);
}

#[test]
fn test_attribute_test_rejects_parameters() {
    let code = r#"
@test
fn test_with_params(x: int) {
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot take parameters"));
}

#[test]
fn test_attribute_test_rejects_entry_conflict() {
    let code = r#"
@entry
@test
fn main() {
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("cannot be marked with both @test and @entry"));
}

// ==========================================
// 2. @const Attribute
// ==========================================

#[test]
fn test_attribute_const_valid_computation() {
    let code = r#"
@const
fn square(x: int) -> int {
    return x * x;
}

@entry
fn main() {
    let res => square(5);
    output <- to_str(res);
}
"#;
    let out = run_source(code).expect("Pure const function should work");
    assert_eq!(out, vec!["25"]);
}

#[test]
fn test_attribute_const_rejects_void_return() {
    let code = r#"
@const
fn do_nothing() {
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("must specify a non-void return type"));
}

#[test]
fn test_attribute_const_rejects_mut_variable() {
    let code = r#"
@const
fn bad_const(x: int) -> int {
    mut counter => x;
    return counter;
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot declare mutable variable"));
}

#[test]
fn test_attribute_const_rejects_io_output() {
    let code = r#"
@const
fn noisy_const(x: int) -> int {
    output <- "log";
    return x;
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot perform I/O"));
}

#[test]
fn test_attribute_const_rejects_calling_non_const() {
    let code = r#"
fn impure_func() -> int {
    return 42;
}

@const
fn caller() -> int {
    return impure_func();
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot call non-@const function"));
}

// ==========================================
// 3. @no_panic Attribute
// ==========================================

#[test]
fn test_attribute_no_panic_valid_safe_code() {
    let code = r#"
@no_panic
fn safe_add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    output <- to_str(safe_add(3, 4));
}
"#;
    let out = run_source(code).expect("Safe @no_panic should run cleanly");
    assert_eq!(out, vec!["7"]);
}

#[test]
fn test_attribute_no_panic_rejects_unwrap() {
    let code = r#"
@no_panic
fn bad_unwrap(opt: Option) -> int {
    return opt!;
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot use unwrap operator"));
}

#[test]
fn test_attribute_no_panic_rejects_fail() {
    let code = r#"
@no_panic
fn bad_fail() -> int {
    fail <- "critical error";
    return 0;
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot fail inside @no_panic function"));
}

#[test]
fn test_attribute_no_panic_rejects_error() {
    let code = r#"
@no_panic
fn bad_error() -> int {
    error() <- "raised";
    return 0;
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot raise runtime error inside @no_panic function"));
}

// ==========================================
// 4. @record Attribute
// ==========================================

#[test]
fn test_attribute_record_immutability() {
    let code = r#"
@record
data Point {
    x: int => 0;
    y: int => 0;
}

@entry
fn main() {
    let p => Point(x => 10, y => 20);
    p.x = 30;
}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("Cannot mutate field of @record type"));
}

// ==========================================
// 5. @bounded Attribute
// ==========================================

#[test]
fn test_attribute_bounded_min_greater_than_max() {
    let code = r#"
@bounded(min => 100, max => 10)
fn invalid_bounds(x: int) -> int {
    return x;
}

@entry
fn main() {}
"#;
    let err = check_source(code).unwrap_err();
    assert_eq!(err.error_type, ViceErrorType::Semantic);
    assert!(err.message.contains("@bounded: min (100) cannot be greater than max (10)"));
}

// ==========================================
// 6. @sealed Attribute
// ==========================================

#[test]
fn test_attribute_sealed_trait_same_file_works() {
    let code = r#"
@sealed
trait InternalContract {
    fn code(self) -> int;
}

data Service { id: int => 1; }

impl InternalContract for Service {
    fn code(self) -> int {
        return self.id;
    }
}

@entry
fn main() {
    let s => Service();
    output <- to_str(s.code());
}
"#;
    let out = run_source(code).expect("@sealed trait in same file should compile and run");
    assert_eq!(out, vec!["1"]);
}

// ==========================================
// 7. New Array Methods
// ==========================================

#[test]
fn test_array_swap_method_runtime() {
    let code = r#"
@entry
fn main() {
    mut arr => [10, 20, 30, 40];
    arr.swap(1, 3);
    output <- to_str(arr[0]);
    output <- to_str(arr[1]);
    output <- to_str(arr[2]);
    output <- to_str(arr[3]);
}
"#;
    let out = run_source(code).expect("Array swap should work");
    assert_eq!(out, vec!["10", "40", "30", "20"]);
}

#[test]
fn test_array_count_method_runtime() {
    let code = r#"
@entry
fn main() {
    let arr => [1, 2, 3, 2, 4, 2, 5];
    let c => arr.count(2);
    let c9 => arr.count(9);
    output <- to_str(c);
    output <- to_str(c9);
}
"#;
    let out = run_source(code).expect("Array count should work");
    assert_eq!(out, vec!["3", "0"]);
}

#[test]
fn test_array_take_and_drop_method_runtime() {
    let code = r#"
@entry
fn main() {
    let arr => [10, 20, 30, 40, 50];
    let t => arr.take(3);
    let d => arr.drop(3);
    output <- to_str(len(t));
    output <- to_str(t[0]);
    output <- to_str(t[2]);
    output <- to_str(len(d));
    output <- to_str(d[0]);
    output <- to_str(d[1]);
}
"#;
    let out = run_source(code).expect("Array take and drop should work");
    assert_eq!(out, vec!["3", "10", "30", "2", "40", "50"]);
}

// ==========================================
// 8. New String Methods
// ==========================================

#[test]
fn test_string_count_method_runtime() {
    let code = r#"
@entry
fn main() {
    let s => "banana";
    let c_an => s.count("an");
    let c_a => s.count("a");
    let c_z => s.count("z");
    output <- to_str(c_an);
    output <- to_str(c_a);
    output <- to_str(c_z);
}
"#;
    let out = run_source(code).expect("String count should work");
    assert_eq!(out, vec!["2", "3", "0"]);
}

#[test]
fn test_string_capitalize_method_runtime() {
    let code = r#"
@entry
fn main() {
    let s => "vice";
    output <- s.capitalize();
    let empty => "";
    output <- empty.capitalize();
}
"#;
    let out = run_source(code).expect("String capitalize should work");
    assert_eq!(out, vec!["Vice", ""]);
}

// ==========================================
// 9. New Map Methods
// ==========================================

#[test]
fn test_map_has_key_and_count_runtime() {
    let code = r#"
@entry
fn main() {
    let m => { "alpha": 1, "beta": 2 };
    let has_a => m.has_key("alpha");
    let has_g => m.has_key("gamma");
    let total => m.count();
    output <- to_str(has_a);
    output <- to_str(has_g);
    output <- to_str(total);
}
"#;
    let out = run_source(code).expect("Map has_key and count should work");
    assert_eq!(out, vec!["true", "false", "2"]);
}

// ==========================================
// 10. New Native Math Functions
// ==========================================

#[test]
fn test_math_is_nan_and_is_finite() {
    let code = r#"
@entry
fn main() {
    let fin => math.is_finite(42.5);
    let nan_val => math.is_nan(42.5);
    output <- to_str(fin);
    output <- to_str(nan_val);
}
"#;
    let out = run_source(code).expect("is_finite and is_nan should work");
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn test_math_factorial_and_round_to() {
    let code = r#"
@entry
fn main() {
    let f0 => math.factorial(0);
    let f5 => math.factorial(5);
    let r => math.round_to(3.1415926, 2);
    let r4 => math.round_to(3.1415926, 4);
    output <- to_str(f0);
    output <- to_str(f5);
    output <- to_str(r);
    output <- to_str(r4);
}
"#;
    let out = run_source(code).expect("math.factorial and round_to should work");
    assert_eq!(out, vec!["1", "120", "3.14", "3.1416"]);
}

// ==========================================
// 11. Constant Folding Extensions
// ==========================================

#[test]
fn test_compiler_constant_folding_extensions() {
    let code = r#"
@entry
fn main() {
    let arr => [10, 20, 30].take(2);
    let cnt => [1, 2, 2, 3].count(2);
    let scnt => "mississippi".count("ss");
    let cap => "hello".capitalize();
    let fact => math.factorial(6);
    let fin => math.is_finite(100.0);

    output <- to_str(len(arr));
    output <- to_str(cnt);
    output <- to_str(scnt);
    output <- cap;
    output <- to_str(fact);
    output <- to_str(fin);
}
"#;
    let out = run_source(code).expect("Constant folding of new methods should execute smoothly");
    assert_eq!(out, vec!["2", "2", "2", "Hello", "720", "true"]);
}

// ==========================================
// 12. Peephole Optimizer Redundant Pair & Jump Elimination
// ==========================================

#[test]
fn test_peephole_optimizer_eliminates_redundant_pairs() {
    let mut chunk = vice::vm::compiler::Chunk::new();
    // Pair 1: Neg, Neg
    chunk.write(Opcode::Neg, 1);
    chunk.write(Opcode::Neg, 1);
    // Pair 2: Swap, Swap
    chunk.write(Opcode::Swap, 2);
    chunk.write(Opcode::Swap, 2);
    // Pair 3: LoadLocal(0), Pop
    chunk.write(Opcode::LoadLocal(0), 3);
    chunk.write(Opcode::Pop, 3);
    // Pair 4: TruncateToInt, TruncateToInt
    chunk.write(Opcode::TruncateToInt, 4);
    chunk.write(Opcode::TruncateToInt, 4);
    // Preserved instruction: Return
    chunk.write(Opcode::Return, 5);

    assert_eq!(chunk.code.len(), 9);
    vice::vm::compiler::optimize_chunk(&mut chunk);
    // All 4 redundant pairs removed! Only Return remains!
    assert_eq!(chunk.code.len(), 1);
    assert_eq!(chunk.code.last(), Some(&Opcode::Return));
}

#[test]
fn test_peephole_optimizer_fallthrough_jump_to_pop() {
    let mut chunk = vice::vm::compiler::Chunk::new();
    // JumpIfFalse targeting immediate next instruction (1)
    chunk.write(Opcode::JumpIfFalse(1), 1);
    chunk.write(Opcode::Return, 2);

    assert_eq!(chunk.code[0], Opcode::JumpIfFalse(1));
    vice::vm::compiler::optimize_chunk(&mut chunk);
    // JumpIfFalse(1) at index 0 should be simplified to Pop!
    assert_eq!(chunk.code[0], Opcode::Pop);
}
