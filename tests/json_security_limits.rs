use std::fs;
use std::path::{Path, PathBuf};

use vice::stdlib;

macro_rules! vice_case_count {
    ($name:ident, $count:expr) => {
        #[allow(dead_code)]
        const $name: usize = $count;
    };
}

vice_case_count!(JSON_SECURITY_LIMIT_CASES, 24);

struct SandboxReset;

impl Drop for SandboxReset {
    fn drop(&mut self) {
        let _ = stdlib::set_sandbox_root(None);
    }
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn vice_string(value: impl AsRef<str>) -> String {
    format!("{:?}", value.as_ref().replace('\\', "/"))
}

fn run_error(source: &str) -> String {
    vice::run_source(source, vec![]).unwrap_err().message
}

fn deep_json(depth: usize) -> String {
    format!("{}0{}", "[".repeat(depth), "]".repeat(depth))
}

fn write_file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

#[test]
fn json_parse_rejects_excessive_depth_without_panicking() {
    let json = deep_json(stdlib::JSON_MAX_DEPTH + 8);
    let source = format!(
        "@entry\nfn main() {{\n    let value => json.parse({});\n    emit <- value;\n}}\n",
        vice_string(json)
    );
    let error = run_error(&source);
    assert!(error.contains("JSON depth limit exceeded"), "{error}");
}

#[test]
fn json_try_parse_returns_err_for_excessive_depth() {
    let json = deep_json(stdlib::JSON_MAX_DEPTH + 8);
    let source = format!(
        "@entry\nfn main() {{\n    let result => json.try_parse({});\n    emit <- result.is_ok, result.error.message;\n}}\n",
        vice_string(json)
    );
    let (_, output) = vice::run_source(&source, vec![]).unwrap();
    assert_eq!(output.len(), 1);
    assert!(output[0].starts_with("false JSON depth limit exceeded"));
}

#[test]
fn json_parse_rejects_oversized_input_without_panicking() {
    let json = format!("\"{}\"", "x".repeat(stdlib::JSON_MAX_INPUT_BYTES + 1));
    let source = format!(
        "@entry\nfn main() {{\n    let value => json.parse({});\n    emit <- value;\n}}\n",
        vice_string(json)
    );
    let error = run_error(&source);
    assert!(error.contains("JSON input limit"), "{error}");
}

#[test]
fn fs_read_json_rejects_oversized_file_without_panicking() {
    let dir = unique_temp_dir("vice_json_limit");
    let path = dir.join("huge.json");
    write_file(
        &path,
        &format!("\"{}\"", "x".repeat(stdlib::JSON_MAX_INPUT_BYTES + 1)),
    );
    let source = format!(
        "@entry\nfn main() {{\n    let value => fs.read_json({});\n    emit <- value;\n}}\n",
        vice_string(path.display().to_string())
    );
    let error = run_error(&source);
    assert!(error.contains("JSON input limit"), "{error}");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn json_stringify_rejects_excessive_depth_without_panicking() {
    let mut source = String::from("@entry\nfn main() {\n    let value => 0;\n");
    for i in 0..(stdlib::JSON_MAX_DEPTH + 8) {
        source.push_str(&format!("    let value_{i} => ["));
        if i == 0 {
            source.push_str("value");
        } else {
            source.push_str(&format!("value_{}", i - 1));
        }
        source.push_str("];\n");
    }
    source.push_str(&format!(
        "    emit <- json.stringify(value_{});\n}}\n",
        stdlib::JSON_MAX_DEPTH + 7
    ));
    let error = run_error(&source);
    assert!(error.contains("JSON depth limit exceeded"), "{error}");
}

#[test]
fn engine_scene_and_layout_io_respect_sandbox_root() {
    let parent = unique_temp_dir("vice_engine_sandbox");
    let project = parent.join("project");
    let outside = parent.join("outside.vscene");
    fs::create_dir_all(&project).unwrap();
    write_file(&outside, "{\"version\":1,\"entities\":[]}");
    stdlib::set_sandbox_root(Some(project.clone())).unwrap();
    let _reset = SandboxReset;

    let scene_error = run_error(
        "@entry\nfn main() {\n    let scene => engine.load_scene(\"../outside.vscene\");\n    emit <- scene;\n}\n",
    );
    assert!(scene_error.contains("Sandbox denied"), "{scene_error}");

    let layout_error = run_error(
        "@entry\nfn main() {\n    let layout => engine.load_layout(\"../outside.vscene\");\n    emit <- layout;\n}\n",
    );
    assert!(layout_error.contains("Sandbox denied"), "{layout_error}");

    let save_error = run_error(
        "@entry\nfn main() {\n    engine.save_scene(\"../saved.vscene\", {\"version\": 1});\n}\n",
    );
    assert!(save_error.contains("Sandbox denied"), "{save_error}");

    fs::remove_dir_all(parent).unwrap();
}
