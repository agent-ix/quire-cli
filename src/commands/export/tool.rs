//! Locating the external executables `quire export` names (FR-032, FR-033).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::failure;

/// An executable resolved to its canonical path.
#[derive(Debug, Clone)]
pub(super) struct Tool {
    pub(super) name: &'static str,
    pub(super) path: PathBuf,
    /// `path` as UTF-8 text, the form the producer request carries.
    pub(super) executable: String,
}

/// Resolves `name` to a canonical regular executable file: the `flag` path when
/// given, otherwise the first PATH match. Every symbolic link in every
/// component is resolved because the executor refuses to follow one. A tool
/// that is absent or is not a regular executable file after resolution is a
/// missing converter (exit 3).
pub(super) fn resolve(
    name: &'static str,
    flag: &'static str,
    explicit: Option<&Path>,
) -> anyhow::Result<Tool> {
    let (candidate, looked) = match explicit {
        Some(path) => (
            Some(path.to_path_buf()),
            format!("{flag} '{}'", path.display()),
        ),
        None => {
            let path = env::var_os("PATH").unwrap_or_default();
            let found = env::split_paths(&path)
                .map(|dir| dir.join(name))
                .find(|candidate| is_executable(candidate));
            (found, format!("PATH '{}'", path.to_string_lossy()))
        }
    };
    let canonical = candidate
        .and_then(|path| fs::canonicalize(path).ok())
        .filter(|path| is_executable(path));
    match canonical {
        Some(path) => {
            let executable = path
                .to_str()
                .ok_or_else(|| {
                    failure::invalid(anyhow::anyhow!(
                        "'{name}' resolves to a path that is not valid UTF-8 ({}); \
                         the producer request cannot carry it",
                        path.display()
                    ))
                })?
                .to_string();
            Ok(Tool {
                name,
                path,
                executable,
            })
        }
        None => Err(failure::invalid(anyhow::anyhow!(
            "missing converter: '{name}' is not an executable file; looked at {looked}; \
             pass {flag} <PATH> to name it"
        ))),
    }
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file() && has_execute_permission(&meta))
}

fn has_execute_permission(meta: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        true
    }
}
