use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    pub source_dir: PathBuf,
    pub target_dir: PathBuf,
    pub binaries: Vec<BinaryInstall>,
    pub std_source: Option<PathBuf>,
    pub std_target: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryInstall {
    pub name: String,
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug)]
pub enum InstallError {
    MissingReleaseBinary { path: PathBuf },
    MissingStdLibrary { path: PathBuf },
    MissingInstallDir,
    Io(std::io::Error),
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallError::MissingReleaseBinary { path } => write!(
                f,
                "Missing '{}'. Run 'cargo build --release' from the vice_parser folder first.",
                path.display()
            ),
            InstallError::MissingStdLibrary { path } => write!(
                f,
                "Missing standard library '{}'. Package the std folder next to the release binaries.",
                path.display()
            ),
            InstallError::MissingInstallDir => write!(f, "Cannot determine user install folder"),
            InstallError::Io(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for InstallError {}

impl From<std::io::Error> for InstallError {
    fn from(value: std::io::Error) -> Self {
        InstallError::Io(value)
    }
}

pub fn build_install_plan(
    project_root: &Path,
    target_dir: Option<PathBuf>,
) -> Result<InstallPlan, InstallError> {
    let source_dir = project_root.join("target").join("release");
    build_install_plan_from_source(&source_dir, target_dir)
}

pub fn build_install_plan_from_source(
    source_dir: &Path,
    target_dir: Option<PathBuf>,
) -> Result<InstallPlan, InstallError> {
    let target_dir = match target_dir {
        Some(path) => path,
        None => default_install_dir().ok_or(InstallError::MissingInstallDir)?,
    };

    let mut binaries = Vec::new();
    for name in binary_names() {
        let source = source_dir.join(name);
        if !source.exists() {
            return Err(InstallError::MissingReleaseBinary { path: source });
        }
        binaries.push(BinaryInstall {
            name: name.to_string(),
            target: target_dir.join(name),
            source,
        });
    }

    let std_source = source_dir.join("std");
    if !std_source.exists() {
        return Err(InstallError::MissingStdLibrary { path: std_source });
    }

    Ok(InstallPlan {
        source_dir: source_dir.to_path_buf(),
        std_target: target_dir.join("std"),
        target_dir,
        binaries,
        std_source: Some(std_source),
    })
}

pub fn install_from_plan(plan: &InstallPlan) -> Result<Vec<PathBuf>, InstallError> {
    fs::create_dir_all(&plan.target_dir)?;
    let mut installed = Vec::new();
    for binary in &plan.binaries {
        fs::copy(&binary.source, &binary.target)?;
        ensure_executable(&binary.target)?;
        installed.push(binary.target.clone());
    }
    if let Some(std_source) = &plan.std_source {
        copy_dir_all(std_source, &plan.std_target)?;
        installed.push(plan.std_target.clone());
    }
    Ok(installed)
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<(), InstallError> {
    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn find_existing_binaries_in_path(name: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Some(paths_os) = env::var_os("PATH") {
        for p in env::split_paths(&paths_os) {
            let candidate = p.join(name);
            if candidate.exists() {
                found.push(candidate);
            }
        }
    }
    found
}

/// Install and also replace existing binaries found earlier in PATH (if writable).
pub fn install_and_replace_from_plan(plan: &InstallPlan) -> Result<Vec<PathBuf>, InstallError> {
    // First install to the plan.target_dir as usual
    let mut installed = install_from_plan(plan)?;

    // Then try to replace any existing copies found in PATH (outside the target_dir)
    for binary in &plan.binaries {
        let existing_paths = find_existing_binaries_in_path(&binary.name);
        for existing in existing_paths {
            // Skip the one we just installed
            if existing == binary.target {
                continue;
            }

            // Attempt to back up previous file (vice.exe -> vice.exe.old)
            if let Some(file_name) = existing.file_name() {
                let backup_name = format!("{}.old", file_name.to_string_lossy());
                let backup_path = existing.with_file_name(backup_name);
                let _ = fs::rename(&existing, &backup_path);
            }

            // Try copy new binary over the old location
            match fs::copy(&binary.source, &existing) {
                Ok(_) => {
                    // Ensure executable bit on unix-like systems
                    #[cfg(unix)]
                    {
                        ensure_executable(&existing)?;
                    }
                    installed.push(existing.clone());
                }
                Err(_) => {
                    // Ignore failures to overwrite (likely permissions); leave as-is
                }
            }
        }
    }

    Ok(installed)
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<(), InstallError> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn ensure_executable(_path: &Path) -> Result<(), InstallError> {
    Ok(())
}

pub fn default_install_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("Vice").join("bin"))
    }

    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|path| path.join(".local").join("bin"))
    }
}

pub fn path_hint(target_dir: &Path) -> String {
    #[cfg(windows)]
    {
        format!(
            "Add '{}' to your user PATH if 'vice' is not found in a new terminal.",
            target_dir.display()
        )
    }

    #[cfg(not(windows))]
    {
        format!(
            "Add '{}' to PATH if 'vice' is not found in a new terminal.",
            target_dir.display()
        )
    }
}

pub fn binary_names() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &["vice.exe", "vicetool.exe"]
    }

    #[cfg(not(windows))]
    {
        &["vice", "vicetool"]
    }
}
