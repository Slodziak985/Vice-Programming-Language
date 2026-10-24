use vice::run_source;

#[test]
fn test_attribute_test_only_called_from_test_succeeds() {
    let source = r#"
@test_only
fn mock_data() -> int {
    return 42;
}

@test
fn verify_mock() {
    let x => mock_data();
    if x != 42 {
        fail <- "Mismatch";
    }
}

@entry
fn main() {}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_ok(), "Expected success: {:?}", res.err());
}

#[test]
fn test_attribute_test_only_called_from_normal_fails() {
    let source = r#"
@test_only
fn mock_data() -> int {
    return 42;
}

@entry
fn main() {
    let x => mock_data();
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("Cannot call @test_only function 'mock_data' from non-test function 'main'"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_test_only_cannot_be_entry() {
    let source = r#"
@test_only
@entry
fn main() {
    output <- "hi";
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("cannot be marked with both @test_only and @entry"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_no_recurse_prevents_recursion() {
    let source = r#"
@no_recurse
fn countdown(n: int) -> int {
    if n <= 0 {
        return 0;
    }
    return countdown(n - 1);
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("Function 'countdown' is marked @no_recurse and cannot call itself"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_no_recurse_cannot_be_tailrec() {
    let source = r#"
@no_recurse
@tailrec
fn loop_fn(n: int) -> int {
    return n;
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("cannot be marked with both @no_recurse and @tailrec"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_no_recurse_valid_function() {
    let source = r#"
@no_recurse
fn calculate(a: int, b: int) -> int {
    return a * b + 10;
}

@entry
fn main() {
    let res => calculate(5, 6);
    output <- res;
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_ok(), "Expected success: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["40"]);
}

#[test]
fn test_attribute_obsolete_function_rejected() {
    let source = r#"
@obsolete("Use modern_api() instead")
fn legacy_calc() -> int {
    return 1;
}

@entry
fn main() {
    let x => legacy_calc();
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("Item 'legacy_calc' is marked @obsolete and cannot be used: Use modern_api() instead"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_obsolete_data_rejected() {
    let source = r#"
@obsolete("Use NewConfig")
data OldConfig {
    port: int => 8080;
}

@entry
fn main() {
    let cfg => OldConfig();
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("Item 'OldConfig' is marked @obsolete and cannot be used: Use NewConfig"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_obsolete_enum_rejected() {
    let source = r#"
@obsolete("Use ModernState")
enum OldState {
    Active,
    Inactive
}

@entry
fn main() {
    let s => OldState.Active;
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("Item 'OldState' is marked @obsolete and cannot be used: Use ModernState"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_sealed_data_same_file_succeeds() {
    let source = r#"
@sealed
data LocalRecord {
    val: int;
}

impl LocalRecord {
    fn double_val(self) -> int {
        return self.val * 2;
    }
}

@entry
fn main() {
    let r => LocalRecord(val: 21);
    output <- r.double_val();
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_ok(), "Expected success: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_attribute_sealed_data_cross_file_rejected() {
    use std::fs;
    let dir = std::env::temp_dir().join("vice_test_sealed_data");
    let _ = fs::create_dir_all(&dir);
    let def_file = dir.join("def.vice");
    let impl_file = dir.join("main.vice");

    fs::write(&def_file, r#"
@sealed
data EngineCore {
    speed: int => 100;
}
"#).unwrap();

    fs::write(&impl_file, r#"
use "def.vice";

impl EngineCore {
    fn boost(self) -> int {
        return self.speed * 2;
    }
}

@entry
fn main() {
    let e => EngineCore();
    output <- e.boost();
}
"#).unwrap();

    let res = vice::load_program(&impl_file).and_then(|p| {
        let mut sem = vice::semantic::SemanticAnalyzer::new();
        sem.analyze(&p)
    });

    let _ = fs::remove_dir_all(&dir);

    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("Data type 'EngineCore' is @sealed and cannot be implemented outside its defining file"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_constant_folding_math_advanced() {
    let source = r#"
@entry
fn main() {
    let p => math.pow(2, 10);
    let s => math.sin(0.0);
    let c => math.cos(0.0);
    let l => math.log2(1024.0);
    output <- p, s, c, l;
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_ok(), "Expected success: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["1024 0 1 10"]);
}

#[test]
fn test_constant_folding_string_advanced() {
    let source = r#"
@entry
fn main() {
    let sliced => string.slice("ViceLang", 0, 4);
    let replaced => string.replace("foo-bar", "-", "_");
    output <- sliced, replaced;
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_ok(), "Expected success: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["Vice foo_bar"]);
}

#[test]
fn test_constant_folding_array_advanced() {
    let source = r#"
@entry
fn main() {
    let repeated => array.fill(3, 7);
    let sliced => array.slice([10, 20, 30, 40], 1, 3);
    output <- repeated, sliced;
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_ok(), "Expected success: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["[7, 7, 7] [20, 30]"]);
}

#[test]
fn test_peephole_comparator_inversion() {
    let source = r#"
@entry
fn main() {
    let a => 10;
    let b => 20;
    let not_lt => !(a < b); // Should fold [Lt, Not] => [Gte]
    let not_eq => !(a == b); // Should fold [Eq, Not] => [Neq]
    let not_gt => !(a > b); // Should fold [Gt, Not] => [Lte]
    output <- not_lt, not_eq, not_gt;
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_ok(), "Expected success: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["false true true"]);
}

#[test]
fn test_attribute_non_exhaustive_enum_requires_wildcard() {
    let source = r#"
@non_exhaustive
enum Status {
    Active,
    Suspended
}

@entry
fn main() {
    let s => Status.Active;
    let label => match s {
        Status.Active => "ok",
        Status.Suspended => "paused"
    };
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("Matching @non_exhaustive enum 'Status' must include a wildcard '_' arm"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_non_exhaustive_enum_with_wildcard_succeeds() {
    let source = r#"
@non_exhaustive
enum Status {
    Active,
    Suspended
}

@entry
fn main() {
    let s => Status.Active;
    let label => match s {
        Status.Active => "ok",
        _ => "other"
    };
    output <- label;
}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_ok(), "Expected success: {:?}", res.err());
    let (_, output) = res.unwrap();
    assert_eq!(output, vec!["ok"]);
}

#[test]
fn test_attribute_cold_and_benchmark_conflict() {
    let source = r#"
@cold
@benchmark
fn bench_me() {}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("cannot be marked with both @cold and @benchmark"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_attribute_hot_and_unreachable_conflict() {
    let source = r#"
@hot
@unreachable
fn never_run() {}
"#;
    let res = run_source(source, vec![]);
    assert!(res.is_err());
    let err = format!("{}", res.err().unwrap());
    assert!(
        err.contains("cannot be marked with both @hot and @unreachable"),
        "Unexpected error: {}",
        err
    );
}
