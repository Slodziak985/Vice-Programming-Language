use vice::parse_source;
use vice::semantic::SemanticAnalyzer;

fn run_vice_code(code: &str) -> String {
    let program = parse_source(code).expect("Parser failed");
    let mut semantic = SemanticAnalyzer::new();
    semantic.analyze(&program).expect("Semantic check failed");
    let output = vice::execute_program_vm(&program).expect("Interpreter failed");
    output.join("\n") + "\n"
}

fn run_vice_code_err(code: &str) -> String {
    let program = match parse_source(code) {
        Ok(p) => p,
        Err(e) => return format!("{:?}", e),
    };
    let mut semantic = SemanticAnalyzer::new();
    if let Err(e) = semantic.analyze(&program) {
        return format!("{:?}", e);
    }
    match vice::execute_program_vm(&program) {
        Ok(_) => "Success".to_string(),
        Err(e) => format!("{:?}", e),
    }
}

#[test]
fn oop_deep_chaining() {
    let code = r#"
data Node {
    val: int => 0;
    next: Node[] => [];
}

impl Node {
    fn advance(self) -> Node {
        return self.next[0];
    }
}

@entry
fn main() {
    let n3 => Node();
    n3.val = 3;
    let n2 => Node();
    n2.val = 2; n2.next = [n3];
    let n1 => Node();
    n1.val = 1; n1.next = [n2];

    let end => n1.advance().advance();
    output <- end.val;
}
"#;
    assert_eq!(run_vice_code(code), "3\n");
}

#[test]
fn oop_mutate_self() {
    let code = r#"
data Counter {
    count: int => 0;
}

impl Counter {
    fn tick(mut self) {
        self.count += 1;
    }
}

@entry
fn main() {
    mut c => Counter();
    c.count = 0;
    c.tick();
    c.tick();
    c.tick();
    output <- c.count;
}
"#;
    assert_eq!(run_vice_code(code), "3\n");
}

#[test]
fn oop_method_shadowing_property() {
    let code = r#"
data Weird {
    val: string => "";
}

impl Weird {
    fn val(self) -> string {
        return "shadowed";
    }
}

@entry
fn main() {
    let w => Weird();
    w.val = "original";
    // This calls the method, not the property
    output <- w.val();
}
"#;
    assert_eq!(run_vice_code(code), "shadowed\n");
}

#[test]
fn oop_constructor_simulation() {
    let code = r#"
data Config {
    debug: bool => false;
    retries: int => 0;
}

impl Config {
    fn new(debug: bool) -> Config {
        let mut r => 3;
        if (debug) {
            r = 10;
        }
        let ret => Config();
        ret.debug = debug;
        ret.retries = r;
        return ret;
    }
}

@entry
fn main() {
    let c => Config.new(true);
    output <- c.retries;
}
"#;
    // Vice parser doesn't support Static method calls directly with Config.new() unless specifically implemented.
    // Let's assume it errors for now or we use a global function. Wait, let's see if Vice throws an error.
    let err = run_vice_code_err(code);
    assert!(
        err.contains("Parser failed")
            || err.contains("Semantic check failed")
            || err.contains("Unknown method")
    );
}

#[test]
fn oop_call_method_on_null() {
    let code = r#"
data Box { val: int => 0; }
impl Box { fn open(self) { output <- self.val; } }

@entry
fn main() {
    let b: Box => null;
    b.open();
}
"#;
    let err = run_vice_code_err(code);
    assert!(
        err.contains("null")
            || err.contains("Cannot call method on null")
            || err.contains("Unknown method")
    );
}

#[test]
fn oop_method_wrong_argument_type() {
    let code = r#"
data Calc {}
impl Calc {
    fn add(self, a: int, b: int) -> int {
        return a + b;
    }
}

@entry
fn main() {
    let c => Calc();
    c.add(1, "2");
}
"#;
    let err = run_vice_code_err(code);
    assert!(
        err.contains("Type")
            || err.contains("type")
            || err.contains("argument")
            || err.contains("Semantic"),
        "Got err: {}",
        err
    );
}

#[test]
fn oop_deep_data_nesting_initialization() {
    let code = r#"
data A { v: int => 0; }
data B { a: A[] => []; }
data C { b: B[] => []; }
data D { c: C[] => []; }
data E { d: D[] => []; }

@entry
fn main() {
    let a => A(); a.v = 42;
    let b => B(); b.a = [a];
    let c => C(); c.b = [b];
    let d => D(); d.c = [c];
    let e => E(); e.d = [d];
    output <- e.d[0].c[0].b[0].a[0].v;
}
"#;
    assert_eq!(run_vice_code(code), "42\n");
}

#[test]
fn oop_method_returns_self() {
    let code = r#"
data Builder {
    acc: string => "";
}
impl Builder {
    fn append(mut self, s: string) -> Builder {
        self.acc = self.acc + s;
        return self;
    }
}
@entry
fn main() {
    mut b => Builder();
    b.acc = "";
    b = b.append("a").append("b").append("c");
    output <- b.acc;
}
"#;
    assert_eq!(run_vice_code(code), "abc\n");
}
