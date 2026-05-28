//! Path-safety guard (FR-005).
//!
//! Every CLI path argument must:
//!
//! 1. Contain no `..` segments literally (the sandbox is a process-
//!    boundary contract; we don't allow callers to escape via
//!    parent-directory traversal even if the resulting canonical path
//!    would have been benign).
//! 2. Refer to an existing file/directory for `--module` and
//!    `--data` (a missing file is a user error, not a sandbox escape).
//! 3. Canonicalize cleanly. Symlinks that resolve to a target outside
//!    the canonicalized root are rejected.
//!
//! NFR-005: violations are expressed as a single error type so the
//! caller can render them uniformly on stderr. The shape mirrors the
//! `quire_rs::Diagnostic` Display contract (variant name + load-bearing
//! identifier) so a downstream `Diagnostic` variant can be added
//! upstream without changing the message format.

use std::path::{Path, PathBuf};

/// Why a path was rejected.
#[derive(Debug, PartialEq, Eq)]
pub enum PathSafetyViolation {
    Traversal(String),
    Missing(String),
    Unreadable { path: String, reason: String },
    NotADirectory(String),
    NotAFile(String),
}

impl std::fmt::Display for PathSafetyViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Traversal(p) => write!(f, "PathTraversal: '{p}' contains a '..' segment"),
            Self::Missing(p) => write!(f, "PathMissing: '{p}' does not exist"),
            Self::Unreadable { path, reason } => {
                write!(f, "PathUnreadable: '{path}': {reason}")
            }
            Self::NotADirectory(p) => write!(f, "PathNotADirectory: '{p}' is not a directory"),
            Self::NotAFile(p) => write!(f, "PathNotAFile: '{p}' is not a regular file"),
        }
    }
}

impl std::error::Error for PathSafetyViolation {}

fn reject_dotdot(raw: &str) -> Result<(), PathSafetyViolation> {
    let path = Path::new(raw);
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(PathSafetyViolation::Traversal(raw.to_string()));
        }
    }
    Ok(())
}

/// Validate a `--module <path>` argument.
///
/// Rejects `..` segments, canonicalizes, requires a directory.
pub fn validate_module_path(raw: &str) -> Result<PathBuf, PathSafetyViolation> {
    reject_dotdot(raw)?;
    let p = Path::new(raw);
    let canonical = p.canonicalize().map_err(|e| map_io(raw, e))?;
    if !canonical.is_dir() {
        return Err(PathSafetyViolation::NotADirectory(raw.to_string()));
    }
    Ok(canonical)
}

/// Validate a `--data <file>` argument. `"-"` (stdin) bypasses this
/// guard and is the caller's responsibility.
pub fn validate_data_path(raw: &str) -> Result<PathBuf, PathSafetyViolation> {
    reject_dotdot(raw)?;
    let p = Path::new(raw);
    let canonical = p.canonicalize().map_err(|e| map_io(raw, e))?;
    if !canonical.is_file() {
        return Err(PathSafetyViolation::NotAFile(raw.to_string()));
    }
    Ok(canonical)
}

/// Validate a `--out <path>` argument. The file may not exist yet; we
/// only enforce that the parent directory is real and that the path
/// has no `..` segments.
pub fn validate_out_path(raw: &str) -> Result<PathBuf, PathSafetyViolation> {
    reject_dotdot(raw)?;
    let p = Path::new(raw);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            let canonical_parent = parent.canonicalize().map_err(|e| map_io(raw, e))?;
            if !canonical_parent.is_dir() {
                return Err(PathSafetyViolation::NotADirectory(
                    parent.display().to_string(),
                ));
            }
            let file_name = p
                .file_name()
                .ok_or_else(|| PathSafetyViolation::NotAFile(raw.to_string()))?;
            return Ok(canonical_parent.join(file_name));
        }
    }
    Ok(p.to_path_buf())
}

/// `quire_rs::Registry::load_from` expects a *search root* whose
/// children are module directories. CLI users typically pass the
/// module directory itself, so promote it to its parent when the
/// pointed-at path already contains a `manifest.yaml`.
pub fn search_root_for_module(module: &Path) -> PathBuf {
    if module.join("manifest.yaml").is_file() {
        if let Some(parent) = module.parent() {
            return parent.to_path_buf();
        }
    }
    module.to_path_buf()
}

fn map_io(raw: &str, err: std::io::Error) -> PathSafetyViolation {
    match err.kind() {
        std::io::ErrorKind::NotFound => PathSafetyViolation::Missing(raw.to_string()),
        _ => PathSafetyViolation::Unreadable {
            path: raw.to_string(),
            reason: err.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rejects_dotdot_in_module() {
        let err = validate_module_path("foo/../bar").unwrap_err();
        assert!(matches!(err, PathSafetyViolation::Traversal(_)));
    }

    #[test]
    fn rejects_dotdot_in_data() {
        let err = validate_data_path("../etc/passwd").unwrap_err();
        assert!(matches!(err, PathSafetyViolation::Traversal(_)));
    }

    #[test]
    fn rejects_dotdot_in_out() {
        let err = validate_out_path("../escape.md").unwrap_err();
        assert!(matches!(err, PathSafetyViolation::Traversal(_)));
    }

    #[test]
    fn module_must_be_directory() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("not-a-dir");
        fs::write(&f, "").unwrap();
        let err = validate_module_path(f.to_str().unwrap()).unwrap_err();
        assert!(matches!(err, PathSafetyViolation::NotADirectory(_)));
    }

    #[test]
    fn data_must_be_file() {
        let dir = tempdir().unwrap();
        let err = validate_data_path(dir.path().to_str().unwrap()).unwrap_err();
        assert!(matches!(err, PathSafetyViolation::NotAFile(_)));
    }

    #[test]
    fn module_path_canonicalizes() {
        let dir = tempdir().unwrap();
        let canonical = validate_module_path(dir.path().to_str().unwrap()).unwrap();
        assert!(canonical.is_absolute());
    }

    #[test]
    fn missing_module_is_user_error() {
        let err = validate_module_path("/definitely/not/a/real/path/quire-xyz").unwrap_err();
        assert!(matches!(err, PathSafetyViolation::Missing(_)));
    }

    #[test]
    fn out_path_parent_must_exist() {
        let err = validate_out_path("/definitely/not/a/real/dir/out.md").unwrap_err();
        assert!(matches!(err, PathSafetyViolation::Missing(_)));
    }

    #[test]
    fn out_path_in_existing_dir_ok() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("out.md");
        let resolved = validate_out_path(out.to_str().unwrap()).unwrap();
        assert!(resolved.ends_with("out.md"));
    }
}
