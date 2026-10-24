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
fn test_enum_basic() {
    let source = "
    enum Color {
        Red,
        Green,
        Blue
    }
    
    @autocheck
    @entry
    fn main() {
        let c => Color.Red;
        output <- c;
    }
    ";
    assert_eq!(run_test(source), "Color.Red");
}

#[test]
fn test_enum_assignment() {
    let source = "
    enum State {
        Idle,
        Running,
        Finished
    }
    
    @autocheck
    @entry
    fn main() {
        mut current => State.Idle;
        current = State.Running;
        output <- current;
    }
    ";
    assert_eq!(run_test(source), "State.Running");
}

#[test]
fn test_enum_equality() {
    let source = "
    enum Direction { North, South, East, West }
    
    @autocheck
    @entry
    fn main() {
        let d1 => Direction.North;
        let d2 => Direction.North;
        let d3 => Direction.South;
        
        output <- d1 == d2;
        output <- d1 == d3;
        output <- d1 != d3;
    }
    ";
    assert_eq!(run_test(source), "true\nfalse\ntrue");
}

#[test]
fn test_enum_in_function() {
    let source = "
    enum Mode { Light, Dark }
    
    fn is_dark_mode(mode: Mode) -> bool {
        return mode == Mode.Dark;
    }
    
    @autocheck
    @entry
    fn main() {
        output <- is_dark_mode(Mode.Light);
        output <- is_dark_mode(Mode.Dark);
    }
    ";
    assert_eq!(run_test(source), "false\ntrue");
}

#[test]
fn test_enum_in_data() {
    let source = "
    enum Status { Alive, Dead }
    
    data Player {
        name: string => \"Unknown\";
        status: Status => Status.Alive;
    }
    
    @autocheck
    @entry
    fn main() {
        let p => Player();
        output <- p.status;
    }
    ";
    assert_eq!(run_test(source), "Status.Alive");
}

#[test]
fn test_enum_array() {
    let source = "
    enum Action { Jump, Shoot, Run }
    
    @autocheck
    @entry
    fn main() {
        let sequence => [Action.Run, Action.Jump, Action.Shoot];
        output <- sequence[1];
    }
    ";
    assert_eq!(run_test(source), "Action.Jump");
}

#[test]
fn test_enum_map() {
    let source = "
    enum Day { Mon, Tue, Wed }
    
    @autocheck
    @entry
    fn main() {
        let schedule => {
            Day.Mon: \"Work\",
            Day.Tue: \"Gym\",
            Day.Wed: \"Rest\"
        };
        
        output <- schedule[Day.Tue];
    }
    ";
    assert_eq!(run_test(source), "Gym");
}

#[test]
fn test_enum_match() {
    let source = "
    enum Level { Low, Medium, High }
    
    fn get_multiplier(level: Level) -> int {
        return match level {
            Level.Low => 1,
            Level.Medium => 2,
            Level.High => 3
        };
    }
    
    @autocheck
    @entry
    fn main() {
        output <- get_multiplier(Level.High);
        output <- get_multiplier(Level.Medium);
    }
    ";
    assert_eq!(run_test(source), "3\n2");
}

#[test]
fn test_enum_invalid_variant() {
    let source = "
    enum State { On, Off }
    @autocheck
    @entry
    fn main() {
        let s => State.Unknown;
    }
    ";
    assert!(run_test_error(source));
}

#[test]
fn test_enum_invalid_assignment() {
    let source = "
    enum State { On, Off }
    enum Mode { Fast, Slow }
    
    @autocheck
    @entry
    fn main() {
        mut s => State.On;
        s = Mode.Fast;
    }
    ";
    assert!(run_test_error(source));
}
