# Vice 0.2.0 — Developer Guide

> Official guide for writing applications, tests, and libraries in Vice.

---

## 1. Quick Start

### Installation
Download pre-compiled binaries from the GitHub Releases tab, run `ViceInstaller.exe`, or place `vice.exe` into your system `PATH`.

```bash
# Verify installation
vice --version
```

### Creating a New Project
```bash
vice new my_project
cd my_project
vice run .
```

---

## 2. Language Fundamentals

### Variables and Mutability
- Immutable declaration: `let <name> => <value>;`
- Mutable declaration: `mut <name> => <value>;`
- Reassignment: `<name> = <new_value>;`

```vice
let pi => 3.14159;
mut counter => 0;
counter = counter + 1;
```

### Data Flow & Output
Use `emit <-` to output values to stdout:

```vice
emit <- "Hello, Vice!";
emit <- counter;
```

### Functions
Functions are declared using `fn` with arrow return types:

```vice
fn add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    let result => add(10, 20);
    emit <- result;
}
```

### Data Structures (`data`)
`data` blocks define typed structs with default field initializers:

```vice
data Player {
    name: string => "Hero";
    score: int => 0;
    alive: bool => true;
}

fn level_up(player: Player) -> Player {
    mut p = player;
    p.score = p.score + 100;
    return p;
}
```

### Pattern Matching (`match`)
```vice
enum Status {
    Ok,
    Pending,
    Err(string)
}

fn handle(status: Status) -> string {
    return match (status) {
        Status.Ok => "All good",
        Status.Pending => "Please wait",
        Status.Err(msg) => msg,
        _ => "Unknown"
    };
}
```

---

## 3. The `matrix` Module (Linear Algebra & AI)

Vice includes built-in tensor operations for scientific and machine learning workflows:

```vice
fn main() {
    # 2x2 matrix
    mut m1 = matrix.from_array([1.0, 2.0, 3.0, 4.0], 2, 2);
    
    # 2x1 vector
    mut v = matrix.from_array([0.5, 1.5], 2, 1);
    
    # Matrix multiplication: Y = M1 * V
    mut result = matrix.mul(m1, v);
    emit <- matrix.to_array(result);
}
```

---

## 4. Testing Framework

Mark test functions with `@test`. Use `fail <- "message"` to signal failure:

```vice
@test
fn verify_addition() {
    let sum => 2 + 2;
    if (sum != 4) {
        fail <- "Expected 2 + 2 to equal 4";
    }
}
```

Run tests from the CLI:
```bash
vice test .
# Output:
# Running tests...
# [PASS] verify_addition
```

---

## 5. Tooling Reference

| Command | Purpose |
|---|---|
| `vice run <file\|dir>` | Compiles and executes code in the VM |
| `vice check <file\|dir>` | Static verification without execution |
| `vice test <dir>` | Discovers and executes all `@test` functions |
| `vice fmt <file\|dir>` | Formats code in place using canonical 4-space indentation |
| `vice schema` | Generates full machine JSON schema of available runtime APIs |
| `vice migrate` | Upgrades legacy v0.1.1 projects to v0.2.0 |

---

**Author**: Slodziak985  
**License**: MIT
