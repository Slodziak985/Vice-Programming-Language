use vice::parse_source;
use vice::semantic::SemanticAnalyzer;

fn run_vice_code(code: &str) -> String {
    let program = parse_source(code).expect("Parser failed");
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).expect("Semantic check failed");
    let output = vice::execute_program_vm(&program).expect("Interpreter failed");
    output.join("\n") + "\n"
}

#[test]
fn memory_deep_recursion() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let code_ok = r#"
fn recurse(n: int) -> int {
    if (n == 0) {
        return 0;
    }
    return 1 + recurse(n - 1);
}

@entry
fn main() {
    output <- recurse(5); // 5 frames of recursion
}
"#;
            assert_eq!(run_vice_code(code_ok), "5\n");

            let code_err = r#"
fn recurse(n: int) -> int {
    if (n == 0) {
        return 0;
    }
    return 1 + recurse(n - 1);
}

@entry
fn main() {
    output <- recurse(80); // Exceeds max_call_depth of 64
}
"#;
            let program = vice::parse_source(code_err).expect("Parser failed");
            let mut semantic = vice::semantic::SemanticAnalyzer::new();
            semantic.analyze(&program).expect("Semantic check failed");
            let result = vice::execute_program_vm(&program);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().message, "Maximum call depth exceeded");
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn memory_deep_scope_shadowing() {
    let code = r#"
@entry
fn main() {
    let a => 1;
    {
        let a => 2;
        {
            let a => 3;
            {
                let a => 4;
                {
                    let a => 5;
                    output <- a;
                }
                output <- a;
            }
            output <- a;
        }
        output <- a;
    }
    output <- a;
}
"#;
    assert_eq!(run_vice_code(code), "5\n4\n3\n2\n1\n");
}

#[test]
fn memory_large_map() {
    let code = r#"
@entry
fn main() {
    mut m => {};
    // Not too big, but enough to ensure hashing and resizing works
    for i in 0..1000 {
        m[to_str(i)] = i;
    }
    output <- m["999"];
}
"#;
    assert_eq!(run_vice_code(code), "999\n");
}
