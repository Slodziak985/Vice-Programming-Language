use vice::parse_source;
use vice::run_source;
use vice::semantic::SemanticAnalyzer;
use vice::value::{MapKey, Value};

fn run_test_error(source: &str, expected_error: &str) {
    let error = parse_source(source)
        .and_then(|p| SemanticAnalyzer::new().analyze(&p))
        .unwrap_err();
    assert!(
        error.message.contains(expected_error),
        "Expected error containing '{}', got '{}'",
        expected_error,
        error.message
    );
}

#[test]
fn test_map_creation() {
    let source = r#"
    @entry
    fn main() {
        let m: map[string, int] => { "a": 1, "b": 2 };
        output <- m;
    }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["{\"a\": 1, \"b\": 2}"]);
}

#[test]
fn test_map_indexing() {
    let source = r#"
    @entry
    fn main() {
        let m: map[string, int] => { "apple": 5, "banana": 10 };
        output <- m["apple"];
        output <- m["banana"];
    }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["5", "10"]);
}

#[test]
fn test_map_mutation() {
    let source = r#"
    @entry
    fn main() {
        mut m: map[string, string] => { "user": "admin" };
        m["user"] = "guest";
        m["role"] = "editor";
        output <- m["user"];
        output <- m["role"];
    }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["guest", "editor"]);
}

#[test]
fn test_map_int_keys() {
    let source = r#"
    @entry
    fn main() {
        let m: map[int, string] => { 1: "one", 2: "two" };
        output <- m[1];
        output <- m[2];
    }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["one", "two"]);
}

#[test]
fn test_map_missing_key() {
    let source = r#"
    @entry
    fn main() {
        let m: map[string, int] => { "a": 1 };
        output <- m["b"];
    }
    "#;
    let err = run_source(source, vec![]).unwrap_err();
    assert!(err.message.contains("Map key not found: b"));
}

#[test]
fn test_map_invalid_key_type() {
    let source = r#"
    @entry
    fn main() {
        let m: map[string, int] => { "a": 1 };
        output <- m[123];
    }
    "#;
    run_test_error(source, "Map key must be string, got int");
}

#[test]
fn test_map_mixed_types_error() {
    let source = r#"
    @autocheck
    @entry
    fn main() {
        let m => { "a": 1, "b": "string" };
        output <- m;
    }
    "#;
    run_test_error(source, "Map values must have one type, got int and string");
}

#[test]
fn test_map_nested() {
    let source = r#"
    @entry
    fn main() {
        let nested: map[string, map[string, int]] => {
            "player": { "hp": 100, "mp": 50 }
        };
        output <- nested["player"]["hp"];
        output <- nested["player"]["mp"];
    }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["100", "50"]);
}

#[test]
fn test_map_nested_mutation() {
    let source = r#"
    @entry
    fn main() {
        mut state: map[string, map[string, int]] => {
            "p1": { "score": 0 }
        };
        state["p1"]["score"] = 500;
        output <- state["p1"]["score"];
    }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["500"]);
}

#[test]
fn test_map_array_values() {
    let source = r#"
    @entry
    fn main() {
        mut m: map[string, int[]] => {
            "primes": [2, 3, 5]
        };
        m["primes"][0] = 99;
        output <- m["primes"][0];
    }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["99"]);
}

#[test]
fn runtime_map_keys_preserve_value_types() {
    let keys = [
        MapKey::from_value(&Value::Int(1)).unwrap(),
        MapKey::from_value(&Value::String("1".to_string())).unwrap(),
        MapKey::from_value(&Value::Float(1.0)).unwrap(),
        MapKey::from_value(&Value::Bool(true)).unwrap(),
    ];
    let unique = keys.into_iter().collect::<std::collections::HashSet<_>>();
    assert_eq!(unique.len(), 4);
}

#[test]
fn float_map_keys_have_stable_zero_and_nan_identity() {
    let positive_zero = MapKey::from_value(&Value::Float(0.0)).unwrap();
    let negative_zero = MapKey::from_value(&Value::Float(-0.0)).unwrap();
    assert_eq!(positive_zero, negative_zero);

    let first_nan = MapKey::from_value(&Value::Float(f64::NAN)).unwrap();
    let second_nan =
        MapKey::from_value(&Value::Float(f64::from_bits(0x7ff8_0000_0000_0001))).unwrap();
    assert_eq!(first_nan, second_nan);
}
