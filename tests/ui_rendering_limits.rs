use std::sync::{Mutex, MutexGuard, OnceLock};
use vice::parse_source;
use vice::semantic::SemanticAnalyzer;

fn ui_test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn run_vice_code_err(code: &str) -> String {
    let program = parse_source(code).expect("Parser failed");
    let mut semantic = SemanticAnalyzer::new();
    if let Err(e) = semantic.analyze(&program) {
        return format!("{:?}", e);
    }
    match vice::execute_program_vm(&program) {
        Ok(_) => "Success".to_string(),
        Err(e) => e.message.clone(),
    }
}

#[test]
fn ui_create_negative_size() {
    let _guard = ui_test_lock();
    let code = r#"
@entry
fn main() {
    let window => ui.window("Test", [-800, -600]);
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("expects positive dimensions") || err.contains("expects"));
}

#[test]
fn ui_invalid_hex_color() {
    let _guard = ui_test_lock();
    let code = r##"
@entry
fn main() {
    let window => ui.window("Test", [800, 600]);
    ui.draw_rect([0, 0], [100, 100], "not_a_color");
}
"##;
    let err = run_vice_code_err(code);
    assert!(err.contains("color must be #rrggbb or #rrggbbaa"));
}

#[test]
fn ui_elements_without_window() {
    let _guard = ui_test_lock();
    // Tests what happens when you draw before opening a window in headless mode.
    let code = r##"
@entry
fn main() {
    ui.draw_rect([0, 0], [100, 100], "#ff0000");
}
"##;
    let err = run_vice_code_err(code);
    assert!(err.contains("Success"));
}

#[test]
fn ui_enormous_amount_of_elements() {
    let _guard = ui_test_lock();
    let code = r##"
@entry
fn main() {
    let window => ui.window("Test", [800, 600]);
    ui.frame(window);
    for i in 0..10000 {
        ui.draw_rect([i, i], [10, 10], "#ff0000");
    }
}
"##;
    // We expect it to not panic
    let out = run_vice_code_err(code);
    assert!(out.contains("Success") || out.contains("window"));
}

#[test]
fn ui_v3_removed_create_reports_migration_hint() {
    let _guard = ui_test_lock();
    let code = r#"
@entry
fn main() {
    ui.create("Old");
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("ui.create was removed in UI v0.3"));
}

#[test]
fn ui_v3_button_requires_stable_id() {
    let _guard = ui_test_lock();
    let code = r#"
@entry
fn main() {
    let window => ui.window("Test", [800, 600]);
    ui.frame(window);
    ui.button("", "Missing id");
}
"#;
    let err = run_vice_code_err(code);
    assert!(err.contains("requires a stable non-empty id"));
}

#[test]
fn ui_v3_headless_runtime_records_commands() {
    let _guard = ui_test_lock();
    use vice::ui::{UiRenderCommand, GLOBAL_UI};

    GLOBAL_UI.clear_recorded_commands();
    let code = r##"
@entry
fn main() {
    let window => ui.window("Recorder", [640, 360]);
    ui.frame(window);
    ui.panel("main", "Dashboard", [10, 20], [300, 200]);
    ui.text("Hello", [20, 40], "#b7f34a");
    let response => ui.button("run", "Run", [20, 80], [100, 36]);
    ui.end_panel();
    output <- response.clicked;
}
"##;

    let (_, output) = vice::run_source(code, vec![]).unwrap();
    assert_eq!(output, vec!["false"]);
    let commands = GLOBAL_UI.recorded_commands();
    assert!(commands.iter().any(|command| matches!(
        command,
        UiRenderCommand::Window { title, .. } if title == "Recorder"
    )));
    assert!(commands.iter().any(|command| matches!(
        command,
        UiRenderCommand::Panel { id, title, .. } if id == "main" && title == "Dashboard"
    )));
    assert!(commands
        .iter()
        .any(|command| matches!(command, UiRenderCommand::Button(button) if button.text == "Run")));
}

#[test]
fn ui_v3_public_api_surface_runs_in_headless_mode() {
    let _guard = ui_test_lock();
    use vice::ui::{UiRenderCommand, GLOBAL_UI};

    GLOBAL_UI.clear_recorded_commands();
    let code = r##"
@entry
fn main() {
    let window => ui.window("Kitchen Sink", [900, 640], "vice");
    ui.frame(window);
    ui.set_theme("vice");
    ui.push_style({"accent": "#b7f34a"});

    ui.menu_bar();
    ui.menu("File");
    let item => ui.menu_item("file.new");
    ui.tooltip("Create a new file");

    ui.panel("main", "Main", [10, 10], [500, 400]);
    ui.row("toolbar");
    ui.button("run", "Run", [20, 40], [90, 32]);
    ui.checkbox("safe");
    ui.slider("volume");
    ui.select("target");
    ui.end();

    ui.column("content");
    ui.heading("Dashboard", [20, 90]);
    ui.text("Body", [20, 120]);
    ui.mono("vice check .", [20, 150]);
    ui.badge("READY", [20, 180]);
    let name_field => ui.input_text("name", "Vice");
    let area => ui.text_area("notes", "Line");
    ui.progress("progress", 0.5);
    ui.list_item("item.one", "One");
    ui.end();

    ui.scroll_area("logs", [20, 220], [220, 120]);
    ui.text("Log line", [24, 240]);
    ui.end_scroll();

    ui.begin_table("packages", 3);
    ui.table_row("package.vice");
    ui.table_cell("Name");
    ui.table_cell("Version");
    ui.table_cell("Status");
    ui.end_table();

    let canvas => ui.canvas("preview", [320, 180]);
    ui.draw_rect(canvas, [550, 40], [100, 80], "#b7f34a");
    ui.draw_line(canvas, [550, 130], [650, 130], "#ffffff");
    ui.draw_circle(canvas, [600, 90], 24, "#55ffff");
    ui.draw_text(canvas, "Canvas", [550, 150], "#ffffff");
    ui.draw_image(canvas, "missing.png", [680, 40], [64, 64]);
    ui.draw_sprite(canvas, "missing.png", [750, 40], [32, 32], [0, 0], [16, 16]);

    ui.status_bar("Ready");
    ui.pop_style();
    ui.end_panel();

    emit <- item.clicked, name_field.value, area.value, canvas.width, window.width;
}
"##;

    let (_, output) = vice::run_source(code, vec![]).unwrap();
    assert_eq!(output, vec!["false Vice Line 320 900"]);
    let commands = GLOBAL_UI.recorded_commands();
    assert!(commands
        .iter()
        .any(|command| matches!(command, UiRenderCommand::Table { id, columns } if id == "packages" && *columns == 3)));
    assert!(commands
        .iter()
        .any(|command| matches!(command, UiRenderCommand::Circle { radius, .. } if *radius == 24)));
    assert!(commands
        .iter()
        .any(|command| matches!(command, UiRenderCommand::StylePush)));
    assert!(commands
        .iter()
        .any(|command| matches!(command, UiRenderCommand::StylePop)));
}

#[test]
fn game2d_aliases_submit_lightweight_draw_commands() {
    let _guard = ui_test_lock();
    use vice::ui::{UiRenderCommand, GLOBAL_UI};

    GLOBAL_UI.set_recording_enabled(true);
    GLOBAL_UI.clear_recorded_commands();
    GLOBAL_UI.reset_stats();

    let code = r##"
@entry
fn main() {
    game2d.begin();
    game2d.set_layer("world");
    game2d.set_camera([0, 0, 320, 180]);
    game2d.rect([4, 8], [16, 12], "#b7f34a");
    game2d.text("Ready", [6, 10], "#ffffff");
    game2d.text_cached("hud.ready", "Ready", [6, 24], "#ffffff");
    game2d.image("missing.png", [20, 20], [8, 8]);
    game2d.sprite("missing.png", [32, 20], [8, 8], [0, 0], [8, 8]);
    game2d.tilemap([], [0, 0]);
    game2d.end();
    emit <- "ok";
}
"##;

    let (_, output) = vice::run_source(code, vec![]).unwrap();
    assert_eq!(output, vec!["ok"]);
    let stats = GLOBAL_UI.stats();
    assert_eq!(stats.rects, 1);
    assert_eq!(stats.labels, 2);
    assert_eq!(stats.images, 2);
    let commands = GLOBAL_UI.recorded_commands();
    assert!(commands
        .iter()
        .any(|command| matches!(command, UiRenderCommand::Rect(rect) if rect.width == 16)));
    assert!(commands
        .iter()
        .any(|command| matches!(command, UiRenderCommand::Text(label) if label.text == "Ready")));
}

#[test]
fn ui_recorder_stays_capped() {
    let _guard = ui_test_lock();
    use vice::ui::{GLOBAL_UI, MAX_RECORDED_COMMANDS};

    GLOBAL_UI.set_recording_enabled(true);
    GLOBAL_UI.clear_recorded_commands();

    let code = r##"
@entry
fn main() {
    for i in 0..9000 {
        ui.draw_rect([i, 0], [1, 1], "#ffffff");
    }
}
"##;

    vice::run_source(code, vec![]).unwrap();
    assert_eq!(GLOBAL_UI.recorded_commands().len(), MAX_RECORDED_COMMANDS);
}
