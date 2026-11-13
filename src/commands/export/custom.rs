//! The organisation's own template (FR-036): a file the user holds locally,
//! read once into memory and written only into the capability root.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use super::Format;
use crate::commands::failure;

/// The largest template or approval file accepted.
const MAX_INPUT_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// The extension a template for `to` must have.
fn extension(to: Format) -> &'static str {
    match to {
        Format::Pdf => "typ",
        Format::Docx => "docx",
    }
}

/// Reads the template at `path` for the output format `to`. A path that is
/// missing, is not a regular file, exceeds 16 MiB or has an extension other
/// than the format's is exit 3; a permission refusal is exit 2 (FR-040).
pub(super) fn load(path: &Path, to: Format) -> anyhow::Result<Vec<u8>> {
    let wanted = extension(to);
    if path.extension().and_then(|ext| ext.to_str()) != Some(wanted) {
        return Err(invalid(
            "--template",
            path,
            &format!("the extension must be .{wanted}"),
        ));
    }
    read_bounded("--template", path)
}

/// Reads the regular file at `path` for the option `flag`, at most 16 MiB. A
/// missing path, a non-regular file (a FIFO would block on open) or an
/// oversized file is exit 3; a permission refusal is exit 2.
pub(super) fn read_bounded(flag: &str, path: &Path) -> anyhow::Result<Vec<u8>> {
    let resolved = fs::canonicalize(path).map_err(|error| io_error(flag, path, error))?;
    // Checked before opening: opening a FIFO would block.
    let before = fs::metadata(&resolved).map_err(|error| io_error(flag, path, error))?;
    if !before.is_file() {
        return Err(invalid(flag, path, "not a regular file"));
    }
    let file = File::open(&resolved).map_err(|error| io_error(flag, path, error))?;
    let opened = file
        .metadata()
        .map_err(|error| io_error(flag, path, error))?;
    if !opened.is_file() {
        return Err(invalid(flag, path, "not a regular file"));
    }
    // One byte past the limit tells an oversized file from one at the limit.
    let mut bytes = Vec::new();
    file.take(MAX_INPUT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| io_error(flag, path, error))?;
    if bytes.len() as u64 > MAX_INPUT_FILE_BYTES {
        return Err(invalid(flag, path, "larger than 16 MiB"));
    }
    Ok(bytes)
}

pub(super) fn invalid(flag: &str, path: &Path, cause: &str) -> anyhow::Error {
    failure::invalid(anyhow::anyhow!("{flag} '{}': {cause}", path.display()))
}

/// A permission refusal is exit 2; every other path error is a file that
/// cannot be used, exit 3.
fn io_error(flag: &str, path: &Path, error: std::io::Error) -> anyhow::Error {
    let refused = error.kind() == std::io::ErrorKind::PermissionDenied;
    let error =
        anyhow::Error::from(error).context(format!("{flag} '{}' cannot be read", path.display()));
    if refused {
        failure::refused(error)
    } else {
        failure::invalid(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-036-AC-5")]
    fn the_extension_must_match_the_format() {
        let dir = tempfile::tempdir().expect("dir");
        let typ = dir.path().join("t.typ");
        fs::write(&typ, "x").expect("write");
        assert_eq!(load(&typ, Format::Pdf).expect("typ"), b"x");
        let error = load(&typ, Format::Docx).expect_err("mismatch");
        assert!(format!("{error:#}").contains("t.typ"));
    }

    #[test]
    #[trace("FR-036-AC-5")]
    fn a_directory_a_missing_path_and_an_oversized_file_are_refused() {
        let dir = tempfile::tempdir().expect("dir");
        let folder = dir.path().join("d.typ");
        fs::create_dir(&folder).expect("dir");
        assert!(load(&folder, Format::Pdf).is_err());
        assert!(load(&dir.path().join("absent.typ"), Format::Pdf).is_err());
        let big = dir.path().join("big.typ");
        fs::write(&big, vec![0u8; MAX_INPUT_FILE_BYTES as usize + 1]).expect("write");
        assert!(load(&big, Format::Pdf).is_err());
        let exact = dir.path().join("exact.typ");
        fs::write(&exact, vec![0u8; MAX_INPUT_FILE_BYTES as usize]).expect("write");
        assert!(load(&exact, Format::Pdf).is_ok());
    }

    #[test]
    #[trace("FR-040-AC-5")]
    fn an_unreadable_approval_file_is_exit_2() {
        let denied = io_error(
            "--approval",
            Path::new("v.json"),
            std::io::ErrorKind::PermissionDenied.into(),
        );
        assert_eq!(
            failure::outcome(&denied),
            ix_cli_kit::exit::Outcome::Refused
        );
        assert!(format!("{denied:#}").contains("--approval 'v.json'"));
    }

    #[test]
    #[trace("FR-040-AC-2")]
    fn a_permission_refusal_is_exit_2_and_any_other_path_error_exit_3() {
        let path = Path::new("t.typ");
        let denied = io_error(
            "--template",
            path,
            std::io::ErrorKind::PermissionDenied.into(),
        );
        assert_eq!(
            failure::outcome(&denied),
            ix_cli_kit::exit::Outcome::Refused
        );
        let other = io_error("--template", path, std::io::ErrorKind::NotADirectory.into());
        assert_eq!(failure::outcome(&other), ix_cli_kit::exit::Outcome::Invalid);
    }
}
