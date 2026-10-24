use std::collections::HashMap;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{SystemTime, UNIX_EPOCH};
use vice::error::ViceErrorType;
use vice::value::Value;
use vice::vm::compiler::{Chunk, CompiledFunction, CompiledProgram, Compiler};
use vice::vm::machine::{Vm, VmConfig};
use vice::vm::opcodes::Opcode;

#[test]
fn closure_captures_stable_slot_by_reference() {
    let source = r#"
@entry
fn main() -> int {
    let mut base => 2;
    let add => |value: int| -> int => base + value;
    base = 10;
    output <- add(5);
    return add(7);
}
"#;
    let (value, output) = vice::run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["15"]);
    assert_eq!(value, Some(Value::Int(17)));
}

#[test]
fn references_mutate_variables_and_index_paths() {
    let source = r#"
@entry
fn main() {
    let mut values => [[1, 2], [3, 4]];
    let first => &values[0];
    *first = [9, 8];
    first.push(7);
    output <- values;
}
"#;
    let (_, output) = vice::run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["[[9, 8, 7], [3, 4]]"]);
}

#[test]
fn imported_impl_is_available_to_cli_pipeline() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("vice-imported-impl-{unique}"));
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("model.vice"),
        r#"
data Counter { value: int => 4; }
impl Counter {
    fn doubled(self) -> int { return self.value * 2; }
}
"#,
    )
    .unwrap();
    fs::write(
        directory.join("main.vice"),
        r#"
@mount "model.vice";
@entry
fn launch() { output <- Counter().doubled(); }
"#,
    )
    .unwrap();

    let program = vice::check_program(&directory.join("main.vice")).unwrap();
    assert_eq!(vice::execute_program_vm(&program).unwrap(), vec!["8"]);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn multiple_entry_points_are_compile_errors() {
    let program = vice::parse_source(
        r#"
@entry
fn one() {}
@entry
fn two() {}
"#,
    )
    .unwrap();
    let error = Compiler::new().compile(&program).unwrap_err();
    assert_eq!(error.error_type, ViceErrorType::Compile);
    assert!(error.message.contains("Multiple @entry"));
}

#[test]
fn runtime_error_contains_full_stack_trace() {
    let error = vice::run_source(
        r#"
fn deepest() { error() <- "boom"; }
fn middle() { deepest(); }
fn outer() { middle(); }
@entry
fn launch() { outer(); }
"#,
        vec![],
    )
    .unwrap_err();
    let functions = error
        .stack_trace
        .iter()
        .map(|frame| frame.function.as_str())
        .collect::<Vec<_>>();
    assert_eq!(functions, vec!["deepest", "middle", "outer", "launch"]);
}

#[test]
fn malformed_bytecode_is_rejected_without_panicking() {
    let mut chunk = Chunk::new();
    chunk.write(Opcode::LoadConst(99), 1);
    chunk.write(Opcode::Return, 1);
    let program = CompiledProgram {
        functions: HashMap::from([(
            "main".to_string(),
            CompiledFunction {
                name: "main".to_string(),
                chunk,
                arity: 0,
                local_count: 0,
                capture_count: 0,
                is_memoized: false,
                budget: None,
                is_nonreentrant: false,
                max_depth: None,
                is_checked: false,
                is_saturating: false,
                is_wrapping: false,
                is_leaf: false,
                is_unreachable: false,
                is_profiled: false,
                retry_limit: None,
                rate_limit: None,
                timeout_ms: None,
                is_audit: false,
            },
        )]),
        entry: "main".to_string(),
        sources: Vec::new(),
        data_declarations: HashMap::new(),
        singleton_types: std::collections::HashSet::new(),
    };
    let outcome = catch_unwind(AssertUnwindSafe(|| Vm::new().interpret(program)));
    assert!(outcome.is_ok());
    assert_eq!(
        outcome.unwrap().unwrap_err().error_type,
        ViceErrorType::Compile
    );
}

#[test]
fn instruction_budget_stops_infinite_programs() {
    let program = vice::parse_source(
        r#"
@entry
fn main() { while (true) {} }
"#,
    )
    .unwrap();
    let compiled = Compiler::new().compile(&program).unwrap();
    let mut vm = Vm::with_config(VmConfig {
        instruction_budget: Some(100),
        ..VmConfig::default()
    });
    let error = vm.interpret(compiled).unwrap_err();
    assert!(error.message.contains("Instruction budget"));
}
