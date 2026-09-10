use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Enumerate regular Rust files below named roots without following symlinks.
pub(crate) fn rust_files(
    repository: &Path,
    roots: &[&str],
    excluded_directories: &[&str],
) -> Result<Vec<PathBuf>> {
    fn walk(directory: &Path, excluded: &[&str], files: &mut Vec<PathBuf>) -> Result<()> {
        let entries = fs::read_dir(directory).map_err(|source| Error::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| Error::ReadDirectoryEntry {
                path: directory.to_path_buf(),
                source,
            })?;
            let file_type = entry.file_type().map_err(|source| Error::InspectPath {
                path: entry.path(),
                source,
            })?;
            if file_type.is_symlink() {
                return Err(Error::SymlinkSource { path: entry.path() });
            }
            if file_type.is_dir() {
                if excluded
                    .iter()
                    .any(|name| entry.file_name() == OsStr::new(name))
                {
                    continue;
                }
                walk(&entry.path(), excluded, files)?;
            } else if file_type.is_file()
                && entry.path().extension().and_then(OsStr::to_str) == Some("rs")
            {
                files.push(entry.path());
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    for root in roots {
        let directory = repository.join(root);
        if directory.is_dir() {
            walk(&directory, excluded_directories, &mut files)?;
        }
    }
    files.sort();
    Ok(files)
}
