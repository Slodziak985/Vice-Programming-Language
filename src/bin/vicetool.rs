use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use vice::ast::{CallArg, Expr, ExprKind, Program, Stmt, StmtKind, TopLevel};
use vice::{format_error_for_file, load_program};

#[derive(Debug)]
struct UiElement {
    kind: String,
    id: Option<String>,
    line: usize,
    editable: bool,
}

#[derive(Debug)]
struct VamProject {
    root: PathBuf,
    entry: PathBuf,
    ui_elements: Vec<UiElement>,
    scenes: Vec<PathBuf>,
    ui_layouts: Vec<PathBuf>,
    assets: Vec<PathBuf>,
    created: Vec<PathBuf>,
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let code = match args.as_slice() {
        [cmd, tool] if cmd == "run" && tool == "VAM" => run_vam(false),
        [cmd, tool, flag] if cmd == "run" && tool == "VAM" && flag == "--json" => run_vam(true),
        _ => {
            eprintln!("Usage:");
            eprintln!("  vicetool run VAM [--json]");
            2
        }
    };
    if code != 0 {
        std::process::exit(code);
    }
}

fn run_vam(json: bool) -> i32 {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("vicetool: cannot read current directory: {}", error);
            return 1;
        }
    };
    let entry = match resolve_entry_path(&cwd) {
        Ok(entry) => entry,
        Err(message) => {
            eprintln!("vicetool VAM: {}", message);
            return 1;
        }
    };
    let program = match load_program(&entry) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{}", format_error_for_file(&error, &entry));
            return 1;
        }
    };
    let elements = collect_ui_elements(&program);
    let project = match prepare_vam_project(cwd, entry, elements) {
        Ok(project) => project,
        Err(message) => {
            eprintln!("vicetool VAM: {}", message);
            return 1;
        }
    };

    if json {
        let elements = project
            .ui_elements
            .iter()
            .map(|element| {
                serde_json::json!({
                    "kind": element.kind,
                    "id": element.id,
                    "line": element.line,
                    "editable": element.editable,
                })
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            serde_json::json!({
                "tool": "VAM",
                "mode": "full_suite_foundation",
                "project": project.root.display().to_string(),
                "entry": project.entry.display().to_string(),
                "elements": elements,
                "scenes": paths_json(&project.scenes),
                "ui_layouts": paths_json(&project.ui_layouts),
                "assets": paths_json(&project.assets),
                "created": paths_json(&project.created),
                "panels": [
                    "Project Dashboard",
                    "Asset Browser",
                    "Scene Tree",
                    "Inspector",
                    "UI Designer",
                    "Tilemap Editor",
                    "Game Scene Editor",
                    "Play Preview",
                    "Profiler Panel",
                    "Packaging Panel"
                ],
            })
        );
        return 0;
    }

    println!("Vice App Manager (VAM)");
    println!("Mode: full suite foundation");
    println!("Project: {}", project.root.display());
    println!("Entry: {}", project.entry.display());
    println!("Panels: Dashboard, Assets, Scene Tree, Inspector, UI Designer, Tilemap, Preview, Profiler, Packaging");
    println!("Scenes: {}", project.scenes.len());
    for path in &project.scenes {
        println!("- scene {}", relative_display(&project.root, path));
    }
    println!("UI layouts: {}", project.ui_layouts.len());
    for path in &project.ui_layouts {
        println!("- vui {}", relative_display(&project.root, path));
    }
    println!("Assets: {}", project.assets.len());
    for path in &project.assets {
        println!("- asset {}", relative_display(&project.root, path));
    }
    if !project.created.is_empty() {
        println!("Created editor files:");
        for path in &project.created {
            println!("- {}", relative_display(&project.root, path));
        }
    }
    println!("UI elements: {}", project.ui_elements.len());
    for element in project.ui_elements {
        let id = element
            .id
            .unwrap_or_else(|| "<dynamic/read-only>".to_string());
        let mode = if element.editable {
            "editable"
        } else {
            "read-only"
        };
        println!(
            "- ui.{} id={} line={} {}",
            element.kind, id, element.line, mode
        );
    }
    println!("Window Manager module: AST-backed inspection ready.");
    println!("Scene/UI data model: .vscene/.vui JSON ready.");
    println!("Play Preview/Profiler/Packaging panels: scaffolded for Vice GUI implementation.");
    0
}

fn prepare_vam_project(
    root: PathBuf,
    entry: PathBuf,
    ui_elements: Vec<UiElement>,
) -> Result<VamProject, String> {
    let mut created = Vec::new();
    for dir in ["assets", "scenes", "ui", "build"] {
        let path = root.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|error| format!("cannot create '{}': {}", path.display(), error))?;
            created.push(path);
        }
    }

    ensure_json_file(
        &root,
        "assets/assets.json",
        default_assets_manifest(),
        &mut created,
    )?;
    ensure_json_file(&root, "scenes/main.vscene", default_scene(), &mut created)?;
    ensure_json_file(&root, "ui/main.vui", default_ui_layout(), &mut created)?;
    ensure_json_file(
        &root,
        "vam.project.json",
        default_editor_project(),
        &mut created,
    )?;

    Ok(VamProject {
        scenes: collect_files(&root.join("scenes"), "vscene"),
        ui_layouts: collect_files(&root.join("ui"), "vui"),
        assets: collect_asset_files(&root.join("assets")),
        root,
        entry,
        ui_elements,
        created,
    })
}

fn ensure_json_file(
    root: &Path,
    relative: &str,
    value: serde_json::Value,
    created: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let path = root.join(relative);
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create '{}': {}", parent.display(), error))?;
    }
    let content = serde_json::to_string_pretty(&value)
        .map_err(|error| format!("cannot serialize '{}': {}", path.display(), error))?;
    fs::write(&path, format!("{}\n", content))
        .map_err(|error| format!("cannot write '{}': {}", path.display(), error))?;
    created.push(path);
    Ok(())
}

fn collect_files(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut values = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some(extension) {
                values.push(path);
            }
        }
    }
    values.sort();
    values
}

fn collect_asset_files(root: &Path) -> Vec<PathBuf> {
    let mut values = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                values.push(path);
            }
        }
    }
    values.sort();
    values
}

fn paths_json(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn default_assets_manifest() -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "kind": "assets",
        "textures": [],
        "spritesheets": [],
        "audio": [],
        "fonts": []
    })
}

fn default_scene() -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "kind": "scene",
        "name": "Main Scene",
        "entities": [
            {
                "id": "camera.main",
                "name": "Main Camera",
                "kind": "camera",
                "transform": {"x": 0, "y": 0, "rotation": 0, "scale_x": 1, "scale_y": 1},
                "camera": {"width": 1280, "height": 720},
                "components": []
            }
        ],
        "layers": ["background", "world", "actors", "ui"],
        "settings": {"width": 1280, "height": 720, "background": "#101820"}
    })
}

fn default_ui_layout() -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "kind": "ui_layout",
        "name": "Main UI",
        "windows": [],
        "elements": []
    })
}

fn default_editor_project() -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "kind": "vice_editor_project",
        "assets": "assets/assets.json",
        "scenes": ["scenes/main.vscene"],
        "ui_layouts": ["ui/main.vui"],
        "last_open_scene": "scenes/main.vscene"
    })
}

fn resolve_entry_path(root: &Path) -> Result<PathBuf, String> {
    let manifest = root.join("vice.toml");
    if manifest.is_file() {
        let source = fs::read_to_string(&manifest)
            .map_err(|error| format!("cannot read '{}': {}", manifest.display(), error))?;
        let entry = toml_string_value(&source, "entry").unwrap_or_else(|| "src/main.vice".into());
        return Ok(root.join(entry));
    }

    let legacy = root.join("vice.json");
    if legacy.is_file() {
        let source = fs::read_to_string(&legacy)
            .map_err(|error| format!("cannot read '{}': {}", legacy.display(), error))?;
        let manifest: serde_json::Value = serde_json::from_str(&source)
            .map_err(|error| format!("invalid '{}': {}", legacy.display(), error))?;
        let entry = manifest
            .get("entry")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("src/main.vice");
        return Ok(root.join(entry));
    }

    let fallback = root.join("src").join("main.vice");
    if fallback.is_file() {
        Ok(fallback)
    } else {
        Err("expected vice.toml, vice.json, or src/main.vice in the current project".into())
    }
}

fn toml_string_value(source: &str, key: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let (left, right) = trimmed.split_once('=')?;
        if left.trim() != key {
            continue;
        }
        let value = right.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].to_string());
        }
    }
    None
}

fn collect_ui_elements(program: &Program) -> Vec<UiElement> {
    let mut elements = Vec::new();
    for top in &program.top_level {
        match top {
            TopLevel::Function(function) => {
                for stmt in &function.body {
                    collect_stmt(stmt, &mut elements);
                }
            }
            TopLevel::Impl(implementation) => {
                for method in &implementation.methods {
                    for stmt in &method.body {
                        collect_stmt(stmt, &mut elements);
                    }
                }
            }
            _ => {}
        }
    }
    elements
}

fn collect_stmt(stmt: &Stmt, elements: &mut Vec<UiElement>) {
    match &stmt.kind {
        StmtKind::Output(expressions) | StmtKind::Emit(expressions) => {
            collect_exprs(expressions, elements)
        }
        StmtKind::Variable { value, .. }
        | StmtKind::Assign { value, .. }
        | StmtKind::Return(Some(value))
        | StmtKind::Expression(value)
        | StmtKind::Fail { message: value } => collect_expr(value, elements),
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_expr(condition, elements);
            collect_stmts(then_branch, elements);
            if let Some(else_branch) = else_branch {
                collect_stmts(else_branch, elements);
            }
        }
        StmtKind::While { condition, body } => {
            collect_expr(condition, elements);
            collect_stmts(body, elements);
        }
        StmtKind::ForLoop {
            start, end, body, ..
        } => {
            collect_expr(start, elements);
            collect_expr(end, elements);
            collect_stmts(body, elements);
        }
        StmtKind::ForEach {
            collection, body, ..
        } => {
            collect_expr(collection, elements);
            collect_stmts(body, elements);
        }
        StmtKind::Block(items) => collect_stmts(items, elements),
        StmtKind::Match { condition, arms } => {
            collect_expr(condition, elements);
            for arm in arms {
                collect_exprs(&arm.patterns, elements);
                collect_stmt(&arm.body, elements);
            }
        }
        StmtKind::MemberAssign { entity, value, .. } => {
            collect_expr(entity, elements);
            collect_expr(value, elements);
        }
        StmtKind::IndexAssign {
            entity,
            index,
            value,
        } => {
            collect_expr(entity, elements);
            collect_expr(index, elements);
            collect_expr(value, elements);
        }
        StmtKind::DataDeclaration { fields, .. } => {
            for field in fields {
                collect_expr(&field.default_value, elements);
            }
        }
        StmtKind::Error { args, message } => {
            collect_args(args, elements);
            collect_expr(message, elements);
        }
        StmtKind::DerefAssign { pointer, value } => {
            collect_expr(pointer, elements);
            collect_expr(value, elements);
        }
        StmtKind::Input(_)
        | StmtKind::Return(None)
        | StmtKind::Break
        | StmtKind::Continue
        | StmtKind::EnumDef { .. }
        | StmtKind::TraitDef { .. } => {}
    }
}

fn collect_stmts(stmts: &[Stmt], elements: &mut Vec<UiElement>) {
    for stmt in stmts {
        collect_stmt(stmt, elements);
    }
}

fn collect_exprs(expressions: &[Expr], elements: &mut Vec<UiElement>) {
    for expr in expressions {
        collect_expr(expr, elements);
    }
}

fn collect_args(args: &[CallArg], elements: &mut Vec<UiElement>) {
    for arg in args {
        collect_expr(&arg.value, elements);
    }
}

fn collect_expr(expr: &Expr, elements: &mut Vec<UiElement>) {
    match &expr.kind {
        ExprKind::MethodCall {
            entity,
            method,
            args,
            ..
        } => {
            if matches!(&entity.kind, ExprKind::Identifier(module) if module == "ui") {
                let id = ui_id_arg(args);
                let editable = id.is_some() && ui_has_literal_layout(args);
                elements.push(UiElement {
                    kind: method.clone(),
                    id,
                    line: expr.span.line,
                    editable,
                });
            }
            collect_expr(entity, elements);
            collect_args(args, elements);
        }
        ExprKind::Call { args, .. } => collect_args(args, elements),
        ExprKind::InterpolatedString(parts) | ExprKind::Array(parts) => {
            collect_exprs(parts, elements)
        }
        ExprKind::Map(entries) => {
            for (key, value) in entries {
                collect_expr(key, elements);
                collect_expr(value, elements);
            }
        }
        ExprKind::Unary { right, .. }
        | ExprKind::Try { expr: right }
        | ExprKind::Unwrap { expr: right }
        | ExprKind::Get { entity: right, .. }
        | ExprKind::PropertyCheck { entity: right, .. } => collect_expr(right, elements),
        ExprKind::Binary { left, right, .. }
        | ExprKind::Logical { left, right, .. }
        | ExprKind::Index {
            entity: left,
            index: right,
        }
        | ExprKind::Range {
            start: left,
            end: right,
        } => {
            collect_expr(left, elements);
            collect_expr(right, elements);
        }
        ExprKind::Match { condition, arms } => {
            collect_expr(condition, elements);
            for arm in arms {
                collect_exprs(&arm.patterns, elements);
                collect_stmt(&arm.body, elements);
            }
        }
        ExprKind::Lambda { body, .. } => collect_expr(body, elements),
        ExprKind::String(_)
        | ExprKind::Number(_)
        | ExprKind::Bool(_)
        | ExprKind::Identifier(_)
        | ExprKind::EnumVariant { .. } => {}
    }
}

fn ui_id_arg(args: &[CallArg]) -> Option<String> {
    if let Some(named) = args
        .iter()
        .find(|arg| arg.name.as_deref() == Some("id"))
        .and_then(|arg| literal_string(&arg.value))
    {
        return Some(named);
    }
    args.first().and_then(|arg| literal_string(&arg.value))
}

fn ui_has_literal_layout(args: &[CallArg]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.name.as_deref(), Some("pos") | Some("size")))
        || args
            .iter()
            .any(|arg| matches!(arg.value.kind, ExprKind::Array(_)))
}

fn literal_string(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::String(value) => Some(value.clone()),
        _ => None,
    }
}
