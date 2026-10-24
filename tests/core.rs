use std::fs;
use std::path::PathBuf;
use std::process::Command;
use vice::ast::{ExprKind, StmtKind};
use vice::error::ViceErrorType;
use vice::formatter::format_source;
use vice::installer::{
    binary_names, build_install_plan, build_install_plan_from_source, install_from_plan,
};
use vice::load_program;
use vice::parse_source;
use vice::run_source;
use vice::semantic::SemanticAnalyzer;
use vice::types::Type;

#[test]
fn parser_accepts_typed_function_return_array_and_import() {
    let program = parse_source(
        r#"
@mount "math.vice";

fn add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    let nums: int[] => [1, 2, 3];
    output <- nums[0];
}
"#,
    )
    .unwrap();

    assert_eq!(program.imports[0].path, "math.vice");
    assert_eq!(
        program.functions[0].params[0].type_annotation,
        Some(Type::Int)
    );
    assert_eq!(program.functions[0].return_type, Some(Type::Int));

    let StmtKind::Variable {
        type_annotation, ..
    } = &program.functions[1].body[0].kind
    else {
        panic!("expected variable statement");
    };
    assert_eq!(type_annotation, &Some(Type::Array(Box::new(Type::Int))));
}

#[test]
fn parser_accepts_v2_use_emit_and_fail() {
    let program = parse_source(
        r#"
use "math.vice";

@entry
fn main() {
    emit <- "hello";
    fail <- "stop";
}
"#,
    )
    .unwrap();

    assert_eq!(program.imports[0].path, "math.vice");
    assert!(matches!(
        program.functions[0].body[0].kind,
        StmtKind::Emit(_)
    ));
    assert!(matches!(
        program.functions[0].body[1].kind,
        StmtKind::Fail { .. }
    ));
}

#[test]
fn semantic_rejects_wrong_argument_count_before_runtime() {
    let program = parse_source(
        r#"
fn add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    output <- add(1);
}
"#,
    )
    .unwrap();

    let error = SemanticAnalyzer::new().analyze(&program).unwrap_err();
    assert_eq!(error.error_type, ViceErrorType::Semantic);
    assert!(error.message.contains("expects 2 arguments, got 1"));
    assert_eq!(error.line, 8);
}

#[test]
fn semantic_checks_array_and_data_field_types() {
    let array_error = parse_source(
        r#"
@entry
fn main() {
    let nums: int[] => [1, "bad"];
}
"#,
    )
    .and_then(|program| SemanticAnalyzer::new().analyze(&program))
    .unwrap_err();
    assert!(array_error.message.contains("Array elements"));

    let data_error = parse_source(
        r#"
data User {
    name: string => "Guest";
}

@entry
fn main() {
    mut user: User => User();
    user.name = 10;
}
"#,
    )
    .and_then(|program| SemanticAnalyzer::new().analyze(&program))
    .unwrap_err();
    assert!(data_error.message.contains("field 'User.name'"));
}

#[test]
fn runtime_buffers_output_and_input() {
    let (_, output) = run_source(
        r#"
@entry
fn main() {
    input -> name;
    output <- "Hello", name;
}
"#,
        vec!["Vice".to_string()],
    )
    .unwrap();

    assert_eq!(output, vec!["Hello Vice"]);
}

#[test]
fn runtime_executes_typed_functions_arrays_and_data() {
    let (_, output) = run_source(
        r#"
data User {
    name: string => "Guest";
}

fn add(a: int, b: int) -> int {
    return a + b;
}

@entry
fn main() {
    mut user: User => User();
    user.name = "Ada";
    mut nums: int[] => [1, 2];
    nums = nums.push(3);
    output <- user.name, add(nums[0], len(nums));
}
"#,
        vec![],
    )
    .unwrap();

    assert_eq!(output, vec!["Ada 4"]);
}

#[test]
fn loader_merges_imported_programs() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("math.vice"),
        r#"
fn add(a: int, b: int) -> int {
    return a + b;
}
"#,
    )
    .unwrap();
    fs::write(
        dir.join("main.vice"),
        r#"
@mount "math.vice";

@entry
fn main() {
    output <- add(2, 5);
}
"#,
    )
    .unwrap();

    let program = load_program(&dir.join("main.vice")).unwrap();
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program).unwrap();
    let output = vice::execute_program_vm(&program).unwrap();
    assert_eq!(output, &["7".to_string()]);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn runtime_errors_keep_source_line() {
    let error = run_source(
        r#"
@entry
fn main() {
    let nums: int[] => [1];
    output <- nums[4];
}
"#,
        vec![],
    )
    .unwrap_err();

    assert_eq!(error.error_type, ViceErrorType::Runtime);
    assert_eq!(error.line, 5);
}

#[test]
fn parser_keeps_expression_line() {
    let program = parse_source(
        r#"
@entry
fn main() {
    output <- 1 + 2;
}
"#,
    )
    .unwrap();

    let StmtKind::Output(expressions) = &program.functions[0].body[0].kind else {
        panic!("expected output");
    };
    assert!(matches!(expressions[0].kind, ExprKind::Binary { .. }));
    assert_eq!(expressions[0].span.line, 4);
}

#[test]
fn parser_preserves_named_arguments() {
    let program = parse_source(
        r#"
@entry
fn main() {
    ui.text(text => "Vice", pos => [10, 20]);
    error(code => 701) <- "boom";
}
"#,
    )
    .unwrap();

    let StmtKind::Expression(expr) = &program.functions[0].body[0].kind else {
        panic!("expected expression statement");
    };
    let ExprKind::MethodCall { args, .. } = &expr.kind else {
        panic!("expected method call");
    };
    assert_eq!(args[0].name.as_deref(), Some("text"));
    assert_eq!(args[1].name.as_deref(), Some("pos"));

    let StmtKind::Error { args, .. } = &program.functions[0].body[1].kind else {
        panic!("expected error statement");
    };
    assert_eq!(args[0].name.as_deref(), Some("code"));
}

#[test]
fn formatter_preserves_named_arguments() {
    let source = r#"@entry
fn main() {
    ui.text(text => "Vice", pos => [10, 20]);
}
"#;

    let formatted = format_source(source, &PathBuf::from("named.vice")).unwrap();
    assert!(formatted.contains("text => \"Vice\""));
    assert!(formatted.contains("pos => [10, 20]"));
    assert_eq!(formatted, source);
}

#[test]
fn formatter_uses_v2_import_emit_and_fail_forms() {
    let source = r#"use "math.vice";

@entry
fn main() {
    emit <- "ok";
    fail <- "boom";
}
"#;
    let formatted = format_source(source, &PathBuf::from("v2.vice")).unwrap();
    assert_eq!(formatted, source);
}

#[test]
fn formatter_preserves_stacked_attributes() {
    let source = r#"@lowlevel
@entry
fn main() {
    output <- 5 / 2;
}
"#;

    let formatted = format_source(source, &PathBuf::from("attrs.vice")).unwrap();
    assert_eq!(formatted, source);
}

#[test]
fn stdlib_core_modules_work() {
    let (_, output) = run_source(
        r#"
@entry
@autocheck
fn main() {
    let clean => string.trim("  Vice Core  ");
    let parts => string.split(clean, " ");
    output <- string.upper(array.join(parts, "-"));
    output <- string.contains(clean, "Core");
    output <- math.max(10, math.abs(12));
    output <- path.basename(path.join("examples", "stdlib_core.vice"));
    output <- time.now_ms() > 0;
}
"#,
        vec![],
    )
    .unwrap();

    assert_eq!(output[0], "VICE-CORE");
    assert_eq!(output[1], "true");
    assert_eq!(output[2], "12");
    assert_eq!(output[3], "stdlib_core.vice");
    assert_eq!(output[4], "true");
}

#[test]
fn stdlib_argument_types_are_checked_semantically() {
    let error = parse_source(
        r#"
@entry
fn main() {
    output <- string.contains("Vice", 10);
}
"#,
    )
    .and_then(|program| SemanticAnalyzer::new().analyze(&program))
    .unwrap_err();

    assert_eq!(error.error_type, ViceErrorType::Semantic);
    assert!(error.message.contains("expected string"));
}

#[test]
fn unary_operators_parse_typecheck_format_and_run() {
    let source = r#"@entry
fn main() {
    let negative: int => -12;
    let grouped: int => -(1 + 2);
    let inverted: bool => !true;
    let compared: bool => !(1 == 2);
    output <- negative, grouped, inverted, compared;
}
"#;

    let program = parse_source(source).unwrap();
    SemanticAnalyzer::new().analyze(&program).unwrap();

    let formatted = format_source(source, &PathBuf::from("unary.vice")).unwrap();
    assert_eq!(formatted, source);

    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["-12 -3 false true"]);
}

#[test]
fn unary_operators_reject_wrong_types() {
    let number_error = parse_source(
        r#"
@entry
fn main() {
    output <- -"bad";
}
"#,
    )
    .and_then(|program| SemanticAnalyzer::new().analyze(&program))
    .unwrap_err();
    assert_eq!(number_error.error_type, ViceErrorType::Semantic);
    assert!(number_error.message.contains("Unary '-' expects numeric"));

    let bool_error = parse_source(
        r#"
@entry
fn main() {
    output <- !10;
}
"#,
    )
    .and_then(|program| SemanticAnalyzer::new().analyze(&program))
    .unwrap_err();
    assert_eq!(bool_error.error_type, ViceErrorType::Semantic);
    assert!(bool_error.message.contains("Unary '!' expects bool"));
}

#[test]
fn ui_rect_example_passes_semantic_check_without_opening_window() {
    let source =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/ui_rect.vice"))
            .unwrap();
    let program = parse_source(&source).unwrap();
    SemanticAnalyzer::new().analyze(&program).unwrap();
}

#[test]
fn loader_reports_import_cycles() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.vice"), "@mount \"b.vice\";\n").unwrap();
    fs::write(dir.join("b.vice"), "@mount \"a.vice\";\n").unwrap();

    let error = load_program(&dir.join("a.vice")).unwrap_err();
    assert!(error.message.contains("Import cycle detected"));
    assert_eq!(error.line, 1);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_check_run_and_legacy_run_work() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.vice");
    fs::write(
        &file,
        r#"
@entry
fn main() {
    output <- "ok";
}
"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let check = Command::new(bin).arg("check").arg(&file).output().unwrap();
    assert!(check.status.success());
    assert!(String::from_utf8_lossy(&check.stdout).contains("OK:"));

    let run = Command::new(bin).arg("run").arg(&file).output().unwrap();
    assert!(run.status.success());
    assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "ok");

    let legacy = Command::new(bin).arg(&file).output().unwrap();
    assert!(legacy.status.success());
    assert_eq!(String::from_utf8_lossy(&legacy.stdout).trim(), "ok");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn vicerun_check_run_and_legacy_run_work() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.vice");
    fs::write(
        &file,
        r#"
@entry
fn main() {
    output <- "ok";
}
"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let check = Command::new(bin).arg("check").arg(&file).output().unwrap();
    assert!(check.status.success());
    assert!(String::from_utf8_lossy(&check.stdout).contains("OK:"));

    let run = Command::new(bin).arg("run").arg(&file).output().unwrap();
    assert!(run.status.success());
    assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "ok");

    let legacy = Command::new(bin).arg(&file).output().unwrap();
    assert!(legacy.status.success());
    assert_eq!(String::from_utf8_lossy(&legacy.stdout).trim(), "ok");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_run_profile_reports_runtime_stats_and_honors_frame_limit() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("profile_game.vice");
    fs::write(
        &file,
        r##"
@entry
fn main() {
    let window => ui.window("Profile", [320, 180]);
    mut frames => 0;
    while (ui.frame(window)) {
        ui.draw_rect([frames, 0], [8, 8], "#ffffff");
        frames += 1;
    }
    output <- frames;
}
"##,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin)
        .arg("run")
        .arg("--profile")
        .arg(&file)
        .arg("--frames")
        .arg("3")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"));
    assert!(stdout.contains("Vice profile report"));
    assert!(stdout.contains("frames:"));
    assert!(stdout.contains("vm_instructions:"));
    assert!(stdout.contains("ui_rects: 3"));
    assert!(stdout.contains("ui_recorded_commands:"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_bench_json_reports_hot_opcodes_and_ui_stats() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("bench_game.vice");
    fs::write(
        &file,
        r##"
@entry
fn main() {
    let window => ui.window("Bench", [320, 180]);
    mut frames => 0;
    while (ui.frame(window)) {
        game2d.rect([frames, 0], [8, 8], "#ffffff");
        frames += 1;
    }
}
"##,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("bench")
        .arg(&file)
        .arg("--frames")
        .arg("5")
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert!(report["frames"].as_u64().unwrap() >= 5);
    assert_eq!(report["ui"]["rects"], 5);
    assert!(report["vm_instructions"].as_u64().unwrap() > 0);
    assert!(!report["hot_opcodes"].as_array().unwrap().is_empty());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_test_metrics_reports_case_counts() {
    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("test-metrics")
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert!(report["rust_tests"].as_u64().unwrap() > 200);
    assert!(report["total_cases"].as_u64().unwrap() >= report["rust_tests"].as_u64().unwrap());
    assert_eq!(report["target_0_4_0"], 2000);
}

#[test]
fn cli_std_scene_asset_tilemap_editor_modules_run() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("editor_sdk.vice");
    fs::write(
        &file,
        r#"
use "std/scene.vice";
use "std/assets.vice";
use "std/tilemap.vice";
use "std/editor.vice";

@entry
fn main() {
    let scene => scene_empty("Demo");
    let texture => assets_texture("hero", "assets/hero.png");
    let map => tilemap_empty(20, 10, 16, 16);
    let layout => editor_ui_layout_empty("HUD");
    output <- scene["kind"], scene["name"];
    output <- texture["kind"], texture["id"];
    output <- map["kind"], map["width"];
    output <- layout["kind"], layout["name"];
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scene Demo"));
    assert!(stdout.contains("texture hero"));
    assert!(stdout.contains("tilemap 20"));
    assert!(stdout.contains("ui_layout HUD"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_new_creates_hello_world_project() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let project = dir.join("sample_app");

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin).arg("new").arg(&project).output().unwrap();
    assert!(output.status.success());

    let main_file = project.join("src").join("main.vice");
    let readme_file = project.join("README.md");
    let manifest_file = project.join("vice.toml");
    let test_file = project.join("tests").join("main_test.vice");
    assert!(main_file.exists());
    assert!(readme_file.exists());
    assert!(manifest_file.exists());
    assert!(test_file.exists());
    assert!(project.join(".gitignore").exists());

    let main_source = fs::read_to_string(&main_file).unwrap();
    assert!(main_source.contains("@entry"));
    assert!(main_source.contains("Hello, Vice!"));

    let readme = fs::read_to_string(&readme_file).unwrap();
    assert!(readme.contains("vice run ."));
    assert!(readme.contains("vice check ."));
    assert!(readme.contains("vice fmt ."));
    assert!(readme.contains("vice test ."));
    let manifest = fs::read_to_string(&manifest_file).unwrap();
    assert!(manifest.contains("entry = \"src/main.vice\""));

    let check = Command::new(bin)
        .arg("check")
        .arg(&project)
        .output()
        .unwrap();
    assert!(check.status.success());

    let run = Command::new(bin).arg("run").arg(&project).output().unwrap();
    assert!(run.status.success());
    assert!(String::from_utf8_lossy(&run.stdout).contains("Hello, Vice!"));

    let test = Command::new(bin)
        .arg("test")
        .arg(&project)
        .output()
        .unwrap();
    assert!(test.status.success());
    assert!(String::from_utf8_lossy(&test.stdout).contains("1 passed; 0 failed"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_new_platformer_template_checks_tests_and_vam_inspects_project() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let project = dir.join("jump_lab");

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin)
        .arg("new")
        .arg(&project)
        .arg("--template")
        .arg("platformer")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let main_source = fs::read_to_string(project.join("src").join("main.vice")).unwrap();
    assert!(main_source.contains("use \"std/platformer.vice\";"));
    assert!(main_source.contains("platformer_update_player"));

    let manifest = fs::read_to_string(project.join("vice.toml")).unwrap();
    assert!(manifest.contains("kind = \"game\""));

    let check = Command::new(bin)
        .arg("check")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );

    let test = Command::new(bin)
        .arg("test")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        test.status.success(),
        "{}",
        String::from_utf8_lossy(&test.stderr)
    );
    assert!(String::from_utf8_lossy(&test.stdout).contains("1 passed; 0 failed"));

    let vam = Command::new(env!("CARGO_BIN_EXE_vicetool"))
        .arg("run")
        .arg("VAM")
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(
        vam.status.success(),
        "{}",
        String::from_utf8_lossy(&vam.stderr)
    );
    let stdout = String::from_utf8_lossy(&vam.stdout);
    assert!(stdout.contains("Vice App Manager"));
    assert!(stdout.contains("Entry:"));
    assert!(stdout.contains("Panels:"));
    assert!(stdout.contains("Scene/UI data model"));
    assert!(stdout.contains("UI elements:"));
    assert!(project.join("scenes").join("main.vscene").is_file());
    assert!(project.join("ui").join("main.vui").is_file());
    assert!(project.join("assets").join("assets.json").is_file());
    assert!(project.join("vam.project.json").is_file());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_init_doc_lsp_and_migrate_support_v2_workflow() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let bin = env!("CARGO_BIN_EXE_vice");

    let init = Command::new(bin).arg("init").arg(&dir).output().unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(dir.join("vice.toml").is_file());
    assert!(fs::read_to_string(dir.join("src").join("main.vice"))
        .unwrap()
        .contains("emit <-"));

    let doc = Command::new(bin)
        .arg("doc")
        .arg("--json")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(doc.status.success());
    let report: serde_json::Value = serde_json::from_slice(&doc.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["functions"][0]["name"], "main");

    let lsp = Command::new(bin).arg("lsp").output().unwrap();
    assert!(lsp.status.success());
    assert!(String::from_utf8_lossy(&lsp.stdout).contains("completionProvider"));

    let legacy = dir.join("legacy.vice");
    fs::write(
        &legacy,
        "@mount \"math.vice\";\n@entry\nfn main() {\n    output <- \"ok\";\n    error() <- \"boom\";\n}\n",
    )
    .unwrap();
    let migrate_check = Command::new(bin)
        .arg("migrate")
        .arg("--check")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(!migrate_check.status.success());

    let migrate = Command::new(bin).arg("migrate").arg(&dir).output().unwrap();
    assert!(migrate.status.success());
    let migrated = fs::read_to_string(&legacy).unwrap();
    assert!(migrated.contains("use \"math.vice\";"));
    assert!(migrated.contains("emit <- \"ok\";"));
    assert!(migrated.contains("fail <- \"boom\";"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_migrate_preserves_legacy_words_inside_strings_and_comments() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("legacy_text.vice");
    fs::write(
        &file,
        r#"@entry
fn main() {
    # @mount "comment.vice";
    // output <- "comment";
    let text => "output <- stays text; error() <- stays text; @mount stays text";
    output <- "real output";
    error() <- "real failure";
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("migrate")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let migrated = fs::read_to_string(&file).unwrap();
    assert!(migrated.contains("# @mount \"comment.vice\";"));
    assert!(migrated.contains("// output <- \"comment\";"));
    assert!(migrated.contains(
        "let text => \"output <- stays text; error() <- stays text; @mount stays text\";"
    ));
    assert!(migrated.contains("emit <- \"real output\";"));
    assert!(migrated.contains("fail <- \"real failure\";"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_check_json_reports_legacy_warnings_without_failing() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("legacy_warning.vice");
    fs::write(
        &file,
        r#"@entry
fn main() {
    # @mount "comment.vice";
    let text => "output <- is only text";
    output <- "ok";
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("check")
        .arg("--json")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["warnings"].as_array().unwrap().len(), 1);
    assert_eq!(report["warnings"][0]["code"], "legacy-output");
    assert_eq!(report["warnings"][0]["line"], 5);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_run_passes_arguments_only_after_separator() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("args.vice");
    fs::write(
        &file,
        r#"@entry
fn main() {
    let args: string[] => process.args();
    output <- len(args), args[0], args[1];
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg(&file)
        .arg("--")
        .arg("first")
        .arg("second")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "2 first second"
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_test_runs_each_vice_test_function_in_isolation() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("math_test.vice");
    fs::write(
        &file,
        r#"@test
fn addition_works() {
    if (2 + 2 != 4) {
        error() <- "addition failed";
    }
}

@test
fn failure_is_reported() {
    error(code => 17) <- "intentional failure";
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("test")
        .arg(&file)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("PASS"));
    assert!(stdout.contains("addition_works"));
    assert!(stderr.contains("FAIL"));
    assert!(stderr.contains("failure_is_reported"));
    assert!(stderr.contains("intentional failure"));
    assert!(stderr.contains("1 passed; 1 failed"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_test_json_is_machine_readable() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("passing_test.vice");
    fs::write(
        &file,
        "@test\nfn passes() {\n    output <- \"observed\";\n}\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("test")
        .arg("--json")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["summary"]["total"], 1);
    assert_eq!(report["summary"]["passed"], 1);
    assert_eq!(report["tests"][0]["name"], "passes");
    assert_eq!(report["tests"][0]["output"][0], "observed");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_check_directory_reports_an_aggregate_json_summary() {
    let dir = unique_temp_dir();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src").join("valid.vice"), "fn helper() {}\n").unwrap();
    fs::write(
        dir.join("src").join("invalid.vice"),
        "fn broken() {\n    let value: int => \"wrong\";\n}\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("check")
        .arg("--json")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], false);
    assert_eq!(report["summary"]["checked"], 2);
    assert_eq!(report["summary"]["passed"], 1);
    assert_eq!(report["summary"]["failed"], 1);
    assert_eq!(report["files"].as_array().unwrap().len(), 2);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_fmt_directory_is_recursive_and_skips_build_directories() {
    let dir = unique_temp_dir();
    fs::create_dir_all(dir.join("src").join("nested")).unwrap();
    fs::create_dir_all(dir.join("target")).unwrap();
    let first = dir.join("src").join("first.vice");
    let second = dir.join("src").join("nested").join("second.vice");
    let ignored = dir.join("target").join("ignored.vice");
    let unformatted = "fn helper(){\noutput <- \"ok\";\n}\n";
    fs::write(&first, unformatted).unwrap();
    fs::write(&second, unformatted).unwrap();
    fs::write(&ignored, unformatted).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("fmt")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(fs::read_to_string(&first)
        .unwrap()
        .contains("fn helper() {"));
    assert!(fs::read_to_string(&second)
        .unwrap()
        .contains("fn helper() {"));
    assert_eq!(fs::read_to_string(&ignored).unwrap(), unformatted);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_run_rejects_manifest_entries_that_escape_the_project() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("vice.json"),
        r#"{"name":"unsafe","entry":"../outside.vice"}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsafe project entry"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_run_rejects_v2_manifest_entries_that_escape_the_project() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("vice.toml"),
        "[package]\nname = \"unsafe\"\nversion = \"0.2.0\"\nentry = \"../outside.vice\"\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsafe project entry"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_new_refuses_existing_folder_without_overwriting() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let project = dir.join("sample_app");
    fs::create_dir_all(&project).unwrap();
    let marker = project.join("main.vice");
    fs::write(&marker, "do not overwrite").unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin).arg("new").arg(&project).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
    assert_eq!(fs::read_to_string(marker).unwrap(), "do not overwrite");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_error_uses_source_snippet() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("bad.vice");
    fs::write(
        &file,
        r#"
@entry
fn main() {
    let total: int => "bad";
}
"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin).arg("check").arg(&file).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("bad.vice:4:1"));
    assert!(stderr.contains("let total"));
    assert!(stderr.contains("^"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_fmt_check_accepts_formatted_file() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("formatted.vice");
    fs::write(
        &file,
        r#"@entry
fn main() {
    output <- "ok";
}
"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin)
        .arg("fmt")
        .arg("--check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("OK:"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_fmt_check_rejects_unformatted_file() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("unformatted.vice");
    fs::write(
        &file,
        r#"@entry
fn main(){
output <- "ok";
}
"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin)
        .arg("fmt")
        .arg("--check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not formatted"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_fmt_preserves_standalone_and_trailing_comments() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("commented.vice");
    fs::write(
        &file,
        r#"# application entry
@entry
fn main(){ // execution starts here
# a URL is not a comment
output <- "https://vice.dev/docs"; # visible result
# keep this inside the function
}
# end of file
"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin).arg("fmt").arg(&file).output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        r#"# application entry
@entry
fn main() { // execution starts here
    # a URL is not a comment
    output <- "https://vice.dev/docs"; # visible result
    # keep this inside the function
}
# end of file
"#
    );

    let check = Command::new(bin)
        .arg("fmt")
        .arg("--check")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_fmt_writes_formatted_source_in_place() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("write.vice");
    fs::write(
        &file,
        r#"@entry
fn main(){
output <- "ok";
}
"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin).arg("fmt").arg(&file).output().unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Formatted:"));
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        r#"@entry
fn main() {
    output <- "ok";
}
"#
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_fmt_stdout_does_not_modify_the_source() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("stdout.vice");
    let original = r#"@entry
fn main(){
output <- "ok";
}
"#;
    fs::write(&file, original).unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin)
        .arg("fmt")
        .arg(&file)
        .arg("--stdout")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("fn main() {"));
    assert_eq!(fs::read_to_string(&file).unwrap(), original);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_fmt_stdin_formats_unsaved_source() {
    use std::io::Write;

    let mut child = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("fmt")
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"fn helper(){\n# keep me\noutput <- \"ok\";\n}\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "fn helper() {\n    # keep me\n    output <- \"ok\";\n}\n"
    );
}

#[test]
fn cli_check_json_reports_success_and_structured_errors() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let valid = dir.join("valid.vice");
    let invalid = dir.join("invalid.vice");
    fs::write(&valid, "@entry\nfn main() {}\n").unwrap();
    fs::write(
        &invalid,
        "@entry\nfn main() {\n    let value: int => \"wrong\";\n}\n",
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let success = Command::new(bin)
        .arg("check")
        .arg("--json")
        .arg(&valid)
        .output()
        .unwrap();
    assert!(success.status.success());
    let success_json: serde_json::Value = serde_json::from_slice(&success.stdout).unwrap();
    assert_eq!(success_json["ok"], true);
    assert_eq!(
        success_json["file"],
        fs::canonicalize(&valid).unwrap().display().to_string()
    );

    let failure = Command::new(bin)
        .arg("check")
        .arg(&invalid)
        .arg("--json")
        .output()
        .unwrap();
    assert!(!failure.status.success());
    assert!(failure.stderr.is_empty());
    let failure_json: serde_json::Value = serde_json::from_slice(&failure.stdout).unwrap();
    assert_eq!(failure_json["ok"], false);
    assert_eq!(failure_json["diagnostic"]["type"], "semantic");
    assert_eq!(failure_json["diagnostic"]["line"], 3);
    assert_eq!(
        failure_json["diagnostic"]["file"],
        fs::canonicalize(&invalid).unwrap().display().to_string()
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_doctor_json_is_machine_readable() {
    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin)
        .arg("doctor")
        .arg("--json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(report["scope"], "All");
    assert_eq!(report["runtime"]["vm"], "ok");
}

#[test]
fn cli_invalid_arguments_use_exit_code_two() {
    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin).arg("check").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires a path"));
}

#[test]
fn vicerun_fmt_check_matches_vice() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("formatted.vice");
    fs::write(
        &file,
        r#"@entry
fn main() {
    output <- "ok";
}
"#,
    )
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_vice");
    let output = Command::new(bin)
        .arg("fmt")
        .arg("--check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("OK:"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_smoke_checks_and_runs_stdlib_core_example() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = manifest.join("examples/stdlib_core.vice");
    let bin = env!("CARGO_BIN_EXE_vice");

    let check = Command::new(bin)
        .current_dir(&manifest)
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();
    assert!(check.status.success());

    let run = Command::new(bin)
        .current_dir(&manifest)
        .arg("run")
        .arg(&file)
        .output()
        .unwrap();
    assert!(run.status.success());
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("VICE-CORE"));
    assert!(stdout.contains("stdlib_core.vice"));
}

#[test]
fn cli_checks_and_runs_vice1_feature_examples() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = env!("CARGO_BIN_EXE_vice");
    let examples = [
        "vice1_numeric_modes.vice",
        "vice1_autocheck_stdlib.vice",
        "vice1_data_lowlevel.vice",
    ];

    for example in examples {
        let file = manifest.join("examples").join(example);
        let check = Command::new(bin)
            .current_dir(&manifest)
            .arg("check")
            .arg(&file)
            .output()
            .unwrap();
        assert!(check.status.success(), "check failed for {}", example);

        let run = Command::new(bin)
            .current_dir(&manifest)
            .arg("run")
            .arg(&file)
            .output()
            .unwrap();
        assert!(run.status.success(), "run failed for {}", example);
        assert!(!String::from_utf8_lossy(&run.stdout).trim().is_empty());
    }
}

#[test]
fn installer_plan_and_copy_use_local_release_binaries() {
    let dir = unique_temp_dir();
    let release_dir = dir.join("target").join("release");
    let install_dir = dir.join("install");
    fs::create_dir_all(&release_dir).unwrap();
    fs::create_dir_all(release_dir.join("std")).unwrap();
    fs::write(
        release_dir.join("std").join("game.vice"),
        "fn marker() {}\n",
    )
    .unwrap();

    for name in binary_names() {
        fs::write(release_dir.join(name), format!("fake {}", name)).unwrap();
    }

    let plan = build_install_plan(&dir, Some(install_dir.clone())).unwrap();
    assert_eq!(plan.source_dir, release_dir);
    assert_eq!(plan.target_dir, install_dir);
    assert_eq!(plan.binaries.len(), binary_names().len());

    let installed = install_from_plan(&plan).unwrap();
    assert_eq!(installed.len(), binary_names().len() + 1);
    for path in installed {
        assert!(path.exists());
    }

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn installer_plan_can_use_release_bundle_directory() {
    let dir = unique_temp_dir();
    let bundle_dir = dir.join("bundle");
    let install_dir = dir.join("install");
    fs::create_dir_all(&bundle_dir).unwrap();
    fs::create_dir_all(bundle_dir.join("std")).unwrap();
    fs::write(bundle_dir.join("std").join("game.vice"), "fn marker() {}\n").unwrap();

    for name in binary_names() {
        fs::write(bundle_dir.join(name), format!("fake {}", name)).unwrap();
    }

    let plan = build_install_plan_from_source(&bundle_dir, Some(install_dir.clone())).unwrap();
    assert_eq!(plan.source_dir, bundle_dir);
    assert_eq!(plan.target_dir, install_dir);

    let installed = install_from_plan(&plan).unwrap();
    assert_eq!(installed.len(), binary_names().len() + 1);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn installer_plan_reports_missing_release_binaries() {
    let dir = unique_temp_dir();
    fs::create_dir_all(dir.join("target").join("release")).unwrap();

    let error = build_install_plan(&dir, Some(dir.join("install"))).unwrap_err();
    assert!(error.to_string().contains("cargo build --release"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn runtime_recursion_factorial() {
    let source = r#"
fn fact(n: int) -> int {
    if (n <= 1) { return 1; }
    return n * fact(n - 1);
}
@entry
fn main() {
    output <- fact(5);
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["120"]);
}

#[test]
fn runtime_logical_short_circuit() {
    let source = r#"
@autocheck
data State { mut effect: bool => false; }
fn set_effect(s: State) -> bool {
    mut temp => s;
    temp.effect = true;
    return true;
}
@entry
fn main() {
    mut s => State();
    let a => false && set_effect(s);
    let b => true || set_effect(s);
    output <- s.effect;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["false"]);
}

#[test]
fn runtime_match_as_expression() {
    let source = r#"
@autocheck
@entry
fn main() {
    let x => 2;
    let result => match x {
        1 => "one",
        2 => "two",
        _ => "many"
    };
    output <- result;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["two"]);
}

#[test]
fn runtime_match_multiple_patterns() {
    let source = r#"
@autocheck
@entry
fn main() {
    let x => 5;
    let category => match x {
        1 | 3 | 5 | 7 | 9 => "odd",
        0 | 2 | 4 | 6 | 8 => "even"
    };
    output <- category;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["odd"]);
}

#[test]
fn hybrid_mode_highlevel_division() {
    let source = r#"
@highlevel
@entry
fn main() {
    output <- 5 / 2;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["2.5"]);
}

#[test]
fn hybrid_mode_lowlevel_implicit_truncate() {
    let source = r#"
@autocheck
@lowlevel
@entry
fn main() {
    let x: int => 5.75;
    output <- x;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["5"]);
}

#[test]
fn semantic_rejects_shadowing_const_with_mut() {
    // Vice v1 design decision: allow or disallow shadowing?
    // Testing isolation of scopes here.
    let source = r#"
@entry
fn main() {
    let x: int => 10;
    {
        let x: int => 20;
        output <- x;
    }
    output <- x;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["20", "10"]);
}

#[test]
fn stdlib_math_min_max_mixed_types() {
    let source = r#"
@entry
fn main() {
    output <- math.min(10, 5.5);
    output <- math.max(2.2, 1);
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["5.5", "2.2"]);
}

#[test]
fn stdlib_string_edge_cases() {
    let source = r#"
@autocheck
@entry
fn main() {
    output <- string.trim("   ");
    output <- string.split("a,b,c", ",");
    output <- string.contains("Vice", "v"); # case sensitive check
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["", "[\"a\", \"b\", \"c\"]", "false"]);
}

#[test]
fn runtime_foreach_empty_array() {
    let source = r#"
@autocheck
@entry
fn main() {
    let items: int[] => [];
    mut count => 0;
    for i in items {
        count = count + 1;
    }
    output <- count;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn semantic_checks_data_default_values() {
    let source = r#"
data BadData {
    val: int => "not an int";
}
@entry
fn main() {
    let d => BadData();
}
"#;
    let error = parse_source(source)
        .and_then(|p| SemanticAnalyzer::new().analyze(&p))
        .unwrap_err();
    assert!(error.message.contains("Type mismatch in data"));
}

#[test]
fn ui_hex_color_parser_cases() {
    use vice::ui::parse_hex_color;
    assert_eq!(parse_hex_color("#ff0000"), Some([255, 0, 0, 255]));
    assert_eq!(parse_hex_color("00ff00"), Some([0, 255, 0, 255]));
    assert_eq!(parse_hex_color("#123"), None);
    assert_eq!(parse_hex_color("invalid"), None);
}

#[test]
fn runtime_member_assign_nested() {
    let source = r#"
@autocheck
data Pos { x: int => 0; y: int => 0; }
data Sprite { pos: Pos => Pos(); }
@entry
fn main() {
    mut s => Sprite();
    s.pos.x = 100;
    output <- s.pos.x;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["100"]);
}

#[test]
fn semantic_rejects_non_boolean_if_condition() {
    let source = r#"
@entry
fn main() {
    if (10) { output <- "no"; }
}
"#;
    let error = parse_source(source)
        .and_then(|p| SemanticAnalyzer::new().analyze(&p))
        .unwrap_err();
    assert!(error.message.contains("condition must be bool"));
}

#[test]
fn runtime_array_index_out_of_bounds() {
    let source = r#"
@autocheck
@entry
fn main() {
    let a => [1, 2];
    output <- a[5];
}
"#;
    let error = run_source(source, vec![]).unwrap_err();
    assert_eq!(error.error_type, ViceErrorType::Runtime);
    assert!(error.message.contains("out of bounds"));
}

#[test]
fn semantic_enforces_autocheck_requirements() {
    let source = r#"
fn main() {
    let x => 5; # No @autocheck and no type annotation
}
"#;
    let ok = parse_source(source).and_then(|p| SemanticAnalyzer::new().analyze(&p));
    assert!(ok.is_ok());
}

#[test]
fn runtime_boss_1_cellular_automata_heavy() {
    // Symulacja automatu komórkowego na siatce 6x6 przez 5 generacji
    // Testuje: pętle w pętlach, indeksowanie tablic 2D, mutację dużych struktur
    let source = r#"
@autocheck
@entry
fn main() {
    mut grid => [
        [0,1,0,0,0,0], [0,0,1,0,0,0], [1,1,1,0,0,0],
        [0,0,0,0,0,0], [0,0,0,0,0,0], [0,0,0,0,0,0]
    ];

    mut gen => 0;
    while (gen < 3) {
        mut next => [
            [0,0,0,0,0,0], [0,0,0,0,0,0], [0,0,0,0,0,0],
            [0,0,0,0,0,0], [0,0,0,0,0,0], [0,0,0,0,0,0]
        ];
        mut y => 1;
        while (y < 5) {
            mut x => 1;
            while (x < 5) {
                let neighbors => grid[y-1][x-1] + grid[y-1][x] + grid[y-1][x+1] +
                                 grid[y][x-1]                + grid[y][x+1] +
                                 grid[y+1][x-1] + grid[y+1][x] + grid[y+1][x+1];
                
                if (grid[y][x] == 1) {
                    if (neighbors == 2 || neighbors == 3) { next[y][x] = 1; }
                } else {
                    if (neighbors == 3) { next[y][x] = 1; }
                }
                x = x + 1;
            }
            y = y + 1;
        }
        grid = next;
        gen = gen + 1;
    }
    output <- grid[2][1], grid[2][2], grid[2][3];
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    // Po 3 generacjach pattern "Glider" powinien się przesunąć
    assert_eq!(output, vec!["0 0 1"]);
}

#[test]
fn runtime_boss_2_nqueens_backtracking() {
    // Mini solver dla problemu 4 hetmanów (aby nie przekroczyć czasu testu, ale zmęczyć rekurencję)
    // Testuje: głęboką rekurencję, przekazywanie tablic, backtracking
    let source = r#"
@autocheck
fn is_safe(row: int, col: int, queens: int[]) -> bool {
    mut i => 0;
    while (i < row) {
        let q_col => queens[i];
        if (q_col == col || math.abs(to_float(q_col - col)) == math.abs(to_float(i - row))) {
            return false;
        }
        i = i + 1;
    }
    return true;
}

fn solve(row: int, n: int, queens: int[]) -> int[] {
    if (row == n) { return queens; }
    mut col => 0;
    while (col < n) {
        if (is_safe(row, col, queens)) {
            queens[row] = col;
            let res => solve(row + 1, n, queens);
            if (len(res) > 0) { return res; }
        }
        col = col + 1;
    }
    return [];
}

@entry
fn main() {
    let board => solve(0, 4, [0, 0, 0, 0]);
    output <- len(board) > 0, board[0], board[1], board[2], board[3];
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["true 1 3 0 2"]);
}

#[test]
fn runtime_boss_3_deep_ecs_stress() {
    // Symulacja systemu ECS z ekstremalnie zagnieżdżonymi danymi
    // Testuje: assign_member_recursive, Instance management, dot notation
    let source = r#"
@autocheck
data Vec2 { mut x: float => 0.0; mut y: float => 0.0; }
data Transform { mut pos: Vec2 => Vec2(); mut rot: float => 0.0; }
data Physics { mut transform: Transform => Transform(); mut vel: Vec2 => Vec2(); }
data Entity { mut phys: Physics => Physics(); mut id: int => 0; }

@entry
fn main() {
    mut player => Entity(id => 1337);
    
    # Tortura: głębokie przypisania w pętli
    mut i => 0;
    while (i < 10) {
        player.phys.transform.pos.x = player.phys.transform.pos.x + 1.5;
        player.phys.transform.pos.y = player.phys.transform.pos.y + 2.0;
        player.phys.vel.x = to_float(i);
        i = i + 1;
    }

    # Sprawdzanie czy stan instancji jest spójny po tylu rekurencyjnych kopiowaniach
    let final_x => player.phys.transform.pos.x;
    let final_id => player.id;
    
    # Kolejna warstwa: przypisanie całej pod-struktury
    mut new_pos => Vec2(x => 500.0, y => 500.0);
    player.phys.transform.pos = new_pos;
    
    output <- final_x, player.phys.transform.pos.x, final_id;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["15 500 1337"]);
}

fn unique_temp_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "vice_test_{}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::thread::current()
            .name()
            .unwrap_or("worker")
            .replace([':', '\\', '/', ' '], "_")
    ))
}
