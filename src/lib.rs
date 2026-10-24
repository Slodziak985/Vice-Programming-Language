pub mod ast;
#[cfg(feature = "audio")]
pub mod audio;
pub mod crypto;
pub mod db_sqlite;
pub mod error;
pub mod ffi;
pub mod formatter;
pub mod installer;
pub mod lexer;
pub mod matrix;
pub mod native_registry;
pub mod net_http;
pub mod parser;
pub mod pkg;
pub mod reg_engine;
pub mod semantic;
pub mod stdlib;
pub mod token;
pub mod types;
#[cfg(feature = "desktop")]
pub mod ui;
#[cfg(not(feature = "desktop"))]
#[path = "ui_stub.rs"]
pub mod ui;
pub mod value;
pub mod vm;

use crate::ast::Program;
use crate::error::{ViceError, ViceErrorType};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::value::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn parse_source(source: &str) -> Result<Program, ViceError> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

pub fn parse_source_named(source: &str, file: &Path) -> Result<Program, ViceError> {
    let file_name = file.display().to_string();
    let mut program = parse_source(source).map_err(|error| error.with_file(file_name.clone()))?;
    program.assign_source(0);
    program.sources.push(file_name);
    Ok(program)
}

pub fn load_program(path: &Path) -> Result<Program, ViceError> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    let mut sources = Vec::new();
    let mut program = load_program_inner(path, &mut visited, &mut stack, &mut sources)?;
    program.sources = sources;
    validate_merged_symbols(&program)?;
    Ok(program)
}

pub fn run_source(
    source: &str,
    input: Vec<String>,
) -> Result<(Option<Value>, Vec<String>), ViceError> {
    let program = parse_source(source)?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)?;
    let compiler = crate::vm::compiler::Compiler::new();
    let compiled = compiler.compile(&program)?;
    let mut machine = crate::vm::machine::Vm::new_with_input(input);
    let result = machine.interpret(compiled)?;
    Ok((Some(result.value), result.output))
}

pub fn execute_program_vm(program: &Program) -> Result<Vec<String>, ViceError> {
    let compiler = crate::vm::compiler::Compiler::new();
    let compiled = compiler.compile(program)?;
    let mut machine = crate::vm::machine::Vm::new();
    Ok(machine.interpret(compiled)?.output)
}

pub fn check_program(path: &Path) -> Result<Program, ViceError> {
    let program = load_program(path)?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)?;
    crate::vm::compiler::Compiler::new().compile(&program)?;
    Ok(program)
}

pub fn format_error_with_source(error: &ViceError, source: &str) -> String {
    let type_str = match error.error_type {
        ViceErrorType::Syntax => "Syntax",
        ViceErrorType::Compile => "Compile",
        ViceErrorType::Runtime => "Runtime",
        ViceErrorType::Semantic => "Semantic",
        ViceErrorType::ReturnEarly => "ReturnEarly",
    };
    let file = error.file.as_deref().unwrap_or("<source>");
    let mut rendered = format!(
        "{}:{}:{}: [{} Error] {}",
        file, error.line, error.column, type_str, error.message
    );

    if error.line > 0 {
        if let Some(line_text) = source.lines().nth(error.line.saturating_sub(1)) {
            rendered.push('\n');
            rendered.push_str(line_text);
            rendered.push('\n');
            let spaces = " ".repeat(error.column.saturating_sub(1));
            let carets = "^".repeat(error.length.max(1));
            rendered.push_str(&spaces);
            rendered.push_str(&carets);
        }
    }

    for frame in &error.stack_trace {
        rendered.push_str(&format!(
            "\n  at {} ({}:{}:{})",
            frame.function,
            frame.file.as_deref().unwrap_or("<source>"),
            frame.line,
            frame.column
        ));
    }

    rendered
}

pub fn format_error_for_file(error: &ViceError, fallback_path: &Path) -> String {
    let source_path = error
        .file
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| fallback_path.to_path_buf());
    let source = fs::read_to_string(&source_path).unwrap_or_default();
    let display_error = if error.file.is_none() {
        let mut error = error.clone();
        error.file = Some(fallback_path.display().to_string());
        error
    } else {
        error.clone()
    };
    format_error_with_source(&display_error, &source)
}

fn load_program_inner(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
    sources: &mut Vec<String>,
) -> Result<Program, ViceError> {
    let canonical = fs::canonicalize(path).map_err(|err| {
        ViceError::new(
            ViceErrorType::Runtime,
            &format!("Cannot open '{}': {}", path.display(), err),
            0,
        )
    })?;
    if let Some(root) = crate::stdlib::sandbox_root() {
        if !canonical.starts_with(&root) && !is_trusted_std_path(&canonical) {
            return Err(ViceError::new(
                ViceErrorType::Runtime,
                &format!(
                    "Sandbox denied import '{}'; root is '{}'",
                    canonical.display(),
                    root.display()
                ),
                0,
            ));
        }
    }

    if let Some(position) = stack.iter().position(|entry| entry == &canonical) {
        let cycle = stack[position..]
            .iter()
            .chain(std::iter::once(&canonical))
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(ViceError::new(
            ViceErrorType::Runtime,
            &format!("Import cycle detected: {}", cycle),
            0,
        ));
    }

    if !visited.insert(canonical.clone()) {
        return Ok(empty_program());
    }

    stack.push(canonical.clone());
    let source = fs::read_to_string(&canonical).map_err(|err| {
        ViceError::new(
            ViceErrorType::Runtime,
            &format!("Cannot read '{}': {}", canonical.display(), err),
            0,
        )
    })?;
    let source_id = sources.len();
    sources.push(canonical.display().to_string());
    let mut program =
        parse_source(&source).map_err(|error| error.with_file(canonical.display().to_string()))?;
    program.assign_source(source_id);
    let imports = program.imports.clone();
    program.imports.clear();

    let base = canonical.parent().unwrap_or_else(|| Path::new("."));
    let mut merged = empty_program();

    for import in imports {
        let import_path = resolve_import_path(base, &import.path);
        let imported =
            load_program_inner(&import_path, visited, stack, sources).map_err(|error| {
                if error.line == 0 {
                    ViceError::at(
                        error.error_type,
                        &error.message,
                        Some(canonical.display().to_string()),
                        import.span.line,
                        import.span.column,
                        import.span.length,
                    )
                } else {
                    error
                }
            })?;
        merged.data_declarations.extend(imported.data_declarations);
        merged.functions.extend(imported.functions);
        merged.impls.extend(imported.impls);
        merged.top_level.extend(imported.top_level);
    }

    merged
        .data_declarations
        .extend(program.data_declarations.clone());
    merged.functions.extend(program.functions.clone());
    merged.impls.extend(program.impls.clone());
    merged.top_level.extend(program.top_level);
    stack.pop();
    Ok(merged)
}

fn resolve_import_path(base: &Path, import_path: &str) -> PathBuf {
    let local = base.join(import_path);
    if local.exists() {
        return local;
    }
    let local_vice = base.join(format!("{}.vice", import_path));
    if local_vice.exists() {
        return local_vice;
    }
    let module_path = base.join("modules").join(import_path);
    if module_path.exists() {
        return module_path;
    }
    let module_vice = base.join("modules").join(format!("{}.vice", import_path));
    if module_vice.exists() {
        return module_vice;
    }
    let module_main = base.join("modules").join(import_path).join("main.vice");
    if module_main.exists() {
        return module_main;
    }

    if !is_std_import(import_path) {
        return local;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let packaged = exe_dir.join(import_path);
            if packaged.exists() {
                return packaged;
            }
        }
    }

    Path::new(env!("CARGO_MANIFEST_DIR")).join(import_path)
}

fn is_std_import(import_path: &str) -> bool {
    import_path == "std" || import_path.starts_with("std/")
}

fn is_trusted_std_path(path: &Path) -> bool {
    let repo_std = Path::new(env!("CARGO_MANIFEST_DIR")).join("std");
    if let Ok(repo_std) = repo_std.canonicalize() {
        if path.starts_with(repo_std) {
            return true;
        }
    }

    if path.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("std")) {
        return true;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let packaged_std = exe_dir.join("std");
            if let Ok(packaged_std) = packaged_std.canonicalize() {
                if path.starts_with(packaged_std) {
                    return true;
                }
            }
            if path.starts_with(exe_dir.join("std")) {
                return true;
            }
        }
    }
    false
}

fn empty_program() -> Program {
    Program {
        impls: Vec::new(),
        imports: Vec::new(),
        functions: Vec::new(),
        data_declarations: Vec::new(),
        top_level: Vec::new(),
        sources: Vec::new(),
    }
}

fn validate_merged_symbols(program: &Program) -> Result<(), ViceError> {
    use crate::ast::{StmtKind, TopLevel};
    use std::collections::HashMap;

    let mut symbols: HashMap<String, crate::ast::Span> = HashMap::new();
    let mut insert = |kind: &str, name: &str, span: crate::ast::Span| {
        let key = format!("{}:{}", kind, name);
        if symbols.insert(key, span).is_some() {
            let file = program.sources.get(span.source_id).cloned();
            return Err(ViceError::at(
                ViceErrorType::Semantic,
                &format!("Duplicate {} declaration: {}", kind, name),
                file,
                span.line,
                span.column,
                span.length,
            ));
        }
        Ok(())
    };

    for function in &program.functions {
        insert("function", &function.name, function.span)?;
    }
    for statement in &program.data_declarations {
        if let StmtKind::DataDeclaration { name, .. } = &statement.kind {
            insert("data", name, statement.span)?;
        }
    }
    for top in &program.top_level {
        match top {
            TopLevel::EnumDef(statement) => {
                if let StmtKind::EnumDef { name, .. } = &statement.kind {
                    insert("enum", name, statement.span)?;
                }
            }
            TopLevel::TraitDef(statement) => {
                if let StmtKind::TraitDef { name, .. } = &statement.kind {
                    insert("trait", name, statement.span)?;
                }
            }
            _ => {}
        }
    }
    for implementation in &program.impls {
        let target = implementation.target.to_string();
        for method in &implementation.methods {
            insert(
                "method",
                &format!("{}::{}", target, method.name),
                method.span,
            )?;
        }
    }
    Ok(())
}
