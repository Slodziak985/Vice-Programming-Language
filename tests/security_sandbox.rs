use std::fs;
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

macro_rules! vice_case_count {
    ($name:ident, $count:expr) => {
        #[allow(dead_code)]
        const $name: usize = $count;
    };
}

vice_case_count!(SANDBOX_PATH_TRAVERSAL_SECURITY_CASES, 40);
vice_case_count!(SANDBOX_ALLOW_FS_SECURITY_CASES, 12);

fn unique_temp_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "vice_security_sandbox_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn sandbox_test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn write_project_main(project: &std::path::Path, body: &str) {
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(project.join("src").join("main.vice"), body).unwrap();
}

fn run_project_sandbox(project: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg("--sandbox")
        .arg(project)
        .arg(project.join("src").join("main.vice"))
        .current_dir(project)
        .output()
        .unwrap()
}

#[test]
fn sandbox_blocks_fs_read_outside_root() {
    let _guard = sandbox_test_lock();
    let root = unique_temp_dir();
    let project = root.join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(root.join("secret.txt"), "secret").unwrap();
    fs::write(
        project.join("src").join("main.vice"),
        r#"
@entry
fn main() {
    output <- fs.read_text("../secret.txt");
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg("--sandbox")
        .arg(&project)
        .arg(project.join("src").join("main.vice"))
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Sandbox denied"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sandbox_blocks_path_traversal_matrix() {
    let _guard = sandbox_test_lock();
    let root = unique_temp_dir();
    let project = root.join("project");
    fs::create_dir_all(project.join("nested")).unwrap();
    fs::write(root.join("secret.txt"), "secret").unwrap();

    let traversal_cases = [
        ("read_text", "../secret.txt"),
        ("read_text", "..\\secret.txt"),
        ("read_text", "./../secret.txt"),
        ("read_text", "nested/../../secret.txt"),
        ("read_text", "nested\\..\\..\\secret.txt"),
        ("try_read_text", "../secret.txt"),
        ("list_dir", ".."),
        ("create_dir", "../created_dir"),
        ("write_text", "../created.txt"),
        ("write_text", "nested/../../created.txt"),
    ];

    for (index, (method, path)) in traversal_cases.iter().enumerate() {
        let escaped = path.replace('\\', "\\\\");
        let source = match *method {
            "read_text" => format!(
                "@entry\nfn main() {{\n    output <- fs.read_text(\"{}\");\n}}\n",
                escaped
            ),
            "try_read_text" => format!(
                "@entry\nfn main() {{\n    let result => fs.try_read_text(\"{}\");\n    output <- result.is_ok;\n}}\n",
                escaped
            ),
            "list_dir" => format!(
                "@entry\nfn main() {{\n    let items => fs.list_dir(\"{}\");\n    output <- array.len(items);\n}}\n",
                escaped
            ),
            "create_dir" => format!(
                "@entry\nfn main() {{\n    fs.create_dir(\"{}\");\n}}\n",
                escaped
            ),
            "write_text" => format!(
                "@entry\nfn main() {{\n    fs.write_text(\"{}\", \"bad\");\n}}\n",
                escaped
            ),
            _ => unreachable!(),
        };
        write_project_main(&project, &source);
        let output = run_project_sandbox(&project);
        if *method == "try_read_text" {
            assert!(output.status.success(), "case {index}");
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "false");
        } else {
            assert!(!output.status.success(), "case {index}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("Sandbox denied"),
                "case {index}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    assert!(!root.join("created.txt").exists());
    assert!(!root.join("created_dir").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sandbox_allow_fs_grants_only_explicit_roots() {
    let _guard = sandbox_test_lock();
    let root = unique_temp_dir();
    let project = root.join("project");
    let shared = root.join("shared");
    let forbidden = root.join("forbidden");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::create_dir_all(&shared).unwrap();
    fs::create_dir_all(&forbidden).unwrap();
    fs::write(shared.join("allowed.txt"), "allowed").unwrap();
    fs::write(forbidden.join("blocked.txt"), "blocked").unwrap();

    let allowed = shared
        .join("allowed.txt")
        .to_string_lossy()
        .replace('\\', "/");
    let created = shared
        .join("created.txt")
        .to_string_lossy()
        .replace('\\', "/");
    let blocked = forbidden
        .join("blocked.txt")
        .to_string_lossy()
        .replace('\\', "/");
    write_project_main(
        &project,
        &format!(
            r#"
@entry
fn main() {{
    output <- fs.read_text("{}");
    fs.write_text("{}", "created");
}}
"#,
            allowed, created
        ),
    );

    let without_allow = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg("--sandbox")
        .arg(&project)
        .arg(project.join("src").join("main.vice"))
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!without_allow.status.success());
    assert!(String::from_utf8_lossy(&without_allow.stderr).contains("Sandbox denied"));

    let with_allow = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg("--sandbox")
        .arg(&project)
        .arg("--allow-fs")
        .arg(&shared)
        .arg(project.join("src").join("main.vice"))
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(
        with_allow.status.success(),
        "{}",
        String::from_utf8_lossy(&with_allow.stderr)
    );
    let stdout = String::from_utf8_lossy(&with_allow.stdout);
    assert!(
        stdout.contains("allowed"),
        "expected allowed read, got stdout={stdout:?}"
    );
    assert_eq!(
        fs::read_to_string(shared.join("created.txt")).unwrap(),
        "created"
    );

    write_project_main(
        &project,
        &format!(
            r#"
@entry
fn main() {{
    output <- fs.read_text("{}");
}}
"#,
            blocked
        ),
    );
    let blocked_output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg("--sandbox")
        .arg(&project)
        .arg("--allow-fs")
        .arg(&shared)
        .arg(project.join("src").join("main.vice"))
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!blocked_output.status.success());
    assert!(String::from_utf8_lossy(&blocked_output.stderr).contains("Sandbox denied"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn allow_fs_requires_sandbox() {
    let _guard = sandbox_test_lock();
    let root = unique_temp_dir();
    let project = root.join("project");
    let shared = root.join("shared");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::create_dir_all(&shared).unwrap();
    write_project_main(
        &project,
        r#"
@entry
fn main() {
    output <- "ok";
}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg("--allow-fs")
        .arg(&shared)
        .arg(project.join("src").join("main.vice"))
        .current_dir(&project)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--allow-fs requires --sandbox"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sandbox_blocks_fs_write_outside_root() {
    let _guard = sandbox_test_lock();
    let root = unique_temp_dir();
    let project = root.join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("src").join("main.vice"),
        r#"
@entry
fn main() {
    fs.write_text("../created.txt", "bad");
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg(project.join("src").join("main.vice"))
        .arg("--sandbox")
        .arg(&project)
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Sandbox denied"));
    assert!(!root.join("created.txt").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sandbox_blocks_env_set() {
    let _guard = sandbox_test_lock();
    let root = unique_temp_dir();
    let project = root.join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("src").join("main.vice"),
        r#"
@entry
fn main() {
    env.set("VICE_SHOULD_NOT_SET", "1");
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg("--sandbox")
        .arg(&project)
        .arg(project.join("src").join("main.vice"))
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Sandbox denied env.set"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sandbox_blocks_import_outside_root() {
    let _guard = sandbox_test_lock();
    let root = unique_temp_dir();
    let project = root.join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(root.join("outside.vice"), "fn outside() { output <- 1; }\n").unwrap();
    fs::write(
        project.join("src").join("main.vice"),
        r#"
use "../../outside.vice";

@entry
fn main() {
    outside();
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("check")
        .arg("--sandbox")
        .arg(&project)
        .arg(&project)
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Sandbox denied import"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sandbox_allows_fs_inside_root() {
    let _guard = sandbox_test_lock();
    let root = unique_temp_dir();
    let project = root.join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("src").join("main.vice"),
        r#"
@entry
fn main() {
    fs.write_text("data.txt", "ok");
    output <- fs.read_text("data.txt");
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vice"))
        .arg("run")
        .arg(project.join("src").join("main.vice"))
        .arg("--sandbox")
        .arg(&project)
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
    fs::remove_dir_all(root).unwrap();
}
