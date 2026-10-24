use vice::value::Value;
use vice::vm::compiler::{optimize_chunk, Chunk};
use vice::vm::opcodes::Opcode;

#[test]
fn test_transparent_struct_valid_and_scalar_nesting() {
    let valid = r#"
@transparent
data UserId {
    raw: int,
}

@packed
data PackedUserSlot {
    user: UserId,
    score: float,
}

@entry
fn main() -> int {
    let u => UserId(raw: 999);
    let slot => PackedUserSlot(user: u, score: 98.5);
    return slot.user.raw;
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Transparent struct in packed struct should compile");
    assert_eq!(val, Some(Value::Int(999)));
}

#[test]
fn test_transparent_struct_rejected_with_multiple_fields() {
    let invalid = r#"
@transparent
data Pair {
    first: int,
    second: int,
}

@entry
fn main() {}
"#;
    let err = vice::run_source(invalid, vec![]).expect_err("Should reject @transparent with 2 fields");
    assert!(
        err.message.contains("@transparent struct 'Pair' must have exactly one field, found 2"),
        "Unexpected error: {}",
        err.message
    );
}

#[test]
fn test_pure_data_mut_self_rejected() {
    let invalid = r#"
@pure_data
data Vector2D {
    x: float,
    y: float,
}

impl Vector2D {
    fn mutate_coords(mut self, dx: float, dy: float) {
        self.x = self.x + dx;
    }
}

@entry
fn main() {}
"#;
    let err = vice::run_source(invalid, vec![]).expect_err("Should reject mut self method on @pure_data struct");
    assert!(
        err.message.contains("Method 'mutate_coords' on @pure_data struct 'Vector2D' cannot take 'mut self'"),
        "Unexpected error: {}",
        err.message
    );

    let valid = r#"
@pure_data
data Vector2D {
    x: float,
    y: float,
}

impl Vector2D {
    fn length_sq(self) -> float {
        return self.x * self.x + self.y * self.y;
    }
}

@entry
fn main() -> int {
    let v => Vector2D(x: 3.0, y: 4.0);
    return to_int(v.length_sq());
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Valid immutable self method on @pure_data should compile");
    assert_eq!(val, Some(Value::Int(25)));
}

#[test]
fn test_security_critical_boundary_enforcement() {
    // Conflict with @experimental
    let conflict_exp = r#"
@security_critical
@experimental
fn dangerous_auth() {}

@entry
fn main() {}
"#;
    let err = vice::run_source(conflict_exp, vec![]).expect_err("Should reject @security_critical + @experimental");
    assert!(
        err.message.contains("cannot be marked with both @security_critical and @experimental"),
        "Unexpected error: {}",
        err.message
    );

    // Call from @experimental function rejected
    let call_from_exp = r#"
@security_critical
fn secret_kernel_call() -> int {
    return 1337;
}

@experimental
fn test_prototype() -> int {
    return secret_kernel_call();
}

@entry
fn main() {}
"#;
    let err = vice::run_source(call_from_exp, vec![]).expect_err("Experimental function cannot call @security_critical");
    assert!(
        err.message.contains("Cannot call @security_critical function 'secret_kernel_call' from @experimental function 'test_prototype'"),
        "Unexpected error: {}",
        err.message
    );

    // Valid call from regular function
    let valid_call = r#"
@security_critical
fn secret_kernel_call() -> int {
    return 1337;
}

fn vetted_wrapper() -> int {
    return secret_kernel_call() + 1;
}

@entry
fn main() -> int {
    return vetted_wrapper();
}
"#;
    let (val, _) = vice::run_source(valid_call, vec![]).expect("Vetted function can call @security_critical");
    assert_eq!(val, Some(Value::Int(1338)));
}

#[test]
fn test_inline_always_validation() {
    // Recursive @inline_always
    let recursive = r#"
@inline_always
fn rec(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * rec(n - 1);
}

@entry
fn main() {}
"#;
    let err = vice::run_source(recursive, vec![]).expect_err("Should reject recursive @inline_always");
    assert!(
        err.message.contains("cannot be marked with @inline_always because it is recursive"),
        "Unexpected error: {}",
        err.message
    );

    // Conflict with @no_inline
    let conflict = r#"
@inline_always
@no_inline
fn foo() {}

@entry
fn main() {}
"#;
    let err = vice::run_source(conflict, vec![]).expect_err("Should reject @inline_always + @no_inline");
    assert!(
        err.message.contains("cannot be marked with both @inline_always and @no_inline"),
        "Unexpected error: {}",
        err.message
    );

    // Conflict with @cold
    let conflict_cold = r#"
@inline_always
@cold
fn bar() {}

@entry
fn main() {}
"#;
    let err = vice::run_source(conflict_cold, vec![]).expect_err("Should reject @inline_always + @cold");
    assert!(
        err.message.contains("cannot be marked with both @inline_always and @cold"),
        "Unexpected error: {}",
        err.message
    );

    // Valid @inline_always
    let valid = r#"
@inline_always
fn fast_add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() -> int {
    return fast_add(40, 2);
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Valid @inline_always should compile");
    assert_eq!(val, Some(Value::Int(42)));
}

#[test]
fn test_reentrant_and_nonreentrant_conflict() {
    let conflict = r#"
@reentrant
@nonreentrant
fn worker() {}

@entry
fn main() {}
"#;
    let err = vice::run_source(conflict, vec![]).expect_err("Should reject @reentrant + @nonreentrant");
    assert!(
        err.message.contains("cannot be marked with both @reentrant and @nonreentrant"),
        "Unexpected error: {}",
        err.message
    );
}

#[test]
fn test_constant_time_validation() {
    let conflict_cold = r#"
@constant_time
@cold
fn crypto_cmp(a: int, b: int) -> bool {
    return a == b;
}

@entry
fn main() {}
"#;
    let err = vice::run_source(conflict_cold, vec![]).expect_err("Should reject @constant_time + @cold");
    assert!(
        err.message.contains("cannot be marked with both @constant_time and @cold"),
        "Unexpected error: {}",
        err.message
    );

    let conflict_timeout = r#"
@constant_time
@timeout(100)
fn crypto_cmp(a: int, b: int) -> bool {
    return a == b;
}

@entry
fn main() {}
"#;
    let err = vice::run_source(conflict_timeout, vec![]).expect_err("Should reject @constant_time + @timeout");
    assert!(
        err.message.contains("cannot be marked with both @constant_time and @timeout"),
        "Unexpected error: {}",
        err.message
    );

    let valid = r#"
@constant_time
fn secure_equal(a: int, b: int) -> int {
    if a == b {
        return 1;
    }
    return 0;
}

@entry
fn main() -> int {
    return secure_equal(100, 100);
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Valid @constant_time should compile");
    assert_eq!(val, Some(Value::Int(1)));
}

#[test]
fn test_no_discard_attribute_support() {
    let invalid = r#"
@no_discard
fn do_nothing() {}

@entry
fn main() {}
"#;
    let err = vice::run_source(invalid, vec![]).expect_err("@no_discard on void function should fail");
    assert!(
        err.message.contains("must have a return value"),
        "Unexpected error: {}",
        err.message
    );

    let valid = r#"
@no_discard
fn compute_answer() -> int {
    return 42;
}

@entry
fn main() -> int {
    let res => compute_answer();
    return res;
}
"#;
    let (val, _) = vice::run_source(valid, vec![]).expect("Valid @no_discard should run");
    assert_eq!(val, Some(Value::Int(42)));
}

#[test]
fn test_stdlib_adler32_crc32_and_combine() {
    let source = r#"
@entry
fn main() -> int {
    let s => "Vice Pure Language 2026";
    let adler1 => hash.adler32(s);
    let adler2 => s.adler32();
    let crc1 => hash.crc32(s);
    let crc2 => s.crc32();
    let fnv1 => hash.fnv1a(s);
    let fnv2 => s.fnv1a();
    let djb1 => hash.djb2(s);
    let djb2 => s.djb2();
    let combined => hash.combine(adler1, crc1);

    if adler1 != 0 && adler1 == adler2 && crc1 != 0 && crc1 == crc2 && fnv1 == fnv2 && djb1 == djb2 && combined != 0 {
        return 1;
    }
    return 0;
}
"#;
    let (val, _) = vice::run_source(source, vec![]).expect("Hash methods should run cleanly");
    assert_eq!(val, Some(Value::Int(1)));
}

#[test]
fn test_stdlib_math_radians_degrees() {
    let source = r#"
@entry
fn main() -> int {
    let rad180 => math.radians(180.0);
    let deg => math.degrees(rad180);
    // Rounding to int should be exactly 180
    if deg >= 179.9 && deg <= 180.1 {
        return 1;
    }
    return 0;
}
"#;
    let (val, _) = vice::run_source(source, vec![]).expect("math.radians and math.degrees should work");
    assert_eq!(val, Some(Value::Int(1)));
}

#[test]
fn test_stdlib_array_average() {
    let source = r#"
@entry
fn main() -> int {
    let arr => [10, 20, 30, 40];
    let avg_mod => array.average(arr);
    let avg_rec => arr.average();

    if avg_mod == 25.0 && avg_rec == 25.0 {
        return 1;
    }
    return 0;
}
"#;
    let (val, _) = vice::run_source(source, vec![]).expect("array.average should work");
    assert_eq!(val, Some(Value::Int(1)));
}

#[test]
fn test_compiler_peephole_dup_swap_and_identities() {
    // Dup followed by Swap is a no-op swap: [Dup, Swap] => [Dup]
    let mut chunk = Chunk::new();
    chunk.write(Opcode::Dup, 1);
    chunk.write(Opcode::Swap, 1);
    chunk.write(Opcode::Return, 2);

    optimize_chunk(&mut chunk);
    assert_eq!(chunk.code, vec![Opcode::Dup, Opcode::Return]);

    // LoadConst(true), And => eliminated identity
    let mut chunk2 = Chunk::new();
    let c_true = chunk2.add_constant(Value::Bool(true));
    chunk2.write(Opcode::LoadConst(c_true), 1);
    chunk2.write(Opcode::And, 1);
    chunk2.write(Opcode::Return, 2);

    optimize_chunk(&mut chunk2);
    assert_eq!(chunk2.code, vec![Opcode::Return]);

    // LoadConst(false), Or => eliminated identity
    let mut chunk3 = Chunk::new();
    let c_false = chunk3.add_constant(Value::Bool(false));
    chunk3.write(Opcode::LoadConst(c_false), 1);
    chunk3.write(Opcode::Or, 1);
    chunk3.write(Opcode::Return, 2);

    optimize_chunk(&mut chunk3);
    assert_eq!(chunk3.code, vec![Opcode::Return]);

    // LoadConst(1), Pow => eliminated identity
    let mut chunk4 = Chunk::new();
    let c_one = chunk4.add_constant(Value::Int(1));
    chunk4.write(Opcode::LoadConst(c_one), 1);
    chunk4.write(Opcode::Pow, 1);
    chunk4.write(Opcode::Return, 2);

    optimize_chunk(&mut chunk4);
    assert_eq!(chunk4.code, vec![Opcode::Return]);
}
