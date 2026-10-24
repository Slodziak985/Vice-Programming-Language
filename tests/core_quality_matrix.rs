use std::fs;

use vice::parse_source;
use vice::semantic::SemanticAnalyzer;
use vice::stdlib;

macro_rules! vice_case_count {
    ($name:ident, $count:expr) => {
        const $name: usize = $count;
    };
}

vice_case_count!(PARSER_INVALID_MATRIX_CASES, 320);
vice_case_count!(SEMANTIC_INVALID_MATRIX_CASES, 320);
vice_case_count!(VM_RUNTIME_MATRIX_CASES, 320);
vice_case_count!(STDLIB_MATRIX_CASES, 240);
vice_case_count!(SECURITY_SANDBOX_MATRIX_CASES, 320);

fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn parser_invalid_matrix_cases_do_not_panic() {
    for i in 0..PARSER_INVALID_MATRIX_CASES {
        let source = match i % 4 {
            0 => format!("@entry\nfn main() {{\n    let x => [{i}, ;\n}}\n"),
            1 => format!("@entry\nfn broken_{i}( {{\n}}\n"),
            2 => format!("data Bad{i} {{ value: int,, }}\n@entry\nfn main() {{}}\n"),
            _ => format!("@entry\nfn main() {{\n    let x => \"unterminated {i};\n}}\n"),
        };
        assert!(
            parse_source(&source).is_err(),
            "parser case {i} unexpectedly parsed"
        );
    }
}

#[test]
fn semantic_invalid_matrix_cases_do_not_panic() {
    for i in 0..SEMANTIC_INVALID_MATRIX_CASES {
        let source = match i % 4 {
            0 => format!("@entry\nfn main() {{\n    let value: int => \"case{i}\";\n}}\n"),
            1 => format!("@entry\nfn main() {{\n    if ({i}) {{ emit <- \"bad\"; }}\n}}\n"),
            2 => String::from(
                "@entry\nfn main() {\n    let items: int[] => [1, 2, 3];\n    let value: string => items[0];\n}\n",
            ),
            _ => format!(
                "fn takes_int(value: int) {{}}\n@entry\nfn main() {{\n    takes_int(\"case{i}\");\n}}\n"
            ),
        };
        let program = parse_source(&source).unwrap_or_else(|error| {
            panic!("semantic case {i} did not parse before analysis: {error}");
        });
        let mut semantic = SemanticAnalyzer::new();
        assert!(
            semantic.analyze(&program).is_err(),
            "semantic case {i} unexpectedly passed"
        );
    }
}

#[test]
fn vm_runtime_matrix_cases_produce_expected_output() {
    let mut source = String::from("@entry\nfn main() {\n");
    let mut expected = Vec::new();
    for i in 0..VM_RUNTIME_MATRIX_CASES {
        let a = i as i64;
        let b = (i % 17 + 1) as i64;
        match i % 4 {
            0 => {
                source.push_str(&format!("    emit <- {} + {};\n", a, b));
                expected.push((a + b).to_string());
            }
            1 => {
                source.push_str(&format!("    emit <- {} * {} - {};\n", a, b, b));
                expected.push((a * b - b).to_string());
            }
            2 => {
                source.push_str(&format!(
                    "    let arr_{i} => [{}, {}, {}];\n    emit <- arr_{i}[1];\n",
                    a,
                    a + b,
                    a + b + 1
                ));
                expected.push((a + b).to_string());
            }
            _ => {
                source.push_str(&format!(
                    "    let item_{i} => {{\"value\": {}}};\n    emit <- item_{i}[\"value\"];\n",
                    a + b
                ));
                expected.push((a + b).to_string());
            }
        }
    }
    source.push_str("}\n");

    let (_, output) = vice::run_source(&source, vec![]).unwrap();
    assert_eq!(output, expected);
}

#[test]
fn stdlib_matrix_cases_produce_expected_output() {
    let mut source = String::from("@entry\nfn main() {\n");
    let mut expected = Vec::new();
    for i in 0..STDLIB_MATRIX_CASES {
        match i % 4 {
            0 => {
                source.push_str(&format!("    emit <- string.len(\"case{i}\");\n"));
                expected.push(format!("case{i}").chars().count().to_string());
            }
            1 => {
                source.push_str(&format!("    emit <- string.upper(\"v{i}\");\n"));
                expected.push(format!("V{i}"));
            }
            2 => {
                source.push_str(&format!(
                    "    emit <- array.len([{}, {}, {}]);\n",
                    i,
                    i + 1,
                    i + 2
                ));
                expected.push("3".to_string());
            }
            _ => {
                source.push_str(&format!(
                    "    emit <- string.starts_with(\"vice-{i}\", \"vice\");\n"
                ));
                expected.push("true".to_string());
            }
        }
    }
    source.push_str("}\n");

    let (_, output) = vice::run_source(&source, vec![]).unwrap();
    assert_eq!(output, expected);
}

#[test]
fn security_sandbox_matrix_denies_root_escape() {
    let parent = unique_temp_dir("vice_security_matrix");
    let root = parent.join("project");
    let nested = root.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let outside = parent.join("outside.txt");
    fs::write(&outside, "secret").unwrap();
    stdlib::set_sandbox_root(Some(root.clone())).unwrap();

    let patterns = [
        "../outside.txt",
        "..\\outside.txt",
        "nested/../../outside.txt",
        "nested\\..\\..\\outside.txt",
    ];

    for i in 0..SECURITY_SANDBOX_MATRIX_CASES {
        let path = patterns[i % patterns.len()];
        let source = format!(
            "@entry\nfn main() {{\n    let value => fs.read_text(\"{}\");\n    emit <- value;\n}}\n",
            path.replace('\\', "\\\\")
        );
        let error = vice::run_source(&source, vec![]).unwrap_err();
        assert!(
            error.message.contains("Sandbox denied"),
            "security case {i} returned unexpected error: {}",
            error.message
        );
    }

    stdlib::set_sandbox_root(None).unwrap();
    fs::remove_dir_all(parent).unwrap();
}
