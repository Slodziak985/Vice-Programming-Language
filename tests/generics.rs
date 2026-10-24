use vice::run_source;

#[test]
fn test_generic_function_parsing_and_execution() {
    let source = r#"
        fn identity<T>(value: T) -> T {
            return value;
        }

        @entry
        fn main() {
            let a: int => identity::<int>(42);
            let b: string => identity::<string>("hello");
            
            output <- a;
            output <- b;
        }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["42", "hello"]);
}

#[test]
fn test_generic_data_declaration() {
    let source = r#"
        data Box<T> {
            value: T => 0;
        }

        @entry
        fn main() {
            let b1: Box<int> => Box::<int>(value => 10);
            let b2: Box<string> => Box::<string>(value => "test");

            output <- b1.value;
            output <- b2.value;
        }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["10", "test"]);
}

#[test]
fn test_generic_impl_block() {
    let source = r#"
        data Container<T> {
            item: T => 0;
        }

        impl<T> Container<T> {
            fn get_item(self) -> T {
                return self.item;
            }
        }

        @entry
        fn main() {
            let c: Container<int> => Container::<int>(item => 99);
            output <- c.get_item();
        }
    "#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["99"]);
}
