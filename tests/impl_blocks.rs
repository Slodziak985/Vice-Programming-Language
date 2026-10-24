use vice::parse_source;
use vice::run_source;
use vice::semantic::SemanticAnalyzer;

#[test]
fn impl_block_basic_method_call() {
    let source = r#"
data User {
    score: int => 0;
}
impl User {
    fn update_score(mut self, points: int) {
        self.score = self.score + points;
    }
}
@entry
fn main() {
    mut user: User => User();
    user.update_score(10);
    output <- user.score;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["10"]);
}

#[test]
fn impl_block_multiple_methods() {
    let source = r#"
data Calc {
    value: int => 0;
}
impl Calc {
    fn add(mut self, v: int) {
        self.value = self.value + v;
    }
    fn sub(mut self, v: int) {
        self.value = self.value - v;
    }
    fn get(self) -> int {
        return self.value;
    }
}
@entry
fn main() {
    mut c: Calc => Calc();
    c.add(5);
    c.sub(2);
    output <- c.get();
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn impl_block_method_returns_value() {
    let source = r#"
data User {
    score: int => 5;
}
impl User {
    fn is_winner(self) -> bool {
        return self.score >= 10;
    }
}
@entry
fn main() {
    mut user: User => User();
    output <- user.is_winner();
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["false"]);
}

#[test]
fn impl_block_chaining_mutations() {
    let source = r#"
data Point {
    x: int => 0;
    y: int => 0;
}
impl Point {
    fn move_x(mut self, dx: int) {
        self.x = self.x + dx;
    }
    fn move_y(mut self, dy: int) {
        self.y = self.y + dy;
    }
}
@entry
fn main() {
    mut p: Point => Point();
    p.move_x(3);
    p.move_y(4);
    output <- p.x, p.y;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["3 4"]);
}

#[test]
fn impl_block_semantic_error_wrong_arg_count() {
    let source = r#"
data User { score: int => 0; }
impl User {
    fn set(mut self, s: int) { self.score = s; }
}
@entry
fn main() {
    mut u: User => User();
    u.set();
}
"#;
    let error = parse_source(source)
        .and_then(|p| SemanticAnalyzer::new().analyze(&p))
        .unwrap_err();
    assert!(error.message.contains("expects 1 arguments, got 0"));
}

#[test]
fn impl_block_semantic_error_wrong_arg_type() {
    let source = r#"
data User { score: int => 0; }
impl User {
    fn set(mut self, s: int) { self.score = s; }
}
@entry
fn main() {
    mut u: User => User();
    u.set("bad");
}
"#;
    let error = parse_source(source)
        .and_then(|p| SemanticAnalyzer::new().analyze(&p))
        .unwrap_err();
    assert!(error.message.contains("expected int, got string"));
}

#[test]
fn impl_block_with_named_args() {
    let source = r#"
data Player {
    health: int => 100;
}
impl Player {
    fn heal(mut self, amount: int) {
        self.health = self.health + amount;
    }
}
@entry
fn main() {
    mut p: Player => Player();
    p.heal(amount => 50);
    output <- p.health;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["150"]);
}

#[test]
fn impl_block_unknown_method_error() {
    let source = r#"
data Player {
    health: int => 100;
}
impl Player {
    fn heal(mut self, amount: int) {
        self.health = self.health + amount;
    }
}
@entry
fn main() {
    mut p: Player => Player();
    p.damage(50);
}
"#;
    let error = parse_source(source)
        .and_then(|p| SemanticAnalyzer::new().analyze(&p))
        .unwrap_err();
    assert!(error.message.contains("Unknown method 'damage' for Player"));
}

#[test]
fn impl_block_no_self_param() {
    let source = r#"
data Math {}
impl Math {
    fn pi() -> float {
        return 3.14;
    }
}
@entry
fn main() {
    mut m: Math => Math();
    output <- m.pi();
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["3.14"]);
}

#[test]
fn impl_block_nested_data() {
    let source = r#"
data Stats {
    level: int => 1;
}
data Hero {
    stats: Stats => Stats();
}
impl Hero {
    fn level_up(mut self) {
        self.stats.level = self.stats.level + 1;
    }
}
@entry
fn main() {
    mut h: Hero => Hero();
    h.level_up();
    output <- h.stats.level;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["2"]);
}
