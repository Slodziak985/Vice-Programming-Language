# Vice 0.2.0 — Architecture & Runtime Specification

> Canonical architectural documentation for the Vice programming language compiler and virtual machine runtime.

---

## 1. High-Level Pipeline

Vice compiles source code to an internal bytecode representation and executes it in an isolated, stack-based Virtual Machine:

```
[ Source Code *.vice ]
         │
         ▼
 1. Loader (Resolves `use` statements, detects import cycles, enforces project root limits)
         │
         ▼
 2. Lexer (Produces token stream with precise spans: line, column, length)
         │
         ▼
 3. Parser (Constructs the Abstract Syntax Tree - AST)
         │
         ▼
 4. Semantic Analyzer (Type-checking, trait contract resolution, loop termination & effect validation)
         │
         ▼
 5. Bytecode Compiler (Generates executable function Chunks)
         │
         ▼
 6. Verifier (Static bytecode verification: jump target bounds, stack balance, instruction validity)
         │
         ▼
 7. Stack VM (Stack-based execution: call frames, value stack, memory safety, sandboxing)
         │
         ▼
[ Execution Result / Output Buffer (`emit`) ]
```

---

## 2. Core Subsystems

### 2.1 Loader (`src/lib.rs`)
- Recursively resolves `use "<path>";` statements relative to the importing file.
- Detects circular dependencies and reports exact import chain cycles.
- Enforces sandbox security: prevents reading files outside the project root directory.

### 2.2 Lexer & Tokens (`src/lexer.rs`, `src/token.rs`)
- Tokenizes keywords, literals (strings, floats, ints, booleans), identifiers, and operators.
- Implements the **Three Arrows** lexical distinction:
  - `=>` (`DoubleArrow`): Variable binding, match arms, parameter naming.
  - `<-` (`LeftArrow`): Stream output (`emit`), test failure (`fail`).
  - `->` (`RightArrow`): Return type specifier, stream input (`input`).
- Maintains exact source spans (`Span { line, col, len, source_id }`) for compiler diagnostics.

### 2.3 Abstract Syntax Tree (`src/ast.rs`, `src/parser.rs`)
- Hand-crafted recursive descent parser with precedence climbing for binary expressions.
- Top-level items: `fn`, `data` (structs), `enum`, `trait`, `impl`.
- Preserves comments (`#` and `//`) to power the built-in formatter (`vice fmt`).

### 2.4 Semantic Analyzer (`src/semantic.rs`)
- Symbol resolution across lexical scopes and imported modules.
- Type checking: validates numeric promotions, struct field types, array homogeneity, and return types.
- Trait contract enforcement: verifies that all required trait methods are implemented with matching signatures.
- Control flow & loop termination analysis: validates that non-void functions terminate and return across all branches.

### 2.5 Bytecode Compiler & Verifier (`src/vm/compiler.rs`)
- Emits linear bytecode chunks per function.
- Bytecode instructions cover: stack push/pop, arithmetic, local load/store, jumps (`Jump`, `JumpIfFalse`), function calls, and module dispatch.
- **Verifier**: Scans the compiled chunk before execution. Ensures all jump offsets land on valid instruction boundaries and call frames do not exceed runtime stack quotas.

### 2.6 Virtual Machine Runtime (`src/vm/machine.rs`)
- Stack-based execution engine with explicit call frames.
- **Value Semantics**: `Value::Array` and `Value::Data` enforce copy-on-write / value semantics. Mutating an array or structure creates an updated copy, eliminating remote side effects.
- **Sandboxing**: File I/O and process arguments are strictly gated through runtime capabilities.

---

## 3. Native Engine Modules

- **`matrix` (`src/matrix.rs`)**: 2D tensor operations inspired by NumPy (multiplication, transposition, dot products, shapes, row/col sum, reshape).
- **`sqlite` (`src/db_sqlite.rs`)**: In-memory and file-based relational SQLite database transactions.
- **`ffi` (`src/ffi.rs`)**: Dynamic C-ABI symbol loading and invocation (`.dll`, `.so`) via `libloading`.
- **`crypto` (`src/crypto.rs`)**: SHA-256 cryptographic hashing and Base64 encoding/decoding.
- **`regex` (`src/reg_engine.rs`)**: Regular expression matching, captures, and search.
- **`desktop` (`src/ui.rs`)**: Hardware-accelerated windowing using `wgpu` and `winit`.
- **`audio` (`src/stdlib.rs`)**: Sound playback powered by `rodio`.

---

## 4. Manifest & Project Structure

Vice projects are configured via `vice.toml`:

```toml
[package]
name = "my_app"
version = "0.1.0"
entry = "src/main.vice"
language = "0.2.0"

[paths]
std = "std"

[features]
desktop = false
```

- Entry points are designated with `@entry fn main() { ... }`.
- Test suites are designated with `@test fn test_name() { ... }` and executed via `vice test`.

---

**Author**: Slodziak985  
**License**: MIT
