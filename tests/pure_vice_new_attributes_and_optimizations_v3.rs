use vice::value::Value;
use vice::vm::compiler::{optimize_chunk, Chunk};
use vice::vm::opcodes::Opcode;

#[test]
fn test_packed_struct_validation() {
    // Valid packed struct with scalars
    let valid = r#"
@packed
data Point {
    x: float,
    y: float,
    active: bool,
    id: int,
}

@entry
fn main() -> int {
    let p => Point(x: 1.5, y: 2.5, active: true, id: 42);
    return p.id;
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Valid packed struct should compile");
    assert_eq!(val, Some(Value::Int(42)));

    // Invalid packed struct with string (non-scalar)
    let invalid = r#"
@packed
data Header {
    title: string,
    id: int,
}

@entry
fn main() {}
"#;
    let err = vice::run_source(invalid, vec![]).expect_err("Should reject non-scalar field in @packed");
    assert!(
        err.message.contains("in @packed struct 'Header' must have a scalar type"),
        "Unexpected error: {}",
        err.message
    );
}

#[test]
fn test_align_attribute_validation() {
    // Valid align(16)
    let valid = r#"
@align(16)
data CacheLine {
    tag: int,
}

@entry
fn main() -> int {
    let c => CacheLine(tag: 7);
    return c.tag;
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Valid align should compile");
    assert_eq!(val, Some(Value::Int(7)));

    // Invalid non-power-of-2 alignment
    let invalid = r#"
@align(15)
data Unaligned {
    x: int,
}

@entry
fn main() {}
"#;
    let err = vice::run_source(invalid, vec![]).expect_err("Should reject non-power-of-two alignment");
    assert!(
        err.message.contains("requires a power-of-two integer alignment"),
        "Unexpected error: {}",
        err.message
    );
}

#[test]
fn test_export_private_conflict() {
    let invalid_data = r#"
@export
@private
data Secret {
    value: int,
}

@entry
fn main() {}
"#;
    let err = vice::run_source(invalid_data, vec![]).expect_err("Should reject @export + @private on struct");
    assert!(
        err.message.contains("cannot be marked with both @export and @private"),
        "Unexpected error: {}",
        err.message
    );

    let invalid_fn = r#"
@export
@private
fn helper() {}

@entry
fn main() {}
"#;
    let err = vice::run_source(invalid_fn, vec![]).expect_err("Should reject @export + @private on function");
    assert!(
        err.message.contains("cannot be marked with both @export and @private"),
        "Unexpected error: {}",
        err.message
    );
}

#[test]
fn test_experimental_and_stability_attributes() {
    // Conflict between @experimental and @stable
    let conflict = r#"
@experimental
@stable
fn compute() -> int { return 1; }

@entry
fn main() {}
"#;
    let err = vice::run_source(conflict, vec![]).expect_err("Should reject @experimental + @stable");
    assert!(
        err.message.contains("cannot be marked with both @experimental and @stable"),
        "Unexpected error: {}",
        err.message
    );

    // Calling @experimental function from @safe function without @experimental
    let safe_caller_err = r#"
@safe
@experimental
fn beta_feature() -> int {
    return 100;
}

@safe
fn verified_system() -> int {
    return beta_feature();
}

@entry
fn main() {}
"#;
    let err = vice::run_source(safe_caller_err, vec![]).expect_err("Safe function cannot call @experimental");
    assert!(
        err.message.contains("Cannot call @experimental function 'beta_feature' from safe/strict function"),
        "Unexpected error: {}",
        err.message
    );

    // Valid experimental caller can call experimental function
    let valid_exp = r#"
@safe
@experimental
fn beta_feature() -> int {
    return 100;
}

@safe
@experimental
fn verified_exp() -> int {
    return beta_feature() + 5;
}

@entry
fn main() -> int {
    return verified_exp();
}
"#;
    let (val, _) = vice::run_source(valid_exp, vec![]).expect("Experimental safe caller should succeed");
    assert_eq!(val, Some(Value::Int(105)));
}

#[test]
fn test_benchmark_only_attribute() {
    // Calling @benchmark_only from regular non-benchmark function should error
    let invalid = r#"
@benchmark_only
fn measure_internal() -> int {
    return 42;
}

@entry
fn main() -> int {
    return measure_internal();
}
"#;
    let err = vice::run_source(invalid, vec![]).expect_err("Should reject @benchmark_only call from non-benchmark");
    assert!(
        err.message.contains("Cannot call @benchmark_only function 'measure_internal' from non-benchmark function"),
        "Unexpected error: {}",
        err.message
    );

    // Cannot mark function with both @benchmark_only and @entry
    let invalid_entry = r#"
@benchmark_only
@entry
fn main() {}
"#;
    let err = vice::run_source(invalid_entry, vec![]).expect_err("Cannot combine @benchmark_only and @entry");
    assert!(
        err.message.contains("cannot be marked with both @benchmark_only and @entry"),
        "Unexpected error: {}",
        err.message
    );
}

#[test]
fn test_must_mutate_attribute() {
    // Function with @must_mutate but no mutable parameter
    let no_mut_param = r#"
@must_mutate
fn bad_fn(x: int) -> int {
    return x + 1;
}

@entry
fn main() {}
"#;
    let err = vice::run_source(no_mut_param, vec![]).expect_err("Requires mutable parameter");
    assert!(
        err.message.contains("@must_mutate function 'bad_fn' must have at least one mutable parameter"),
        "Unexpected error: {}",
        err.message
    );

    // Function with mutable parameter that does NOT mutate it
    let unmutated = r#"
@must_mutate
fn neglected(mut x: int) -> int {
    return x + 1;
}

@entry
fn main() {}
"#;
    let err = vice::run_source(unmutated, vec![]).expect_err("Must mutate parameter");
    assert!(
        err.message.contains("@must_mutate function 'neglected' does not mutate any of its mutable parameters"),
        "Unexpected error: {}",
        err.message
    );

    // Valid @must_mutate
    let valid = r#"
@must_mutate
fn modify(mut x: int) -> int {
    x = x + 10;
    return x;
}

@entry
fn main() -> int {
    let mut val => 5;
    return modify(val);
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Valid @must_mutate should compile");
    assert_eq!(val, Some(Value::Int(15)));
}

#[test]
fn test_since_attribute() {
    let valid = r#"
@since("2.1.0")
fn modern_api() -> int {
    return 210;
}

@entry
fn main() -> int {
    return modern_api();
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Valid @since should compile");
    assert_eq!(val, Some(Value::Int(210)));

    let invalid = r#"
@since
fn missing_version() {}

@entry
fn main() {}
"#;
    let err = vice::run_source(invalid, vec![]).expect_err("@since requires version");
    assert!(
        err.message.contains("@since attribute on function 'missing_version' requires a version string or number"),
        "Unexpected error: {}",
        err.message
    );
}

#[test]
fn test_hash_module_fnv1a_and_djb2() {
    let source = r#"
@entry
fn main() -> int {
    let s => "Vice pure programming language";
    let h1 => hash.fnv1a(s);
    let h2 => hash.djb2(s);
    if h1 != 0 && h2 != 0 && h1 != h2 {
        return 1;
    }
    return 0;
}
"#;
    let (val, _) = vice::run_source(source, vec![]).expect("hash module should run cleanly");
    assert_eq!(val, Some(Value::Int(1)));
}

#[test]
fn test_string_trim_start_and_end() {
    let source = r#"
@entry
fn main() -> int {
    let padded => "   vice runtime   ";
    let left => padded.trim_start();
    let right => padded.trim_end();
    let left_mod => string.trim_start(padded);
    let right_mod => string.trim_end(padded);

    if left == "vice runtime   " && right == "   vice runtime" && left_mod == left && right_mod == right {
        return 1;
    }
    return 0;
}
"#;
    let (val, _) = vice::run_source(source, vec![]).expect("string trim_start/end should work");
    assert_eq!(val, Some(Value::Int(1)));
}

#[test]
fn test_math_hyperbolic_and_utilities() {
    let source = r#"
@entry
fn main() -> int {
    let c0 => math.cosh(0.0);
    let s0 => math.sinh(0.0);
    let t0 => math.tanh(0.0);
    let cs => math.copysign(42.0, -1.0);
    let d => math.dist(0.0, 0.0, 3.0, 4.0);

    if c0 == 1.0 && s0 == 0.0 && t0 == 0.0 && cs == -42.0 && d == 5.0 {
        return 1;
    }
    return 0;
}
"#;
    let (val, _) = vice::run_source(source, vec![]).expect("math hyperbolic and utilities should work");
    assert_eq!(val, Some(Value::Int(1)));
}

#[test]
fn test_compiler_optimizer_peephole_rotations_and_swap() {
    let mut chunk = Chunk::new();
    // Rotate3 x 3 should be eliminated as an identity operation
    chunk.write(Opcode::Rotate3, 1);
    chunk.write(Opcode::Rotate3, 1);
    chunk.write(Opcode::Rotate3, 1);
    chunk.write(Opcode::Return, 2);

    optimize_chunk(&mut chunk);
    // After 3x Rotate3 removal, only Opcode::Return should remain
    assert_eq!(chunk.code, vec![Opcode::Return]);

    // LoadConst(a), LoadConst(b), Swap => LoadConst(b), LoadConst(a)
    let mut chunk2 = Chunk::new();
    let idx_a = chunk2.add_constant(Value::Int(10));
    let idx_b = chunk2.add_constant(Value::Int(20));
    chunk2.write(Opcode::LoadConst(idx_a), 1);
    chunk2.write(Opcode::LoadConst(idx_b), 1);
    chunk2.write(Opcode::Swap, 1);
    chunk2.write(Opcode::Return, 2);

    optimize_chunk(&mut chunk2);
    // Swap should be eliminated, instructions reordered
    assert_eq!(chunk2.code, vec![Opcode::LoadConst(idx_b), Opcode::LoadConst(idx_a), Opcode::Return]);
}
