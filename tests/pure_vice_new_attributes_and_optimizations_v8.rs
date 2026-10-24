use vice::run_source;
use vice::value::Value;

#[test]
fn test_pure_array_attribute_validation() {
    // 1. Valid @pure_array function
    let src_valid = r#"
@pure_array
fn filter_and_compact(items: int[], exclude: int[]) -> int[] {
    let diffed => items.diff(exclude);
    return diffed;
}

@entry
fn main() -> int {
    let a => [10, 20, 30, 40];
    let b => [20, 40];
    let res => filter_and_compact(a, b);
    if res.len() == 2 && res[0] == 10 && res[1] == 30 {
        return 1;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @pure_array should succeed");
    assert_eq!(res, Some(Value::Int(1)));

    // 2. Reject @entry on @pure_array
    let src_entry = r#"
@pure_array
@entry
fn main() -> int[] {
    return [1, 2, 3];
}
"#;
    let err_entry = run_source(src_entry, vec![]).expect_err("Should reject @entry on @pure_array");
    assert!(
        format!("{:?}", err_entry).contains("@pure_array function 'main' cannot be marked with @entry"),
        "Got: {:?}",
        err_entry
    );

    // 3. Reject non-array parameters and return type
    let src_no_arr = r#"
@pure_array
fn add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {}
"#;
    let err_no_arr = run_source(src_no_arr, vec![]).expect_err("Should reject non-array signature");
    assert!(
        format!("{:?}", err_no_arr).contains("@pure_array function 'add' requires at least one array parameter or array return type"),
        "Got: {:?}",
        err_no_arr
    );

    // 4. Reject calling impure module (e.g. fs)
    let src_fs = r#"
@pure_array
fn write_log(items: string[]) -> string[] {
    fs.write_text("log.txt", "data");
    return items;
}

@entry
fn main() {}
"#;
    let err_fs = run_source(src_fs, vec![]).expect_err("Should reject calling fs inside @pure_array");
    assert!(
        format!("{:?}", err_fs).contains("Cannot call non-array module 'fs.write_text' inside @pure_array function"),
        "Got: {:?}",
        err_fs
    );

    // 5. Reject mutating global variable
    let src_global = r#"
@pure_array
fn count_items(items: int[]) -> int[] {
    g_counter = g_counter + 1;
    return items;
}

@entry
fn main() {}
"#;
    let err_global = run_source(src_global, vec![]).expect_err("Should reject mutating global inside @pure_array");
    assert!(
        format!("{:?}", err_global).contains("Cannot mutate global variable 'g_counter' inside @pure_array function"),
        "Got: {:?}",
        err_global
    );
}

#[test]
fn test_ascii_only_attribute_validation() {
    // 1. Valid execution with ASCII strings
    let src_valid = r#"
@ascii_only
fn sanitize_tag(tag: string) -> string {
    return tag.lower();
}

@entry
fn main() -> int {
    let t => sanitize_tag("alpha_numeric_tag");
    if t == "alpha_numeric_tag" {
        return 1;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @ascii_only should succeed");
    assert_eq!(res, Some(Value::Int(1)));

    // 2. Reject function without string parameter
    let src_no_str = r#"
@ascii_only
fn calc_tag(id: int) -> int {
    return id * 2;
}

@entry
fn main() {}
"#;
    let err_no_str = run_source(src_no_str, vec![]).expect_err("Should reject @ascii_only without string param");
    assert!(
        format!("{:?}", err_no_str).contains("@ascii_only function 'calc_tag' requires at least one string parameter"),
        "Got: {:?}",
        err_no_str
    );

    // 3. Statically reject non-ASCII constant argument
    let src_non_ascii = r#"
@ascii_only
fn sanitize(s: string) -> string {
    return s;
}

@entry
fn main() {
    sanitize("Zażółć gęślą jaźń");
}
"#;
    let err_non_ascii = run_source(src_non_ascii, vec![]).expect_err("Should reject non-ASCII constant string");
    assert!(
        format!("{:?}", err_non_ascii).contains("Constant argument to @ascii_only function 'sanitize' at parameter 1 contains non-ASCII characters"),
        "Got: {:?}",
        err_non_ascii
    );
}

#[test]
fn test_sorted_attribute_validation() {
    // 1. Valid execution with sorted array & sorted arguments
    let src_valid = r#"
@sorted
fn binary_lookup(sorted_list: int[], target: int) -> int {
    return sorted_list.binary_search(target);
}

@sorted
fn range_span(min: int, max: int) -> int {
    return max - min;
}

@entry
fn main() -> int {
    let idx => binary_lookup([10, 20, 30, 40], 30);
    let span => range_span(10, 50);
    if idx == 2 && span == 40 {
        return 1;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @sorted should succeed");
    assert_eq!(res, Some(Value::Int(1)));

    // 2. Reject function without array or multiple numeric parameters
    let src_no_sup = r#"
@sorted
fn single_num(val: int) -> int {
    return val;
}

@entry
fn main() {}
"#;
    let err_no_sup = run_source(src_no_sup, vec![]).expect_err("Should reject @sorted on single int");
    assert!(
        format!("{:?}", err_no_sup).contains("@sorted function 'single_num' requires at least one array parameter or at least two numeric parameters"),
        "Got: {:?}",
        err_no_sup
    );

    // 3. Statically reject unsorted constant array
    let src_unsorted_arr = r#"
@sorted
fn process(items: int[]) -> int {
    return items.len();
}

@entry
fn main() {
    process([10, 50, 30, 40]);
}
"#;
    let err_unsorted_arr = run_source(src_unsorted_arr, vec![]).expect_err("Should reject unsorted array literal");
    assert!(
        format!("{:?}", err_unsorted_arr).contains("Constant array argument to @sorted function 'process' at parameter 1 is not sorted"),
        "Got: {:?}",
        err_unsorted_arr
    );

    // 4. Statically reject unsorted multiple constant numeric arguments
    let src_unsorted_args = r#"
@sorted
fn check_bounds(low: int, high: int) -> int {
    return high - low;
}

@entry
fn main() {
    check_bounds(100, 20);
}
"#;
    let err_unsorted_args = run_source(src_unsorted_args, vec![]).expect_err("Should reject unsorted arguments");
    assert!(
        format!("{:?}", err_unsorted_args).contains("Constant arguments to @sorted function 'check_bounds' are not in sorted order"),
        "Got: {:?}",
        err_unsorted_args
    );
}

#[test]
fn test_distinct_attribute_validation() {
    // 1. Valid execution with distinct array and arguments
    let src_valid = r#"
@distinct
fn unique_set(items: int[]) -> int {
    return items.len();
}

@distinct
fn compare_pair(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() -> int {
    let l => unique_set([1, 2, 3, 4, 5]);
    let c => compare_pair(10, 20);
    if l == 5 && c == 30 {
        return 1;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @distinct should succeed");
    assert_eq!(res, Some(Value::Int(1)));

    // 2. Reject function with fewer than 2 parameters and no array
    let src_single = r#"
@distinct
fn test_single(x: int) -> int {
    return x;
}

@entry
fn main() {}
"#;
    let err_single = run_source(src_single, vec![]).expect_err("Should reject @distinct with single non-array param");
    assert!(
        format!("{:?}", err_single).contains("@distinct function 'test_single' requires at least one array parameter or at least two parameters"),
        "Got: {:?}",
        err_single
    );

    // 3. Statically reject constant array with duplicates
    let src_dup_arr = r#"
@distinct
fn process(items: int[]) -> int {
    return items.len();
}

@entry
fn main() {
    process([1, 2, 3, 2, 5]);
}
"#;
    let err_dup_arr = run_source(src_dup_arr, vec![]).expect_err("Should reject array with duplicates");
    assert!(
        format!("{:?}", err_dup_arr).contains("Constant array argument to @distinct function 'process' at parameter 1 contains duplicate elements"),
        "Got: {:?}",
        err_dup_arr
    );

    // 4. Statically reject duplicate constant arguments
    let src_dup_args = r#"
@distinct
fn pair(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    pair(42, 42);
}
"#;
    let err_dup_args = run_source(src_dup_args, vec![]).expect_err("Should reject duplicate arguments");
    assert!(
        format!("{:?}", err_dup_args).contains("Call to @distinct function 'pair' contains duplicate constant arguments"),
        "Got: {:?}",
        err_dup_args
    );
}

#[test]
fn test_positive_attribute_validation() {
    // 1. Valid execution with positive constant
    let src_valid = r#"
@positive
fn allocate_buffer(size: int) -> int {
    return size * 1024;
}

@entry
fn main() -> int {
    return allocate_buffer(16);
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @positive should succeed");
    assert_eq!(res, Some(Value::Int(16384)));

    // 2. Reject function without numeric parameter
    let src_no_num = r#"
@positive
fn log_message(msg: string) -> string {
    return msg;
}

@entry
fn main() {}
"#;
    let err_no_num = run_source(src_no_num, vec![]).expect_err("Should reject @positive without numeric param");
    assert!(
        format!("{:?}", err_no_num).contains("@positive function 'log_message' requires at least one numeric parameter"),
        "Got: {:?}",
        err_no_num
    );

    // 3. Statically reject 0 or negative arguments
    let src_zero = r#"
@positive
fn alloc(bytes: int) -> int {
    return bytes;
}

@entry
fn main() {
    alloc(0);
}
"#;
    let err_zero = run_source(src_zero, vec![]).expect_err("Should reject 0 in @positive");
    assert!(
        format!("{:?}", err_zero).contains("Constant argument to @positive function 'alloc' at parameter 1 must be positive (> 0)"),
        "Got: {:?}",
        err_zero
    );

    let src_neg = r#"
@positive
fn alloc(bytes: int) -> int {
    return bytes;
}

@entry
fn main() {
    alloc(-10);
}
"#;
    let err_neg = run_source(src_neg, vec![]).expect_err("Should reject negative in @positive");
    assert!(
        format!("{:?}", err_neg).contains("Constant argument to @positive function 'alloc' at parameter 1 must be positive (> 0)"),
        "Got: {:?}",
        err_neg
    );
}

#[test]
fn test_bounded_attribute_validation() {
    // 1. Valid execution within bounds
    let src_valid = r#"
@bounded(0, 100)
fn set_percentage(pct: int) -> int {
    return pct;
}

@entry
fn main() -> int {
    return set_percentage(75);
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @bounded should succeed");
    assert_eq!(res, Some(Value::Int(75)));

    // 2. Reject min > max
    let src_inv = r#"
@bounded(100, 50)
fn invalid_fn(x: int) -> int {
    return x;
}

@entry
fn main() {}
"#;
    let err_inv = run_source(src_inv, vec![]).expect_err("Should reject min > max");
    assert!(
        format!("{:?}", err_inv).contains("@bounded: min (100) cannot be greater than max (50)"),
        "Got: {:?}",
        err_inv
    );

    // 3. Statically reject out of bounds argument
    let src_out_high = r#"
@bounded(0, 100)
fn set_pct(pct: int) -> int {
    return pct;
}

@entry
fn main() {
    set_pct(150);
}
"#;
    let err_out_high = run_source(src_out_high, vec![]).expect_err("Should reject argument > max");
    assert!(
        format!("{:?}", err_out_high).contains("is out of bounds [0, 100]"),
        "Got: {:?}",
        err_out_high
    );

    let src_out_low = r#"
@bounded(10, 50)
fn set_range(val: int) -> int {
    return val;
}

@entry
fn main() {
    set_range(5);
}
"#;
    let err_out_low = run_source(src_out_low, vec![]).expect_err("Should reject argument < min");
    assert!(
        format!("{:?}", err_out_low).contains("is out of bounds [10, 50]"),
        "Got: {:?}",
        err_out_low
    );
}

#[test]
fn test_idempotent_attribute_validation() {
    // 1. Valid @idempotent function
    let src_valid = r#"
@idempotent
fn normalize_name(name: string) -> string {
    return name.trim();
}

@entry
fn main() -> int {
    let once => normalize_name("  vice  ");
    let twice => normalize_name(once);
    if once == "vice" && twice == "vice" {
        return 1;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src_valid, vec![]).expect("Valid @idempotent should succeed");
    assert_eq!(res, Some(Value::Int(1)));

    // 2. Reject when return type does not match parameter type
    let src_mismatch = r#"
@idempotent
fn bad_idempotent(val: int) -> string {
    return "num";
}

@entry
fn main() {}
"#;
    let err_mismatch = run_source(src_mismatch, vec![]).expect_err("Should reject mismatched return type on @idempotent");
    assert!(
        format!("{:?}", err_mismatch).contains("@idempotent function 'bad_idempotent' return type"),
        "Got: {:?}",
        err_mismatch
    );

    // 3. Reject mutating global variable inside @idempotent
    let src_global = r#"
@idempotent
fn mut_global(x: int) -> int {
    g_val = x;
    return x;
}

@entry
fn main() {}
"#;
    let err_global = run_source(src_global, vec![]).expect_err("Should reject mutating global inside @idempotent");
    assert!(
        format!("{:?}", err_global).contains("Cannot mutate global variable 'g_val' inside @idempotent function"),
        "Got: {:?}",
        err_global
    );

    // 4. Reject calling impure module inside @idempotent
    let src_impure = r#"
@idempotent
fn log_idempotent(msg: string) -> string {
    fs.write_text("log.txt", msg);
    return msg;
}

@entry
fn main() {}
"#;
    let err_impure = run_source(src_impure, vec![]).expect_err("Should reject calling fs inside @idempotent");
    assert!(
        format!("{:?}", err_impure).contains("Cannot call impure module method 'fs.write_text' inside @idempotent function"),
        "Got: {:?}",
        err_impure
    );
}

#[test]
fn test_algorithms_runtime_v8() {
    let src = r#"
fn empty_item() {}

@entry
fn main() -> int {
    // 1. math.is_square
    let s1 => math.is_square(16);
    let s2 => math.is_square(25);
    let s3 => math.is_square(20);
    let s4 => math.is_square(0);
    let s5 => math.is_square(1);
    let s6 => math.is_square(-4);

    // 2. string.lines_count & words_count
    let text => "line 1\nline 2\nline 3";
    let lc1 => text.lines_count();
    let lc2 => string.lines_count("hello");
    let lc3 => string.lines_count("");

    let sentence => "  pure vice language runtime 2026  ";
    let wc1 => sentence.words_count();
    let wc2 => string.words_count("one");
    let wc3 => string.words_count("");

    // 3. array.compact
    let arr_with_unit => [1, empty_item(), 2, empty_item(), 3];
    let compacted => arr_with_unit.compact();

    // 4. array.diff
    let a => [1, 2, 3, 4, 5];
    let b => [2, 4, 6];
    let diff_ab => a.diff(b);
    let diff_mod => array.diff([10, 20, 30], [20]);

    // 5. array.intersect
    let inter_ab => a.intersect(b);
    let inter_mod => array.intersect([10, 20, 30], [20, 30, 40]);

    if s1 && s2 && !s3 && s4 && s5 && !s6
        && lc1 == 3 && lc2 == 1 && lc3 == 0
        && wc1 == 5 && wc2 == 1 && wc3 == 0
        && compacted.len() == 3 && compacted[0] == 1 && compacted[2] == 3
        && diff_ab.len() == 3 && diff_ab[0] == 1 && diff_ab[1] == 3 && diff_ab[2] == 5
        && diff_mod.len() == 2 && diff_mod[0] == 10 && diff_mod[1] == 30
        && inter_ab.len() == 2 && inter_ab[0] == 2 && inter_ab[1] == 4
        && inter_mod.len() == 2 && inter_mod[0] == 20 && inter_mod[1] == 30 {
        return 888;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src, vec![]).expect("Algorithms runtime v8 should execute cleanly");
    assert_eq!(res, Some(Value::Int(888)));
}

#[test]
fn test_compiler_constant_folding_v8() {
    let src = r#"
@entry
fn main() -> int {
    // Compile-time folded receiver methods and module calls
    let s16 => math.is_square(16);
    let s17 => math.is_square(17);
    let lc => "alpha\nbeta\ngamma".lines_count();
    let wc => "pure vice compiler constant folding".words_count();
    let comp => [10, 20].compact();
    let diff => [1, 2, 3, 4].diff([2, 4]);
    let inter => [1, 2, 3].intersect([2, 3, 4]);
    let sorted => [1, 2, 3, 4].is_sorted();
    let unique => [1, 2, 3, 4].all_unique();

    if s16 && !s17 && lc == 3 && wc == 5
        && comp.len() == 2 && comp[0] == 10 && comp[1] == 20
        && diff.len() == 2 && diff[0] == 1 && diff[1] == 3
        && inter.len() == 2 && inter[0] == 2 && inter[1] == 3
        && sorted && unique {
        return 999;
    }
    return 0;
}
"#;
    let (res, _) = run_source(src, vec![]).expect("Constant folding v8 should execute cleanly");
    assert_eq!(res, Some(Value::Int(999)));
}
