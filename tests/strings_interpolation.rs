use vice::parse_source;
use vice::run_source;
use vice::semantic::SemanticAnalyzer;

fn run_test(source: &str) -> String {
    let (_, output) = run_source(source, vec![]).unwrap();
    output.join("\n")
}

fn run_test_error(source: &str) -> bool {
    run_source(source, vec![]).is_err()
        || parse_source(source)
            .and_then(|p| SemanticAnalyzer::new().analyze(&p))
            .is_err()
}

#[test]
fn test_interpolation_basic() {
    let source = "
    @autocheck
    @entry
    fn main() {
        let name => \"John\";
        let msg => \"Hello ${name}!\";
        output <- msg;
    }
    ";
    assert_eq!(run_test(source), "Hello John!");
}

#[test]
fn test_interpolation_multiple() {
    let source = "
    @autocheck
    @entry
    fn main() {
        let x => 10;
        let y => 20;
        output <- \"Sum of ${x} and ${y} is ${x + y}.\";
    }
    ";
    assert_eq!(run_test(source), "Sum of 10 and 20 is 30.");
}

#[test]
fn test_interpolation_nested_math() {
    let source = "
    @autocheck
    @entry
    fn main() {
        let hp => 100;
        let damage => 15;
        output <- \"Current HP: ${hp - damage}\";
    }
    ";
    assert_eq!(run_test(source), "Current HP: 85");
}

#[test]
fn test_interpolation_boolean() {
    let source = "
    @autocheck
    @entry
    fn main() {
        let is_admin => true;
        output <- \"Admin status: ${is_admin}\";
    }
    ";
    assert_eq!(run_test(source), "Admin status: true");
}

#[test]
fn test_interpolation_function_call() {
    let source = "
    fn get_greeting() -> string {
        return \"Hello\";
    }

    @autocheck
    @entry
    fn main() {
        let name => \"Alice\";
        output <- \"${get_greeting()}, ${name}!\";
    }
    ";
    assert_eq!(run_test(source), "Hello, Alice!");
}

#[test]
fn test_interpolation_unclosed() {
    let source = "
    @autocheck
    @entry
    fn main() {
        let hp => 100;
        let msg => \"Unclosed ${hp \";
    }
    ";
    assert!(run_test_error(source));
}

#[test]
fn test_interpolation_syntax_error_inside() {
    let source = "
    @autocheck
    @entry
    fn main() {
        let msg => \"Invalid ${10 + }\";
    }
    ";
    assert!(run_test_error(source));
}
