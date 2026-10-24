use vice::run_source;

fn run(code: &str) -> Vec<String> {
    let (_, output) = run_source(code, vec![]).unwrap();
    output
}

#[test]
fn test_match_wildcard() {
    let code = r#"
        @entry
        fn main() {
            let x: int => 42;
            match x {
                1 => { output <- "one"; },
                2 => { output <- "two"; },
                _ => { output <- "other"; }
            }
        }
    "#;
    assert_eq!(run(code), vec!["other"]);
}

#[test]
fn test_match_range() {
    let code = r#"
        @entry
        fn main() {
            let temp: int => 22;
            match temp {
                0..10 => { output <- "cold"; },
                11..25 => { output <- "comfortable"; },
                26..40 => { output <- "hot"; },
                _ => { output <- "extreme"; }
            }
        }
    "#;
    assert_eq!(run(code), vec!["comfortable"]);
}

#[test]
fn test_match_range_boundary() {
    let code = r#"
        @entry
        fn main() {
            let x: int => 10;
            match x {
                0..10 => { output <- "in range"; },
                _ => { output <- "out"; }
            }
        }
    "#;
    assert_eq!(run(code), vec!["in range"]);
}

#[test]
fn test_match_or_pattern() {
    let code = r#"
        @entry
        fn main() {
            let x: int => 3;
            match x {
                1 | 3 | 5 => { output <- "odd"; },
                2 | 4 | 6 => { output <- "even"; },
                _ => { output <- "other"; }
            }
        }
    "#;
    assert_eq!(run(code), vec!["odd"]);
}
