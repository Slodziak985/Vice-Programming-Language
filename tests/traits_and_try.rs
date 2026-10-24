use vice::run_source;

fn run(code: &str) -> Vec<String> {
    let (_, output) = run_source(code, vec![]).unwrap();
    output
}

fn semantic_error(code: &str) -> String {
    run_source(code, vec![]).unwrap_err().message
}

#[test]
fn test_trait_basic() {
    let code = r#"
        trait Printable {
            fn print(self);
        }

        data Person {
            name: string => "";
        }

        impl Printable for Person {
            fn print(self) {
                output <- self.name;
            }
        }

        @entry
        fn main() {
            let p: Person => Person(name => "Alice");
            p.print();
        }
    "#;
    assert_eq!(run(code), vec!["Alice"]);
}

#[test]
fn generic_trait_implementation_is_checked_and_runs() {
    let code = r#"
        trait Mapper<T> {
            fn map(self, value: T) -> T;
        }

        data IntMapper {
            enabled: bool => true;
        }

        impl Mapper<int> for IntMapper {
            fn map(self, value: int) -> int {
                return value + 1;
            }
        }

        @entry
        fn main() {
            let mapper => IntMapper();
            output <- mapper.map(41);
        }
    "#;
    assert_eq!(run(code), vec!["42"]);
}

#[test]
fn trait_impl_rejects_missing_method() {
    let error = semantic_error(
        r#"
        trait Named {
            fn name(self) -> string;
        }
        data Person { name: string => ""; }
        impl Named for Person {}
        "#,
    );
    assert!(error.contains("missing trait method 'Named.name'"));
}

#[test]
fn trait_impl_rejects_wrong_parameter_type() {
    let error = semantic_error(
        r#"
        trait Scale {
            fn scale(self, value: int) -> int;
        }
        data Meter { value: int => 0; }
        impl Scale for Meter {
            fn scale(self, value: string) -> int { return 0; }
        }
        "#,
    );
    assert!(error.contains("parameter 2 expected int, got string"));
}

#[test]
fn trait_impl_rejects_wrong_return_type() {
    let error = semantic_error(
        r#"
        trait Named {
            fn name(self) -> string;
        }
        data Person { name: string => ""; }
        impl Named for Person {
            fn name(self) -> int { return 1; }
        }
        "#,
    );
    assert!(error.contains("must return string, got int"));
}

#[test]
fn trait_impl_rejects_extra_method() {
    let error = semantic_error(
        r#"
        trait Named {
            fn name(self) -> string;
        }
        data Person { name: string => ""; }
        impl Named for Person {
            fn name(self) -> string { return self.name; }
            fn age(self) -> int { return 0; }
        }
        "#,
    );
    assert!(error.contains("Method 'age' is not declared by trait 'Named'"));
}

#[test]
fn trait_impl_rejects_unknown_trait() {
    let error = semantic_error(
        r#"
        data Person { name: string => ""; }
        impl Missing for Person {
            fn name(self) -> string { return self.name; }
        }
        "#,
    );
    assert!(error.contains("Unknown trait 'Missing'"));
}

#[test]
fn trait_impl_rejects_duplicate_implementation() {
    let error = semantic_error(
        r#"
        trait Named {
            fn name(self) -> string;
        }
        data Person { name: string => ""; }
        impl Named for Person {
            fn name(self) -> string { return self.name; }
        }
        impl Named for Person {
            fn name(self) -> string { return self.name; }
        }
        "#,
    );
    assert!(error.contains("Trait 'Named' is already implemented for 'Person'"));
}

#[test]
fn trait_requires_parameter_type_annotations() {
    let error = semantic_error(
        r#"
        trait Invalid {
            fn apply(self, value) -> int;
        }
        "#,
    );
    assert!(error.contains("parameter 'value' requires a type annotation"));
}

#[test]
fn impl_self_uses_the_concrete_target_type() {
    let error = semantic_error(
        r#"
        trait Named {
            fn name(self) -> string;
        }
        data Person { name: string => ""; }
        impl Named for Person {
            fn name(self) -> string { return self.missing; }
        }
        "#,
    );
    assert!(error.contains("Field 'missing' not found in data 'Person'"));
}

#[test]
fn trait_requires_self_to_be_the_first_parameter() {
    let error = semantic_error(
        r#"
        trait Invalid {
            fn apply(value: int, self) -> int;
        }
        "#,
    );
    assert!(error.contains("may declare self only once as its first parameter"));
}

#[test]
fn test_result_try_operator() {
    let code = r#"
        fn get_value(x: int) -> Result {
            if (x > 0) {
                return Result(value => x, is_ok => true);
            }
            return Result(error => "negative", is_ok => false);
        }

        @autocheck
        fn process(x: int) -> Result {
            let val => get_value(x)?;
            return Result(value => val * 2, is_ok => true);
        }

        @entry
        @autocheck
        fn main() {
            let res1 => process(5);
            let res2 => process(-1);
            if (res1.is_ok) {
                output <- res1.value;
            }
            if (res2.is_ok == false) {
                output <- res2.error;
            }
        }
    "#;
    assert_eq!(run(code), vec!["10", "negative"]);
}

#[test]
fn test_option_unwrap() {
    let code = r#"
        fn get_opt(x: bool) -> Option {
            if (x) {
                return Option(value => "hello", is_some => true);
            }
            return Option(is_some => false);
        }

        @entry
        @autocheck
        fn main() {
            let val => get_opt(true)!;
            output <- val;
        }
    "#;
    assert_eq!(run(code), vec!["hello"]);
}
