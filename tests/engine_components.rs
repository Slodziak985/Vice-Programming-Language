use std::fs;
use vice::run_source;

#[test]
fn runtime_audio_play_sound_handles_missing_file_safely() {
    let source = r#"
@entry
fn main() {
    let success => audio.play_sound("missing_sound.ogg");
    output <- success;
}
"#;
    let result = run_source(source, vec![]);
    assert!(result.is_err());
}

#[test]
fn runtime_ui_image_adds_image_command() {
    let source = r#"
@entry
fn main() {
    let success => ui.draw_image("missing_texture.png", [10, 20], [100, 100]);
    output <- success;
}
"#;
    let (_, output) = run_source(source, vec![]).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn runtime_engine_load_scene_vscn() {
    let dir = std::env::temp_dir().join("vice_test_vscn");
    fs::create_dir_all(&dir).unwrap();
    let scene_path = dir.join("test_level.vscn");

    let vscn_content = r#"{
        "__type__": "Level",
        "name": "Level 1",
        "entities": [
            {
                "__type__": "Player",
                "x": 100,
                "y": 200
            }
        ]
    }"#;
    fs::write(&scene_path, vscn_content).unwrap();

    // Replace backslashes for Windows if needed, though we're mostly on Linux here
    let scene_path_str = scene_path.to_str().unwrap().replace("\\", "/");

    let source = format!(
        r#"
data Player {{
    x: int => 0;
    y: int => 0;
}}

data Level {{
    name: string => "";
    entities: Player[] => [];
}}

@entry
fn main() {{
    mut level: Level => engine.load_scene("{}");
    output <- level.name;
    output <- level.entities[0].x;
}}
"#,
        scene_path_str
    );

    let (_, output) = run_source(&source, vec![]).unwrap();
    assert_eq!(output, vec!["Level 1", "100"]);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn runtime_engine_load_scene_invalid_json() {
    let dir = std::env::temp_dir().join("vice_test_vscn_invalid");
    fs::create_dir_all(&dir).unwrap();
    let scene_path = dir.join("test_invalid.vscn");
    fs::write(&scene_path, "{ bad_json }").unwrap();

    let scene_path_str = scene_path.to_str().unwrap().replace("\\", "/");

    let source = format!(
        r#"
@entry
fn main() {{
    let level => engine.load_scene("{}");
}}
"#,
        scene_path_str
    );

    let result = run_source(&source, vec![]);
    assert!(result.is_err());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn runtime_engine_save_and_load_layout_round_trips_json() {
    let dir = std::env::temp_dir().join("vice_test_layout_round_trip");
    fs::create_dir_all(&dir).unwrap();
    let layout_path = dir.join("main.vui.json");
    let layout_path = layout_path.to_string_lossy().replace('\\', "/");
    let source = format!(
        r#"
@entry
fn main() {{
    let layout => {{"button": {{"x": 10, "y": 20}}}};
    let saved => engine.save_layout("{}", layout);
    let loaded => engine.load_layout("{}");
    output <- saved, loaded["button"]["x"], loaded["button"]["y"];
}}
"#,
        layout_path, layout_path
    );

    let (_, output) = run_source(&source, vec![]).unwrap();
    assert_eq!(output, vec!["true 10 20"]);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn runtime_fs_read_json_matches_its_registered_contract() {
    let dir = std::env::temp_dir().join("vice_test_read_json");
    fs::create_dir_all(&dir).unwrap();
    let json_path = dir.join("config.json");
    fs::write(&json_path, r#"{"name":"Vice","count":3}"#).unwrap();
    let json_path = json_path.to_string_lossy().replace('\\', "/");
    let source = format!(
        r#"
@entry
fn main() {{
    let config => fs.read_json("{}");
    output <- config["name"], config["count"];
}}
"#,
        json_path
    );

    let (_, output) = run_source(&source, vec![]).unwrap();
    assert_eq!(output, vec!["Vice 3"]);
    fs::remove_dir_all(dir).unwrap();
}
