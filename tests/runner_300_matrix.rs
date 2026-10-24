use std::fs;
use std::path::{Path, PathBuf};
use vice::{execute_program_vm, load_program};

fn get_suite_dir(suite: &str) -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("tests").join("programs").join(suite)
}

fn run_suite(suite_name: &str, expected_count: usize) {
    let dir = get_suite_dir(suite_name);
    assert!(
        dir.is_dir(),
        "Suite directory '{}' does not exist",
        dir.display()
    );

    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("read_dir failed")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|s| s == "vice").unwrap_or(false))
        .collect();

    files.sort();
    assert_eq!(
        files.len(),
        expected_count,
        "Suite '{}' expected {} tests, found {}",
        suite_name,
        expected_count,
        files.len()
    );

    let mut failed = Vec::new();
    let mut passed = 0;

    for file in &files {
        match load_program(file) {
            Ok(program) => match execute_program_vm(&program) {
                Ok(output) => {
                    let has_pass = output.iter().any(|line| line.trim() == "PASS");
                    if has_pass {
                        passed += 1;
                    } else {
                        failed.push((
                            file.display().to_string(),
                            format!("Program ran but output did not contain 'PASS': {:?}", output),
                        ));
                    }
                }
                Err(err) => {
                    failed.push((
                        file.display().to_string(),
                        format!("VM/Runtime Error: {:?}", err),
                    ));
                }
            },
            Err(err) => {
                failed.push((
                    file.display().to_string(),
                    format!("Load/Semantic Error: {:?}", err),
                ));
            }
        }
    }

    if !failed.is_empty() {
        eprintln!("\n=== FAILURES IN SUITE '{}' ({} failed, {} passed) ===", suite_name, failed.len(), passed);
        for (f, err) in &failed {
            eprintln!("[-] {}: {}", f, err);
        }
        panic!(
            "Suite '{}': {} of {} programs failed!",
            suite_name,
            failed.len(),
            files.len()
        );
    }

    println!(
        "\n[SUCCESS] Suite '{}': All {} programs passed perfectly!",
        suite_name, passed
    );
}

#[test]
fn test_suite_simple_100() {
    run_suite("simple", 100);
}

#[test]
fn test_suite_medium_100() {
    run_suite("medium", 100);
}

#[test]
fn test_suite_heavy_100() {
    run_suite("heavy", 100);
}
