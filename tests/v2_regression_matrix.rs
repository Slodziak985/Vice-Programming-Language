use vice::run_source;

fn assert_expr(expr: &str, expected: impl ToString) {
    let source = format!(
        r#"@entry
fn main() {{
    emit <- {};
}}
"#,
        expr
    );
    let (_, output) = run_source(&source, vec![]).unwrap_or_else(|error| {
        panic!("expression `{expr}` failed: {error}");
    });
    assert_eq!(output, vec![expected.to_string()], "expression `{expr}`");
}

macro_rules! expr_cases {
    ($($name:ident: $expr:expr => $expected:expr;)*) => {
        $(
            #[test]
            fn $name() {
                assert_expr($expr, $expected);
            }
        )*
    };
}

expr_cases! {
    v2_arith_add_001: "1 + 1" => 2;
    v2_arith_add_002: "2 + 3" => 5;
    v2_arith_add_003: "3 + 5" => 8;
    v2_arith_add_004: "4 + 7" => 11;
    v2_arith_add_005: "5 + 9" => 14;
    v2_arith_add_006: "6 + 11" => 17;
    v2_arith_add_007: "7 + 13" => 20;
    v2_arith_add_008: "8 + 15" => 23;
    v2_arith_add_009: "9 + 17" => 26;
    v2_arith_add_010: "10 + 19" => 29;
    v2_arith_add_011: "11 + 21" => 32;
    v2_arith_add_012: "12 + 23" => 35;
    v2_arith_add_013: "13 + 25" => 38;
    v2_arith_add_014: "14 + 27" => 41;
    v2_arith_add_015: "15 + 29" => 44;
    v2_arith_add_016: "16 + 31" => 47;
    v2_arith_add_017: "17 + 33" => 50;
    v2_arith_add_018: "18 + 35" => 53;
    v2_arith_add_019: "19 + 37" => 56;
    v2_arith_add_020: "20 + 39" => 59;
    v2_arith_add_021: "21 + 41" => 62;
    v2_arith_add_022: "22 + 43" => 65;
    v2_arith_add_023: "23 + 45" => 68;
    v2_arith_add_024: "24 + 47" => 71;
    v2_arith_add_025: "25 + 49" => 74;
    v2_arith_add_026: "26 + 51" => 77;
    v2_arith_add_027: "27 + 53" => 80;
    v2_arith_add_028: "28 + 55" => 83;
    v2_arith_add_029: "29 + 57" => 86;
    v2_arith_add_030: "30 + 59" => 89;
    v2_arith_add_031: "31 + 61" => 92;
    v2_arith_add_032: "32 + 63" => 95;
    v2_arith_add_033: "33 + 65" => 98;
    v2_arith_add_034: "34 + 67" => 101;
    v2_arith_add_035: "35 + 69" => 104;
    v2_arith_add_036: "36 + 71" => 107;
    v2_arith_add_037: "37 + 73" => 110;
    v2_arith_add_038: "38 + 75" => 113;
    v2_arith_add_039: "39 + 77" => 116;
    v2_arith_add_040: "40 + 79" => 119;
    v2_arith_add_041: "41 + 81" => 122;
    v2_arith_add_042: "42 + 83" => 125;
    v2_arith_add_043: "43 + 85" => 128;
    v2_arith_add_044: "44 + 87" => 131;
    v2_arith_add_045: "45 + 89" => 134;
    v2_arith_add_046: "46 + 91" => 137;
    v2_arith_add_047: "47 + 93" => 140;
    v2_arith_add_048: "48 + 95" => 143;
    v2_arith_add_049: "49 + 97" => 146;
    v2_arith_add_050: "50 + 99" => 149;
    v2_arith_sub_001: "100 - 1" => 99;
    v2_arith_sub_002: "100 - 2" => 98;
    v2_arith_sub_003: "100 - 3" => 97;
    v2_arith_sub_004: "100 - 4" => 96;
    v2_arith_sub_005: "100 - 5" => 95;
    v2_arith_sub_006: "100 - 6" => 94;
    v2_arith_sub_007: "100 - 7" => 93;
    v2_arith_sub_008: "100 - 8" => 92;
    v2_arith_sub_009: "100 - 9" => 91;
    v2_arith_sub_010: "100 - 10" => 90;
    v2_arith_sub_011: "100 - 11" => 89;
    v2_arith_sub_012: "100 - 12" => 88;
    v2_arith_sub_013: "100 - 13" => 87;
    v2_arith_sub_014: "100 - 14" => 86;
    v2_arith_sub_015: "100 - 15" => 85;
    v2_arith_sub_016: "100 - 16" => 84;
    v2_arith_sub_017: "100 - 17" => 83;
    v2_arith_sub_018: "100 - 18" => 82;
    v2_arith_sub_019: "100 - 19" => 81;
    v2_arith_sub_020: "100 - 20" => 80;
    v2_arith_sub_021: "100 - 21" => 79;
    v2_arith_sub_022: "100 - 22" => 78;
    v2_arith_sub_023: "100 - 23" => 77;
    v2_arith_sub_024: "100 - 24" => 76;
    v2_arith_sub_025: "100 - 25" => 75;
    v2_arith_sub_026: "100 - 26" => 74;
    v2_arith_sub_027: "100 - 27" => 73;
    v2_arith_sub_028: "100 - 28" => 72;
    v2_arith_sub_029: "100 - 29" => 71;
    v2_arith_sub_030: "100 - 30" => 70;
    v2_arith_sub_031: "100 - 31" => 69;
    v2_arith_sub_032: "100 - 32" => 68;
    v2_arith_sub_033: "100 - 33" => 67;
    v2_arith_sub_034: "100 - 34" => 66;
    v2_arith_sub_035: "100 - 35" => 65;
    v2_arith_sub_036: "100 - 36" => 64;
    v2_arith_sub_037: "100 - 37" => 63;
    v2_arith_sub_038: "100 - 38" => 62;
    v2_arith_sub_039: "100 - 39" => 61;
    v2_arith_sub_040: "100 - 40" => 60;
    v2_arith_sub_041: "100 - 41" => 59;
    v2_arith_sub_042: "100 - 42" => 58;
    v2_arith_sub_043: "100 - 43" => 57;
    v2_arith_sub_044: "100 - 44" => 56;
    v2_arith_sub_045: "100 - 45" => 55;
    v2_arith_sub_046: "100 - 46" => 54;
    v2_arith_sub_047: "100 - 47" => 53;
    v2_arith_sub_048: "100 - 48" => 52;
    v2_arith_sub_049: "100 - 49" => 51;
    v2_arith_sub_050: "100 - 50" => 50;
    v2_arith_mul_001: "1 * 3" => 3;
    v2_arith_mul_002: "2 * 3" => 6;
    v2_arith_mul_003: "3 * 3" => 9;
    v2_arith_mul_004: "4 * 3" => 12;
    v2_arith_mul_005: "5 * 3" => 15;
    v2_arith_mul_006: "6 * 3" => 18;
    v2_arith_mul_007: "7 * 3" => 21;
    v2_arith_mul_008: "8 * 3" => 24;
    v2_arith_mul_009: "9 * 3" => 27;
    v2_arith_mul_010: "10 * 3" => 30;
    v2_arith_mul_011: "11 * 3" => 33;
    v2_arith_mul_012: "12 * 3" => 36;
    v2_arith_mul_013: "13 * 3" => 39;
    v2_arith_mul_014: "14 * 3" => 42;
    v2_arith_mul_015: "15 * 3" => 45;
    v2_arith_mul_016: "16 * 3" => 48;
    v2_arith_mul_017: "17 * 3" => 51;
    v2_arith_mul_018: "18 * 3" => 54;
    v2_arith_mul_019: "19 * 3" => 57;
    v2_arith_mul_020: "20 * 3" => 60;
    v2_arith_mul_021: "21 * 3" => 63;
    v2_arith_mul_022: "22 * 3" => 66;
    v2_arith_mul_023: "23 * 3" => 69;
    v2_arith_mul_024: "24 * 3" => 72;
    v2_arith_mul_025: "25 * 3" => 75;
    v2_arith_mul_026: "26 * 3" => 78;
    v2_arith_mul_027: "27 * 3" => 81;
    v2_arith_mul_028: "28 * 3" => 84;
    v2_arith_mul_029: "29 * 3" => 87;
    v2_arith_mul_030: "30 * 3" => 90;
    v2_arith_mul_031: "31 * 3" => 93;
    v2_arith_mul_032: "32 * 3" => 96;
    v2_arith_mul_033: "33 * 3" => 99;
    v2_arith_mul_034: "34 * 3" => 102;
    v2_arith_mul_035: "35 * 3" => 105;
    v2_arith_mul_036: "36 * 3" => 108;
    v2_arith_mul_037: "37 * 3" => 111;
    v2_arith_mul_038: "38 * 3" => 114;
    v2_arith_mul_039: "39 * 3" => 117;
    v2_arith_mul_040: "40 * 3" => 120;
    v2_arith_mod_001: "1 % 5" => 1;
    v2_arith_mod_002: "2 % 5" => 2;
    v2_arith_mod_003: "3 % 5" => 3;
    v2_arith_mod_004: "4 % 5" => 4;
    v2_arith_mod_005: "5 % 5" => 0;
    v2_arith_mod_006: "6 % 5" => 1;
    v2_arith_mod_007: "7 % 5" => 2;
    v2_arith_mod_008: "8 % 5" => 3;
    v2_arith_mod_009: "9 % 5" => 4;
    v2_arith_mod_010: "10 % 5" => 0;
    v2_arith_mod_011: "11 % 5" => 1;
    v2_arith_mod_012: "12 % 5" => 2;
    v2_arith_mod_013: "13 % 5" => 3;
    v2_arith_mod_014: "14 % 5" => 4;
    v2_arith_mod_015: "15 % 5" => 0;
    v2_arith_mod_016: "16 % 5" => 1;
    v2_arith_mod_017: "17 % 5" => 2;
    v2_arith_mod_018: "18 % 5" => 3;
    v2_arith_mod_019: "19 % 5" => 4;
    v2_arith_mod_020: "20 % 5" => 0;
    v2_arith_precedence_001: "1 + 2 * 3" => 7;
    v2_arith_precedence_002: "(1 + 2) * 3" => 9;
    v2_arith_precedence_003: "10 - 2 * 4" => 2;
    v2_arith_precedence_004: "(10 - 2) * 4" => 32;
    v2_arith_precedence_005: "2 * 3 + 4 * 5" => 26;
    v2_arith_precedence_006: "(2 * 3 + 4) * 5" => 50;
    v2_arith_precedence_007: "30 - 4 - 3" => 23;
    v2_arith_precedence_008: "30 - (4 - 3)" => 29;
    v2_arith_precedence_009: "7 + 8 % 3" => 9;
    v2_arith_precedence_010: "(7 + 8) % 3" => 0;
    v2_float_001: "1.5 + 2.5" => "4";
    v2_float_002: "5.5 - 1.25" => "4.25";
    v2_float_003: "2.5 * 4.0" => "10";
    v2_float_004: "10.0 / 4.0" => "2.5";
    v2_float_005: "math.floor(3.99)" => 3;
    v2_float_006: "math.ceil(3.01)" => 4;
    v2_float_007: "math.round(3.49)" => 3;
    v2_float_008: "math.round(3.50)" => 4;
    v2_float_009: "math.abs(-7.25)" => "7.25";
    v2_float_010: "math.sqrt(81.0)" => "9";
    v2_cmp_001: "1 == 1" => true;
    v2_cmp_002: "1 != 2" => true;
    v2_cmp_003: "1 < 2" => true;
    v2_cmp_004: "2 <= 2" => true;
    v2_cmp_005: "3 > 2" => true;
    v2_cmp_006: "3 >= 3" => true;
    v2_cmp_007: "\"a\" == \"a\"" => true;
    v2_cmp_008: "\"a\" != \"b\"" => true;
    v2_cmp_009: "true == true" => true;
    v2_cmp_010: "false != true" => true;
    v2_cmp_011: "10 == 5 + 5" => true;
    v2_cmp_012: "9 != 3 * 4" => true;
    v2_cmp_013: "8 < 3 * 3" => true;
    v2_cmp_014: "8 <= 4 * 2" => true;
    v2_cmp_015: "15 > 4 * 3" => true;
    v2_cmp_016: "16 >= 4 * 4" => true;
    v2_cmp_017: "1.5 < 2.5" => true;
    v2_cmp_018: "2.5 <= 2.5" => true;
    v2_cmp_019: "3.5 > 2.5" => true;
    v2_cmp_020: "3.5 >= 3.5" => true;
    v2_bool_001: "true && true" => true;
    v2_bool_002: "true && false" => false;
    v2_bool_003: "false && true" => false;
    v2_bool_004: "false && false" => false;
    v2_bool_005: "true || true" => true;
    v2_bool_006: "true || false" => true;
    v2_bool_007: "false || true" => true;
    v2_bool_008: "false || false" => false;
    v2_bool_009: "!true" => false;
    v2_bool_010: "!false" => true;
    v2_bool_011: "(1 < 2) && (3 < 4)" => true;
    v2_bool_012: "(1 < 2) && (3 > 4)" => false;
    v2_bool_013: "(1 > 2) || (3 < 4)" => true;
    v2_bool_014: "(1 > 2) || (3 > 4)" => false;
    v2_bool_015: "!(1 > 2)" => true;
    v2_bool_016: "!(1 < 2)" => false;
    v2_string_001: "string.len(\"\")" => 0;
    v2_string_002: "string.len(\"a\")" => 1;
    v2_string_003: "string.len(\"Vice\")" => 4;
    v2_string_004: "string.len(\"żółw\")" => 4;
    v2_string_005: "string.lower(\"VICE\")" => "vice";
    v2_string_006: "string.upper(\"vice\")" => "VICE";
    v2_string_007: "string.trim(\"  Vice  \")" => "Vice";
    v2_string_008: "string.contains(\"Vice language\", \"Vice\")" => true;
    v2_string_009: "string.contains(\"Vice language\", \"Rust\")" => false;
    v2_string_010: "string.starts_with(\"Vice language\", \"Vice\")" => true;
    v2_string_011: "string.starts_with(\"Vice language\", \"language\")" => false;
    v2_string_012: "string.ends_with(\"Vice language\", \"language\")" => true;
    v2_string_013: "string.ends_with(\"Vice language\", \"Vice\")" => false;
    v2_string_014: "\"Vice\" + \"Lang\"" => "ViceLang";
    v2_string_015: "\"v\" + \"0\" + \".2\"" => "v0.2";
    v2_string_016: "string.len(\"hello\" + \"world\")" => 10;
    v2_string_017: "string.upper(string.lower(\"Vice\"))" => "VICE";
    v2_string_018: "string.contains(string.upper(\"vice\"), \"V\")" => true;
    v2_string_019: "string.trim(\"\\tVice\\n\")" => "Vice";
    v2_string_020: "string.len(string.trim(\"  data  \"))" => 4;
    v2_array_001: "array.len([])" => 0;
    v2_array_002: "array.len([1])" => 1;
    v2_array_003: "array.len([1, 2])" => 2;
    v2_array_004: "array.len([1, 2, 3])" => 3;
    v2_array_005: "array.first([4, 5, 6])" => 4;
    v2_array_006: "array.last([4, 5, 6])" => 6;
    v2_array_007: "array.len([\"a\", \"b\", \"c\"])" => 3;
    v2_array_008: "array.first([\"a\", \"b\"])" => "a";
    v2_array_009: "array.last([\"a\", \"b\"])" => "b";
    v2_array_010: "array.len([true, false, true])" => 3;
    v2_option_001: "option.some(1).is_some" => true;
    v2_option_002: "option.some(\"x\").is_some" => true;
    v2_option_003: "option.none().is_some" => false;
    v2_option_004: "option.some(42)!" => 42;
    v2_option_005: "option.some(\"Vice\")!" => "Vice";
    v2_result_001: "result.ok(1).is_ok" => true;
    v2_result_002: "result.ok(\"x\").is_ok" => true;
    v2_result_003: "result.err(\"bad\", 7).is_ok" => false;
    v2_result_004: "result.err(\"bad\", 7).error.message" => "bad";
    v2_result_005: "result.err(\"bad\", 7).error.code" => 7;
    v2_cli_001: "cli.flag([\"--debug\"], \"--debug\")" => true;
    v2_cli_002: "cli.flag([\"--debug\"], \"--release\")" => false;
    v2_cli_003: "cli.option([\"--name\", \"vice\"], \"--name\")!" => "vice";
    v2_cli_004: "cli.option([\"--name\", \"vice\"], \"--missing\").is_some" => false;
    v2_cli_005: "cli.flag([\"--name\", \"vice\", \"--debug\"], \"--debug\")" => true;
    v2_math_001: "math.min(1, 2)" => 1;
    v2_math_002: "math.min(2, 1)" => 1;
    v2_math_003: "math.max(1, 2)" => 2;
    v2_math_004: "math.max(2, 1)" => 2;
    v2_math_005: "math.pow(2.0, 3.0)" => "8";
    v2_math_006: "math.sin(0.0)" => "0";
    v2_math_007: "math.cos(0.0)" => "1";
    v2_math_008: "math.tan(0.0)" => "0";
    v2_math_009: "math.abs(-10)" => 10;
    v2_math_010: "math.abs(10)" => 10;
    v2_nested_001: "math.max(1 + 2, 2 + 2)" => 4;
    v2_nested_002: "math.min(10 - 3, 2 * 4)" => 7;
    v2_nested_003: "string.len(string.upper(\"vice\")) + 1" => 5;
    v2_nested_004: "array.len([1, 2, 3]) + array.len([4, 5])" => 5;
    v2_nested_005: "(option.some(4)!) + 6" => 10;
    v2_nested_006: "result.err(\"bad\", 9).error.code + 1" => 10;
    v2_nested_007: "cli.option([\"--n\", \"8\"], \"--n\")!" => "8";
    v2_nested_008: "string.contains(\"Vice\", \"V\") && option.some(true)!" => true;
    v2_nested_009: "result.ok(1).is_ok && option.none().is_some == false" => true;
    v2_nested_010: "math.floor(math.sqrt(20.0))" => 4;
}

fn assert_program(source: &str, expected: &[&str]) {
    let (_, output) = run_source(source, vec![]).unwrap_or_else(|error| {
        panic!("program failed: {error}\n{source}");
    });
    let expected = expected
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(output, expected);
}

macro_rules! program_cases {
    ($($name:ident: $source:expr => [$($expected:expr),* $(,)?];)*) => {
        $(
            #[test]
            fn $name() {
                assert_program($source, &[$($expected),*]);
            }
        )*
    };
}

program_cases! {
    v2_program_fn_return_int:
        "@entry\nfn main() { emit <- add(20, 22); }\nfn add(a: int, b: int) -> int { return a + b; }\n" => ["42"];
    v2_program_fn_return_string:
        "@entry\nfn main() { emit <- make_label(\"Vice\"); }\nfn make_label(name: string) -> string { return name; }\n" => ["Vice"];
    v2_program_fn_return_bool:
        "@entry\nfn main() { emit <- bigger(8, 3); }\nfn bigger(a: int, b: int) -> bool { return a > b; }\n" => ["true"];
    v2_program_nested_calls:
        "@entry\nfn main() { emit <- twice(add(4, 5)); }\nfn add(a: int, b: int) -> int { return a + b; }\nfn twice(value: int) -> int { return value * 2; }\n" => ["18"];
    v2_program_if_true_branch:
        "@entry\nfn main() { if (10 > 2) { emit <- \"yes\"; } else { emit <- \"no\"; } }\n" => ["yes"];
    v2_program_if_false_branch:
        "@entry\nfn main() { if (2 > 10) { emit <- \"yes\"; } else { emit <- \"no\"; } }\n" => ["no"];
    v2_program_mut_reassignment:
        "@entry\nfn main() { mut value => 1; value = value + 41; emit <- value; }\n" => ["42"];
    v2_program_while_sum:
        "@entry\nfn main() { mut i => 0; mut total => 0; while (i < 5) { total = total + i; i = i + 1; } emit <- total; }\n" => ["10"];
    v2_program_while_factorial:
        "@entry\nfn main() { mut i => 1; mut total => 1; while (i <= 5) { total = total * i; i = i + 1; } emit <- total; }\n" => ["120"];
    v2_program_while_break:
        "@entry\nfn main() { mut i => 0; while (true) { i = i + 1; if (i == 4) { break; } } emit <- i; }\n" => ["4"];
    v2_program_while_continue:
        "@entry\nfn main() { mut i => 0; mut total => 0; while (i < 5) { i = i + 1; if (i == 3) { continue; } total = total + i; } emit <- total; }\n" => ["12"];
    v2_program_match_literal:
        "@entry\nfn main() { match 2 { 1 => { emit <- \"one\"; }, 2 => { emit <- \"two\"; }, _ => { emit <- \"other\"; } } }\n" => ["two"];
    v2_program_match_range:
        "@entry\nfn main() { match 7 { 0..5 => { emit <- \"low\"; }, 6..10 => { emit <- \"mid\"; }, _ => { emit <- \"high\"; } } }\n" => ["mid"];
    v2_program_match_or:
        "@entry\nfn main() { match 5 { 1 | 3 | 5 => { emit <- \"odd\"; }, 2 | 4 => { emit <- \"even\"; }, _ => { emit <- \"other\"; } } }\n" => ["odd"];
    v2_program_match_expression:
        "@entry\nfn main() { let name => match 3 { 1 => \"one\", 3 => \"three\", _ => \"other\" }; emit <- name; }\n" => ["three"];
    v2_program_array_index:
        "@entry\nfn main() { let values: int[] => [3, 5, 8]; emit <- values[2]; }\n" => ["8"];
    v2_program_array_push:
        "@entry\nfn main() { mut values: int[] => [1, 2]; values = values.push(3); emit <- values[2]; }\n" => ["3"];
    v2_program_data_field:
        "data User { name: string => \"\"; score: int => 0; }\n@entry\nfn main() { let user => User(name => \"Vice\", score => 42); emit <- user.name, user.score; }\n" => ["Vice 42"];
    v2_program_impl_method:
        "data Counter { value: int => 0; }\nimpl Counter { fn inc(self) -> int { return self.value + 1; } }\n@entry\nfn main() { let counter => Counter(value => 41); emit <- counter.inc(); }\n" => ["42"];
    v2_program_enum_match:
        "enum Mode { Dev, Release }\n@entry\nfn main() { let mode: Mode => Mode.Release; match mode { Mode.Dev => { emit <- \"dev\"; }, Mode.Release => { emit <- \"release\"; } } }\n" => ["release"];
    v2_program_result_try_success:
        "fn get() -> Result { return result.ok(42); }\n@entry\nfn main() { let value => get()?; emit <- value; }\n" => ["42"];
    v2_program_option_unwrap:
        "@entry\nfn main() { let value => option.some(\"Vice\")!; emit <- value; }\n" => ["Vice"];
    v2_program_cli_helpers:
        "@entry\nfn main() { let args => [\"--name\", \"Vice\", \"--debug\"]; emit <- cli.flag(args, \"--debug\"), cli.option(args, \"--name\")!; }\n" => ["true Vice"];
    v2_program_string_pipeline:
        "@entry\nfn main() { let name => string.upper(string.trim(\" vice \")); emit <- name; }\n" => ["VICE"];
    v2_program_result_object:
        "@entry\nfn main() { let error => result.err(\"bad\", 12); emit <- error.is_ok, error.error.message, error.error.code; }\n" => ["false bad 12"];
}

expr_cases! {
    v2_extra_arith_001: "101 + 1" => 102;
    v2_extra_arith_002: "102 + 2" => 104;
    v2_extra_arith_003: "103 + 3" => 106;
    v2_extra_arith_004: "104 + 4" => 108;
    v2_extra_arith_005: "105 + 5" => 110;
    v2_extra_arith_006: "106 + 6" => 112;
    v2_extra_arith_007: "107 + 7" => 114;
    v2_extra_arith_008: "108 + 8" => 116;
    v2_extra_arith_009: "109 + 9" => 118;
    v2_extra_arith_010: "110 + 10" => 120;
    v2_extra_arith_011: "111 - 1" => 110;
    v2_extra_arith_012: "112 - 2" => 110;
    v2_extra_arith_013: "113 - 3" => 110;
    v2_extra_arith_014: "114 - 4" => 110;
    v2_extra_arith_015: "115 - 5" => 110;
    v2_extra_arith_016: "116 - 6" => 110;
    v2_extra_arith_017: "117 - 7" => 110;
    v2_extra_arith_018: "118 - 8" => 110;
    v2_extra_arith_019: "119 - 9" => 110;
    v2_extra_arith_020: "120 - 10" => 110;
    v2_extra_arith_021: "11 * 11" => 121;
    v2_extra_arith_022: "12 * 12" => 144;
    v2_extra_arith_023: "13 * 13" => 169;
    v2_extra_arith_024: "14 * 14" => 196;
    v2_extra_arith_025: "15 * 15" => 225;
    v2_extra_arith_026: "16 * 16" => 256;
    v2_extra_arith_027: "17 * 17" => 289;
    v2_extra_arith_028: "18 * 18" => 324;
    v2_extra_arith_029: "19 * 19" => 361;
    v2_extra_arith_030: "20 * 20" => 400;
    v2_extra_bool_001: "(10 % 2 == 0) && true" => true;
    v2_extra_bool_002: "(11 % 2 == 0) && true" => false;
    v2_extra_bool_003: "(12 % 3 == 0) || false" => true;
    v2_extra_bool_004: "(13 % 3 == 0) || false" => false;
    v2_extra_bool_005: "!(14 < 10)" => true;
    v2_extra_bool_006: "!(9 < 10)" => false;
    v2_extra_bool_007: "\"vice\" == string.lower(\"VICE\")" => true;
    v2_extra_bool_008: "\"VICE\" == string.upper(\"vice\")" => true;
    v2_extra_bool_009: "array.len([1, 2, 3, 4]) == 4" => true;
    v2_extra_bool_010: "option.some(false)!" => false;
    v2_extra_string_001: "\"Vice \" + \"v0.2\"" => "Vice v0.2";
    v2_extra_string_002: "string.upper(\"v\") + string.lower(\"ICE\")" => "Vice";
    v2_extra_string_003: "string.trim(\" docs \") + \":ok\"" => "docs:ok";
    v2_extra_string_004: "string.lower(\"LEARN\") + \".html\"" => "learn.html";
    v2_extra_string_005: "string.upper(\"cli\") + \" tool\"" => "CLI tool";
    v2_extra_string_006: "\"safe \" + \"stdlib\"" => "safe stdlib";
    v2_extra_string_007: "\"emit\" + \" <-\"" => "emit <-";
    v2_extra_string_008: "\"fail\" + \" <-\"" => "fail <-";
    v2_extra_string_009: "\"use\" + \" import\"" => "use import";
    v2_extra_string_010: "\"vice\" + \".toml\"" => "vice.toml";
    v2_extra_math_001: "math.max(math.min(10, 20), 5)" => 10;
    v2_extra_math_002: "math.min(math.max(10, 20), 25)" => 20;
    v2_extra_math_003: "math.floor(9.9) + math.ceil(0.1)" => 10;
    v2_extra_math_004: "math.round(2.4) + math.round(2.6)" => 5;
    v2_extra_math_005: "math.abs(-5) + math.abs(6)" => 11;
    v2_extra_math_006: "math.sqrt(16.0) + math.sqrt(25.0)" => "9";
    v2_extra_math_007: "math.pow(3.0, 2.0) + 1.0" => "10";
    v2_extra_math_008: "math.floor(math.pow(2.0, 5.0))" => 32;
    v2_extra_math_009: "math.ceil(math.sqrt(30.0))" => 6;
    v2_extra_math_010: "math.round(math.sqrt(30.0))" => 5;
    v2_extra_option_001: "option.some(array.len([1, 2]))!" => 2;
    v2_extra_option_002: "option.some(string.len(\"Vice\"))!" => 4;
    v2_extra_option_003: "option.some(math.max(4, 7))!" => 7;
    v2_extra_option_004: "option.some(cli.flag([\"--x\"], \"--x\"))!" => true;
    v2_extra_option_005: "option.none().is_some == false" => true;
    v2_extra_result_001: "result.ok(array.len([1, 2, 3])).value" => 3;
    v2_extra_result_002: "result.ok(string.upper(\"ok\")).value" => "OK";
    v2_extra_result_003: "result.ok(math.min(3, 9)).value" => 3;
    v2_extra_result_004: "result.err(\"missing\", 404).error.code" => 404;
    v2_extra_result_005: "result.err(\"missing\", 404).error.message" => "missing";
}
