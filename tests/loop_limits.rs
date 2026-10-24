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
fn loops_deeply_nested() {
    let code = r#"
@entry
fn main() {
    mut counter => 0;
    for i in 0..9 {
        for j in 0..9 {
            for k in 0..9 {
                counter += 1;
            }
        }
    }
    output <- counter;
}
"#;
    assert_eq!(run_vice_code(code), "1000\n");
}

#[test]
fn loops_while_break_huge() {
    let code = r#"
@entry
fn main() {
    mut counter => 0;
    while (true) {
        counter += 1;
        if (counter == 10000) {
            break;
        }
    }
    output <- counter;
}
"#;
    assert_eq!(run_vice_code(code), "10000\n");
}

#[test]
fn loops_continue_skip() {
    let code = r#"
@entry
fn main() {
    mut sum => 0;
    for i in 0..99 {
        if (i % 2 == 0) {
            continue;
        }
        sum += 1;
    }
    output <- sum;
}
"#;
    assert_eq!(run_vice_code(code), "50\n");
}
