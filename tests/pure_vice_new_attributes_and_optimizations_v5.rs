use vice::value::Value;
use vice::vm::compiler::Compiler;
use vice::vm::opcodes::Opcode;

#[test]
fn test_must_initialize_missing_fields_rejected() {
    // Named arguments missing field
    let src_named = r#"
@must_initialize
data ServerConfig {
    host: string => "127.0.0.1",
    port: int => 8080,
    tls: bool => false,
}

@entry
fn main() {
    let cfg => ServerConfig(host: "0.0.0.0", port: 9000);
}
"#;
    let err = vice::run_source(src_named, vec![]).expect_err("Should reject missing field in @must_initialize struct");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("missing field 'tls'") || msg.contains("requires all 3 fields"),
        "Unexpected error message: {}",
        msg
    );

    // Positional arguments missing field
    let src_pos = r#"
@must_init
data Coordinate {
    x: int => 0,
    y: int => 0,
    z: int => 0,
}

@entry
fn main() {
    let c => Coordinate(10, 20);
}
"#;
    let err_pos = vice::run_source(src_pos, vec![]).expect_err("Should reject missing positional fields in @must_init struct");
    let msg_pos = format!("{:?}", err_pos);
    assert!(
        msg_pos.contains("requires all 3 fields") || msg_pos.contains("missing field"),
        "Unexpected error message: {}",
        msg_pos
    );
}

#[test]
fn test_must_initialize_valid() {
    let src = r#"
@must_initialize
data Point {
    x: int => 0,
    y: int => 0,
}

@entry
fn main() -> int {
    let p => Point(x: 15, y: 25);
    return p.x + p.y;
}
"#;
    let (val, _) = vice::run_source(src, vec![]).expect("Valid @must_initialize instantiation should compile and run");
    assert_eq!(val, Some(Value::Int(40)));
}

#[test]
fn test_simd_recursion_rejected() {
    let src = r#"
@simd
fn dot_recurse(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * dot_recurse(n - 1);
}

@entry
fn main() {}
"#;
    let err = vice::run_source(src, vec![]).expect_err("Should reject recursive @simd function");
    assert!(
        format!("{:?}", err).contains("cannot be recursive"),
        "Expected recursion rejection, got {:?}",
        err
    );
}

#[test]
fn test_simd_conflicts_rejected() {
    // @simd + @cold
    let src_cold = r#"
@simd
@cold
fn compute() -> int {
    return 42;
}

@entry
fn main() {}
"#;
    let err_cold = vice::run_source(src_cold, vec![]).expect_err("Should reject @simd + @cold");
    assert!(
        format!("{:?}", err_cold).contains("cannot be marked with both @simd and @cold"),
        "Got: {:?}",
        err_cold
    );

    // @simd + @no_inline
    let src_no_inline = r#"
@simd
@no_inline
fn compute() -> int {
    return 42;
}

@entry
fn main() {}
"#;
    let err_no_inline = vice::run_source(src_no_inline, vec![]).expect_err("Should reject @simd + @no_inline");
    assert!(
        format!("{:?}", err_no_inline).contains("cannot be marked with both @simd and @no_inline"),
        "Got: {:?}",
        err_no_inline
    );
}

#[test]
fn test_simd_io_rejected() {
    let src_emit = r#"
@simd
fn compute_and_log(x: int) -> int {
    emit <- "Processing:", x;
    return x * 2;
}

@entry
fn main() {}
"#;
    let err_emit = vice::run_source(src_emit, vec![]).expect_err("Should reject emit inside @simd function");
    assert!(
        format!("{:?}", err_emit).contains("Cannot perform I/O (output/emit) inside @simd function"),
        "Got: {:?}",
        err_emit
    );

    let src_io_call = r#"
@simd
fn compute_and_print(x: int) -> int {
    print("x:", x);
    return x * 2;
}

@entry
fn main() {}
"#;
    let err_call = vice::run_source(src_io_call, vec![]).expect_err("Should reject print inside @simd function");
    assert!(
        format!("{:?}", err_call).contains("Cannot call I/O function 'print' inside @simd function"),
        "Got: {:?}",
        err_call
    );
}

#[test]
fn test_simd_valid_vector_computation() {
    let src = r#"
@simd
fn dot3(a: int[], b: int[]) -> int {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

@entry
fn main() -> int {
    let v1 => [2, 3, 4];
    let v2 => [5, 6, 7];
    return dot3(v1, v2);
}
"#;
    let (val, _) = vice::run_source(src, vec![]).expect("Valid @simd function should execute cleanly");
    // 2*5 + 3*6 + 4*7 = 10 + 18 + 28 = 56
    assert_eq!(val, Some(Value::Int(56)));
}

#[test]
fn test_idempotent_conflicts_and_global_mutation_rejected() {
    // Conflict with @cold
    let src_cold = r#"
@idempotent
@cold
fn fetch(x: int) -> int {
    return x * 2;
}

@entry
fn main() {}
"#;
    let err_cold = vice::run_source(src_cold, vec![]).expect_err("Should reject @idempotent + @cold");
    assert!(
        format!("{:?}", err_cold).contains("cannot be marked with both @idempotent and @cold"),
        "Got: {:?}",
        err_cold
    );

    // Global mutation
    let src_global = r#"
@idempotent
fn impure_increment(delta: int) -> int {
    g_counter = g_counter + delta;
    return delta;
}

@entry
fn main() {}
"#;
    let err_global = vice::run_source(src_global, vec![]).expect_err("Should reject global mutation inside @idempotent function");
    assert!(
        format!("{:?}", err_global).contains("Cannot mutate global variable 'g_counter' inside @idempotent function"),
        "Got: {:?}",
        err_global
    );
}

#[test]
fn test_idempotent_valid_computation() {
    let src = r#"
@idempotent
fn normalize_hash(val: int, mask: int) -> int {
    return val % mask;
}

@entry
fn main() -> int {
    return normalize_hash(12345, 100);
}
"#;
    let (val, _) = vice::run_source(src, vec![]).expect("Valid @idempotent function should execute cleanly");
    assert_eq!(val, Some(Value::Int(45)));
}

#[test]
fn test_murmur3_hash_algorithm() {
    let src = r#"
@entry
fn main() -> bool {
    let s => "Vice compiler engine 2026";
    let h1 => hash.murmur3(s);
    let h2 => s.murmur3();

    let diff => "Vice compiler engine 2027".murmur3();

    return h1 == h2 && h1 != 0 && h1 != diff;
}
"#;
    let (val, _) = vice::run_source(src, vec![]).expect("Murmur3 hash should compile and execute");
    assert_eq!(val, Some(Value::Bool(true)));
}

#[test]
fn test_array_product_and_dedup() {
    let src = r#"
@entry
fn main() -> bool {
    let nums => [2, 3, 4, 5];
    let prod_mod => array.product(nums);
    let prod_rec => nums.product();

    let floats => [1.5, 2.0, 4.0];
    let prod_flt => floats.product();

    let dupes => [1, 1, 2, 2, 2, 3, 1, 1];
    let deduped_mod => array.dedup(dupes);
    let deduped_rec => dupes.dedup();

    return prod_mod == 120
        && prod_rec == 120
        && prod_flt == 12.0
        && deduped_mod.len() == 4
        && deduped_rec[0] == 1
        && deduped_rec[1] == 2
        && deduped_rec[2] == 3
        && deduped_rec[3] == 1;
}
"#;
    let (val, _) = vice::run_source(src, vec![]).expect("array.product and array.dedup should execute cleanly");
    assert_eq!(val, Some(Value::Bool(true)));
}

#[test]
fn test_compiler_peephole_len_and_empty_collection_folding() {
    let src = r#"
@entry
fn main() -> int {
    let s_len => len("Antigravity");
    let a_len => len([]);
    return s_len + a_len;
}
"#;
    let program = vice::parse_source(src).expect("Parse program");
    let compiled = Compiler::new().compile(&program).expect("Compile program");
    let main_chunk = &compiled.functions.get("main").expect("main exists").chunk;

    for (idx, op) in main_chunk.code.iter().enumerate() {
        println!("{}: {:?}", idx, op);
    }
    let has_len = main_chunk.code.iter().any(|op| matches!(op, Opcode::Len));
    assert!(!has_len, "Opcode::Len should be folded into constant by peephole pass");

    let (val, _) = vice::run_source(src, vec![]).expect("Program should execute cleanly");
    // "Antigravity".len() is 11, [].len() is 0 => total 11
    assert_eq!(val, Some(Value::Int(11)));
}
