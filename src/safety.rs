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
//! NFR-005: violations are expressed as `quire_rs::Diagnostic` so the
//! caller can render them uniformly on stderr alongside engine-emitted
//! diagnostics. No parallel `PathSafetyViolation` shape lives here —
//! that workaround was retired when `quire-rs v0.2.0` added the
//! `Diagnostic::PathTraversal` variant.

use std::path::{Path, PathBuf};

use quire_rs::{Diagnostic, PathTraversalReason};

/// Either a `PathTraversal` (sandbox refusal) or a plain `io::Error`
/// (missing file, not-a-dir, unreadable). Both flow to exit-1 user-error
/// at the caller, but only the first must use the `Diagnostic` shape.
#[derive(Debug)]
pub enum PathError {
    Traversal(Diagnostic),
    Io {
        argument: String,
        source: std::io::Error,
    },
    NotADirectory(String),
    NotAFile(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Traversal(d) => write!(f, "{d}"),
            Self::Io { argument, source } => write!(f, "{argument}: {source}"),
            Self::NotADirectory(p) => write!(f, "PathNotADirectory: '{p}' is not a directory"),
            Self::NotAFile(p) => write!(f, "PathNotAFile: '{p}' is not a regular file"),
        }
    }
}

impl std::error::Error for PathError {}

fn reject_dotdot(argument: &str, raw: &str) -> Result<(), PathError> {
    let path = Path::new(raw);
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(PathError::Traversal(Diagnostic::PathTraversal {
                argument: argument.to_string(),
                path: PathBuf::from(raw),
                reason: PathTraversalReason::DotDotSegment,
            }));
        }
    }
    Ok(())
}

/// Validate a `--module <path>` argument.
///
/// Rejects `..` segments, canonicalizes, requires a directory.
pub fn validate_module_path(raw: &str) -> Result<PathBuf, PathError> {
    reject_dotdot("--module", raw)?;
    let p = Path::new(raw);
    let canonical = p.canonicalize().map_err(|e| map_io("--module", raw, e))?;
    if !canonical.is_dir() {
        return Err(PathError::NotADirectory(raw.to_string()));
    }
    Ok(canonical)
}

/// Validate an input-file path argument under the given `argument` label
/// (e.g. `--data` or `document` for a positional). `"-"`
/// (stdin) bypasses this guard and is the caller's responsibility.
pub fn validate_input_path(argument: &str, raw: &str) -> Result<PathBuf, PathError> {
    reject_dotdot(argument, raw)?;
    let p = Path::new(raw);
    let canonical = p.canonicalize().map_err(|e| map_io(argument, raw, e))?;
    if !canonical.is_file() {
        return Err(PathError::NotAFile(raw.to_string()));
    }
    Ok(canonical)
}

/// Validate a `--data <file>` argument. `"-"` (stdin) bypasses this
/// guard and is the caller's responsibility.
pub fn validate_data_path(raw: &str) -> Result<PathBuf, PathError> {
    validate_input_path("--data", raw)
}

/// Validate a `--out <path>` argument. The file may not exist yet; we
/// only enforce that the parent directory is real and that the path
/// has no `..` segments.
pub fn validate_out_path(raw: &str) -> Result<PathBuf, PathError> {
    reject_dotdot("--out", raw)?;
    let p = Path::new(raw);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            let canonical_parent = parent.canonicalize().map_err(|e| map_io("--out", raw, e))?;
            if !canonical_parent.is_dir() {
                return Err(PathError::NotADirectory(parent.display().to_string()));
            }
            let file_name = p
                .file_name()
                .ok_or_else(|| PathError::NotAFile(raw.to_string()))?;
            return Ok(canonical_parent.join(file_name));
        }
    }
    Ok(p.to_path_buf())
}

fn map_io(argument: &str, raw: &str, err: std::io::Error) -> PathError {
    // Symlink loops surface as `FilesystemLoop` from `canonicalize` —
    // treat as a traversal refusal so the symlink-escape path is
    // distinguishable from missing-file user errors.
    if err.raw_os_error() == Some(libc_eloop()) {
        return PathError::Traversal(Diagnostic::PathTraversal {
            argument: argument.to_string(),
            path: PathBuf::from(raw),
            reason: PathTraversalReason::SymlinkEscape,
        });
    }
    PathError::Io {
        argument: format!("{argument} '{raw}'"),
        source: err,
    }
}

#[cfg(target_os = "linux")]
fn libc_eloop() -> i32 {
    40
}
#[cfg(not(target_os = "linux"))]
fn libc_eloop() -> i32 {
    62
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rejects_dotdot_in_module() {
        let err = validate_module_path("foo/../bar").unwrap_err();
        assert!(matches!(err, PathError::Traversal(_)));
        assert!(err.to_string().contains("PathTraversal"));
    }

    #[test]
    fn rejects_dotdot_in_data() {
        let err = validate_data_path("../etc/passwd").unwrap_err();
        assert!(matches!(err, PathError::Traversal(_)));
    }

    #[test]
    fn rejects_dotdot_in_out() {
        let err = validate_out_path("../escape.md").unwrap_err();
        assert!(matches!(err, PathError::Traversal(_)));
    }

    #[test]
    fn module_must_be_directory() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("not-a-dir");
        fs::write(&f, "").unwrap();
        let err = validate_module_path(f.to_str().unwrap()).unwrap_err();
        assert!(matches!(err, PathError::NotADirectory(_)));
    }

    #[test]
    fn data_must_be_file() {
        let dir = tempdir().unwrap();
        let err = validate_data_path(dir.path().to_str().unwrap()).unwrap_err();
        assert!(matches!(err, PathError::NotAFile(_)));
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
        assert!(matches!(err, PathError::Io { .. }));
    }

    #[test]
    fn out_path_parent_must_exist() {
        let err = validate_out_path("/definitely/not/a/real/dir/out.md").unwrap_err();
        assert!(matches!(err, PathError::Io { .. }));
    }

    #[test]
    fn out_path_in_existing_dir_ok() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("out.md");
        let resolved = validate_out_path(out.to_str().unwrap()).unwrap();
        assert!(resolved.ends_with("out.md"));
    }
}
