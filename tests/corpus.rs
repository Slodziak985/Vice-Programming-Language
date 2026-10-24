use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn vice() -> &'static str {
    env!("CARGO_BIN_EXE_vice")
}

fn run(command: &str, file: &Path) -> Output {
    Command::new(vice())
        .arg(command)
        .arg(file)
        .output()
        .unwrap_or_else(|error| panic!("cannot run Vice for '{}': {error}", file.display()))
}

fn assert_success(command: &str, file: &Path) {
    let output = run(command, file);
    assert!(
        output.status.success(),
        "vice {command} '{}' failed:\n{}",
        file.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn vice_files(directory: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read '{}': {error}", directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "vice")
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn every_example_passes_full_check_pipeline() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in vice_files(&root.join("examples")) {
        assert_success("check", &file);
    }
}

#[test]
fn migration_regression_programs_execute() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let migration_programs = [
        "test_lambda.vice",
        "test_arc.vice",
        "test_result_option.vice",
        "test_vm.vice",
    ];
    for file in migration_programs {
        assert_success(
            "run",
            &root
                .join("tests")
                .join("programs")
                .join("migration")
                .join(file),
        );
    }

    let examples = [
        "full_test.vice",
        "typed_imports.vice",
        "test_1_math_logic.vice",
        "test_2_arrays.vice",
        "test_3_maps.vice",
        "test_4_functions.vice",
        "test_5_structs.vice",
        "test_6_enums.vice",
        "test_7_error_handling.vice",
        "test_8_borrowing.vice",
        "test_9_traits.vice",
        "test_10_modules.vice",
    ];
    for file in examples {
        assert_success("run", &root.join("examples").join(file));
    }
}

#[test]
fn tasks_pass_full_check_pipeline() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = if root.join("challenges").exists() {
        root.join("challenges")
    } else if root.join("Zadania").exists() {
        root.join("Zadania")
    } else {
        return;
    };
    for file in vice_files(&dir) {
        assert_success("check", &file);
    }
}
