use std::fs;
use std::path::Path;

pub fn add_dependency(root: &Path, source: &str) -> Result<String, String> {
    let manifest_path = root.join("vice.toml");
    if !manifest_path.is_file() {
        return Err(format!(
            "No 'vice.toml' found in '{}'. Run 'vice init' first.",
            root.display()
        ));
    }

    let source_path = Path::new(source);
    let name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Invalid dependency source name: '{}'", source))?;

    let modules_dir = root.join("modules");
    if !modules_dir.exists() {
        fs::create_dir_all(&modules_dir)
            .map_err(|e| format!("Failed to create modules directory: {}", e))?;
    }

    // If source is a local path
    if source_path.exists() {
        if source_path.is_file() {
            let dest = modules_dir.join(format!("{}.vice", name));
            fs::copy(source_path, &dest)
                .map_err(|e| format!("Failed to copy module file to '{}': {}", dest.display(), e))?;
        } else if source_path.is_dir() {
            let dest_dir = modules_dir.join(name);
            copy_dir_recursive(source_path, &dest_dir)?;
        }
    } else {
        // Create a stub if it's a new package name
        let dest = modules_dir.join(format!("{}.vice", name));
        if !dest.exists() {
            fs::write(&dest, format!("# Package: {}\n# Installed via 'vice pkg add'\n", name))
                .map_err(|e| format!("Failed to write package file: {}", e))?;
        }
    }

    // Update vice.toml
    let mut toml_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read 'vice.toml': {}", e))?;

    if !toml_content.contains("[dependencies]") {
        toml_content.push_str("\n[dependencies]\n");
    }

    let dep_entry = format!("{} = {{ path = \"{}\" }}\n", name, source);
    if !toml_content.contains(&format!("{} =", name)) {
        toml_content.push_str(&dep_entry);
        fs::write(&manifest_path, &toml_content)
            .map_err(|e| format!("Failed to update 'vice.toml': {}", e))?;
    }

    // Update vice.lock
    update_lockfile(root, name, source)?;

    Ok(format!(
        "Package '{}' successfully added and installed into modules/.",
        name
    ))
}

pub fn list_dependencies(root: &Path) -> Result<String, String> {
    let manifest_path = root.join("vice.toml");
    if !manifest_path.is_file() {
        return Err(format!("No 'vice.toml' found in '{}'", root.display()));
    }

    let toml_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read 'vice.toml': {}", e))?;

    let mut out = String::from("Dependencies in vice.toml:\n");
    let mut in_deps = false;
    let mut count = 0;

    for line in toml_content.lines() {
        let trimmed = line.trim();
        if trimmed == "[dependencies]" {
            in_deps = true;
            continue;
        } else if trimmed.starts_with('[') {
            in_deps = false;
        }

        if in_deps && !trimmed.is_empty() && !trimmed.starts_with('#') {
            out.push_str(&format!("  - {}\n", trimmed));
            count += 1;
        }
    }

    let modules_dir = root.join("modules");
    if modules_dir.is_dir() {
        out.push_str("\nInstalled in modules/:\n");
        if let Ok(entries) = fs::read_dir(&modules_dir) {
            for entry in entries.flatten() {
                out.push_str(&format!("  * {}\n", entry.file_name().to_string_lossy()));
            }
        }
    }

    if count == 0 {
        out.push_str("  (No dependencies declared)\n");
    }

    Ok(out)
}

fn update_lockfile(root: &Path, name: &str, source: &str) -> Result<(), String> {
    let lock_path = root.join("vice.lock");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let lock_json = serde_json::json!({
        "version": 1,
        "packages": {
            name: {
                "source": source,
                "installed_at": timestamp
            }
        }
    });

    fs::write(
        &lock_path,
        serde_json::to_string_pretty(&lock_json).unwrap_or_else(|_| "{}".to_string()),
    )
    .map_err(|e| format!("Failed to write 'vice.lock': {}", e))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Cannot create dir '{}': {}", dst.display(), e))?;
    }
    let entries = fs::read_dir(src)
        .map_err(|e| format!("Cannot read dir '{}': {}", src.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target)
                .map_err(|e| format!("Failed to copy file to '{}': {}", target.display(), e))?;
        }
    }
    Ok(())
}
