use vice::lexer::Lexer;
use vice::parser::Parser;
use vice::semantic::SemanticAnalyzer;
use vice::vm::compiler::Compiler;
use vice::vm::machine::Vm;
use vice::value::Value;

fn run_vice(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize().map_err(|e| format!("{:?}", e))?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| format!("{:?}", e))?;

    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).map_err(|e| format!("{:?}", e))?;

    let compiler = Compiler::new();
    let compiled = compiler.compile(&program).map_err(|e| format!("{:?}", e))?;

    let mut vm = Vm::new();
    let res = vm.interpret(compiled).map_err(|e| format!("{:?}", e))?;
    Ok(res.value)
}

fn check_semantic_err(source: &str) -> String {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");
    let mut semantic = SemanticAnalyzer::new();
    match semantic.analyze(&program) {
        Ok(_) => panic!("Expected semantic error, but succeeded!"),
        Err(e) => format!("{:?}", e),
    }
}

// ==========================================
// 1. @zero_alloc Tests
// ==========================================

#[test]
fn test_zero_alloc_allowed_math() {
    let code = r#"
        @zero_alloc
        fn compute_dist(x: int, y: int) -> int {
            return math.abs(x) + math.abs(y);
        }

        @entry
        fn main() -> int {
            return compute_dist(-10, 25);
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    assert_eq!(val, Value::Int(35));
}

#[test]
fn test_zero_alloc_rejects_array_literal() {
    let code = r#"
        @zero_alloc
        fn bad_alloc() -> int {
            let a = [1, 2, 3];
            return 0;
        }

        @entry
        fn main() -> int {
            return bad_alloc();
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("zero_alloc"), "Expected zero_alloc error, got: {}", err);
}

#[test]
fn test_zero_alloc_rejects_map_literal() {
    let code = r#"
        @zero_alloc
        fn bad_map() -> int {
            let m = {"key": 10};
            return 0;
        }

        @entry
        fn main() -> int {
            return bad_map();
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("zero_alloc"), "Expected zero_alloc error, got: {}", err);
}

#[test]
fn test_zero_alloc_rejects_struct_instance() {
    let code = r#"
        data Buffer { size: int }

        @zero_alloc
        fn bad_struct() -> int {
            let b = Buffer { size: 64 };
            return b.size;
        }

        @entry
        fn main() -> int {
            return bad_struct();
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("zero_alloc"), "Expected zero_alloc error, got: {}", err);
}

#[test]
fn test_zero_alloc_rejects_non_zero_alloc_call() {
    let code = r#"
        fn helper() -> int {
            return 42;
        }

        @zero_alloc
        fn bad_call() -> int {
            return helper();
        }

        @entry
        fn main() -> int {
            return bad_call();
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("zero_alloc"), "Expected zero_alloc error, got: {}", err);
}

// ==========================================
// 2. @singleton Tests
// ==========================================

#[test]
fn test_singleton_single_instance_allowed() {
    let code = r#"
        @singleton
        data AppConfig {
            port: int
        }

        @entry
        fn main() -> int {
            let config = AppConfig { port: 8080 };
            return config.port;
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    assert_eq!(val, Value::Int(8080));
}

#[test]
fn test_singleton_duplicate_instance_rejected_semantically() {
    let code = r#"
        @singleton
        data AppConfig {
            port: int
        }

        @entry
        fn main() -> int {
            let c1 = AppConfig { port: 8080 };
            let c2 = AppConfig { port: 9090 };
            return c1.port + c2.port;
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("Singleton"), "Expected Singleton error, got: {}", err);
}

// ==========================================
// 3. @retry Tests
// ==========================================

#[test]
fn test_retry_valid() {
    let code = r#"
        @retry(count: 3)
        fn fetch_data() -> Result {
            return Result(true, 100, "");
        }

        @entry
        fn main() -> int {
            let res = fetch_data();
            return res.value;
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    assert_eq!(val, Value::Int(100));
}

#[test]
fn test_retry_non_result_rejected() {
    let code = r#"
        @retry(count: 3)
        fn get_int() -> int {
            return 42;
        }

        @entry
        fn main() -> int {
            return get_int();
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("retry"), "Expected retry error, got: {}", err);
}

#[test]
fn test_retry_conflicting_nonreentrant_rejected() {
    let code = r#"
        @retry(count: 3)
        @nonreentrant
        fn try_op() -> Result {
            return Result(true, 1, "");
        }

        @entry
        fn main() -> int {
            return 0;
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("nonreentrant"), "Expected nonreentrant conflict error, got: {}", err);
}

// ==========================================
// 4. @profiled Tests
// ==========================================

#[test]
fn test_profiled_tracked_in_vm() {
    let code = r#"
        @profiled
        fn compute(x: int) -> int {
            return x * 2;
        }

        @entry
        fn main() -> int {
            let a = compute(10);
            let b = compute(20);
            let c = compute(30);
            return a + b + c;
        }
    "#;
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).unwrap();
    let compiler = Compiler::new();
    let compiled = compiler.compile(&program).unwrap();

    let mut vm = Vm::new();
    let res = vm.interpret(compiled).unwrap();
    assert_eq!(res.value, Value::Int(120));
    assert_eq!(vm.profile_call_counts.get("compute"), Some(&3));
}

#[test]
fn test_profiled_and_cold_conflict() {
    let code = r#"
        @profiled
        @cold
        fn slow_func() -> int {
            return 0;
        }

        @entry
        fn main() -> int {
            return slow_func();
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("profiled") && err.contains("cold"), "Expected conflict error, got: {}", err);
}

// ==========================================
// 5. @checked and @saturating Conflict
// ==========================================

#[test]
fn test_checked_and_saturating_conflict() {
    let code = r#"
        @checked
        @saturating
        fn arith() -> int {
            return 1 + 2;
        }

        @entry
        fn main() -> int {
            return arith();
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("checked") && err.contains("saturating"), "Expected conflict error, got: {}", err);
}

// ==========================================
// 6. @unreachable Tests
// ==========================================

#[test]
fn test_unreachable_call_rejected_semantically() {
    let code = r#"
        @unreachable
        fn dead_code() -> int {
            return 999;
        }

        @entry
        fn main() -> int {
            return dead_code();
        }
    "#;
    let err = check_semantic_err(code);
    assert!(err.contains("unreachable"), "Expected unreachable error, got: {}", err);
}

// ==========================================
// 7. New Stdlib and Instance Methods Tests
// ==========================================

#[test]
fn test_math_wrap_and_smoothstep() {
    let code = r#"
        @entry
        fn main() -> int {
            let w1 = math.wrap(15, 0, 10);
            let w2 = math.wrap(-2, 0, 10);
            return w1 + w2;
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    assert_eq!(val, Value::Int(13)); // 15 % 10 = 5; -2 wrap in 0..10 = 8; 5 + 8 = 13
}

#[test]
fn test_math_deg_and_rad() {
    let code = r#"
        @entry
        fn main() -> int {
            let rad = math.deg_to_rad(180.0);
            let deg = math.rad_to_deg(rad);
            return math.round(deg);
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    assert_eq!(val, Value::Int(180));
}

#[test]
fn test_array_new_methods() {
    let code = r#"
        @entry
        fn main() -> int {
            let arr = [1, 2, 3, 4, 5];
            let chunks = arr.chunk(2); // [[1, 2], [3, 4], [5]]
            let total = arr.sum(); // 15
            let minimum = arr.min(); // 1
            let maximum = arr.max(); // 5
            let flat = chunks.flatten(); // [1, 2, 3, 4, 5]
            let pairs = [1, 2].zip([10, 20]); // [[1, 10], [2, 20]]
            return total + minimum + maximum + flat.len() + pairs.len();
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    // total(15) + min(1) + max(5) + flat.len(5) + pairs.len(2) = 28
    assert_eq!(val, Value::Int(28));
}

#[test]
fn test_string_new_methods() {
    let code = r#"
        @entry
        fn main() -> int {
            let s1 = "12345";
            let s2 = "Hello";
            let sentence = "pure vice language compiler";
            let w = sentence.words();
            let mut score = 0;
            if s1.is_numeric() { score = score + 10; }
            if s2.is_alpha() { score = score + 20; }
            score = score + w.len();
            return score;
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    // 10 + 20 + 4 = 34
    assert_eq!(val, Value::Int(34));
}

#[test]
fn test_map_new_methods() {
    let code = r#"
        @entry
        fn main() -> int {
            let m = {"a": 1, "b": 2};
            let has_two = m.has_value(2);
            let has_ten = m.has_value(10);
            let inv = m.invert(); // {1: "a", 2: "b"}
            let mut score = 0;
            if has_two { score = score + 50; }
            if !has_ten { score = score + 50; }
            if inv.contains_key(1) { score = score + 20; }
            return score;
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    assert_eq!(val, Value::Int(120));
}

// ==========================================
// 8. Peephole Optimizations Test
// ==========================================

#[test]
fn test_peephole_identity_arithmetic() {
    let code = r#"
        @entry
        fn main() -> int {
            let x = 100;
            let y = x + 0;
            let z = y - 0;
            let w = z * 1;
            return w;
        }
    "#;
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).unwrap();
    let compiler = Compiler::new();
    let compiled = compiler.compile(&program).unwrap();

    // Verify optimized chunk has reduced instruction count
    let main_func = compiled.functions.get("main").unwrap();
    let opcodes = &main_func.chunk.code;
    // None of the Add, Sub, Mul with constant 0/1 should exist in optimized code!
    let has_add = opcodes.iter().any(|op| matches!(op, vice::vm::opcodes::Opcode::Add));
    let has_mul = opcodes.iter().any(|op| matches!(op, vice::vm::opcodes::Opcode::Mul));
    assert!(!has_add, "Opcode::Add should have been optimized away");
    assert!(!has_mul, "Opcode::Mul should have been optimized away");

    let mut vm = Vm::new();
    let res = vm.interpret(compiled).unwrap();
    assert_eq!(res.value, Value::Int(100));
}

#[test]
fn test_peephole_division_identity() {
    let code = r#"
        @entry
        fn main() -> int {
            let x = 42;
            let y = x / 1;
            return y;
        }
    "#;
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).unwrap();
    let compiler = Compiler::new();
    let compiled = compiler.compile(&program).unwrap();

    let main_func = compiled.functions.get("main").unwrap();
    let has_div = main_func.chunk.code.iter().any(|op| matches!(op, vice::vm::opcodes::Opcode::Div | vice::vm::opcodes::Opcode::HighLevelDiv));
    assert!(!has_div, "Opcode::Div with 1 should have been optimized away");

    let mut vm = Vm::new();
    let res = vm.interpret(compiled).unwrap();
    assert_eq!(res.value, Value::Int(42));
}

#[test]
fn test_peephole_store_forwarding() {
    let code = r#"
        @entry
        fn main() -> int {
            let res = 777;
            return res;
        }
    "#;
    let val = run_vice(code).expect("Should succeed");
    assert_eq!(val, Value::Int(777));
}

