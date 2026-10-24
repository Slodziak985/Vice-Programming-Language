use vice::run_source;

#[test]
fn runtime_oop_nested_data() {
    let source = r#"
data Point {
    x: int => 0;
    y: int => 0;
}

data Circle {
    center: Point => Point();
    radius: int => 1;
}

impl Circle {
    fn set_radius(mut self, r: int) -> bool {
        self.radius = r;
        return true;
    }
}

@entry
fn main() {
    mut c: Circle => Circle();
    c.center.x = 10;
    c.set_radius(5);
    output <- c.center.x;
    output <- c.radius;
}
"#;
    let (_, out) = run_source(source, vec![]).unwrap();
    assert_eq!(out, vec!["10", "5"]);
}

#[test]
fn runtime_oop_array_of_data() {
    let source = r#"
data Enemy {
    hp: int => 100;
}

impl Enemy {
    fn damage(mut self, amount: int) -> bool {
        self.hp = self.hp - amount;
        return true;
    }
}

@entry
fn main() {
    mut enemies: Enemy[] => [Enemy(), Enemy()];
    enemies[0].damage(20);
    output <- enemies[0].hp;
    output <- enemies[1].hp;
}
"#;
    let (_, out) = run_source(source, vec![]).unwrap();
    assert_eq!(out, vec!["80", "100"]);
}

#[test]
fn runtime_oop_recursive_method() {
    let source = r#"
data Counter {
    val: int => 0;
}

impl Counter {
    fn count(mut self, to: int) -> bool {
        if (self.val >= to) {
            return true;
        }
        self.val = self.val + 1;
        return self.count(to);
    }
}

@entry
fn main() {
    mut c: Counter => Counter();
    c.count(5);
    output <- c.val;
}
"#;
    let (_, out) = run_source(source, vec![]).unwrap();
    assert_eq!(out, vec!["5"]);
}

#[test]
fn runtime_pattern_matching_advanced() {
    let source = r#"
@entry
fn main() {
    let score => 85;
    let grade => match score {
        90..100 => "A",
        80..89 => "B",
        70..79 => "C",
        _ => "F"
    };
    output <- grade;
}
"#;
    let (_, out) = run_source(source, vec![]).unwrap();
    assert_eq!(out, vec!["B"]);
}

#[test]
fn runtime_match_multiple_types() {
    let source = r#"
@entry
fn main() {
    // Just testing that match can handle expressions
    let val => 10;
    let result => match val {
        5 => "five",
        10 => "ten",
        _ => "unknown"
    };
    output <- result;
}
"#;
    let (_, out) = run_source(source, vec![]).unwrap();
    assert_eq!(out, vec!["ten"]);
}
