//! The organisation's own template (FR-036): a file the user holds locally,
//! read once into memory and written only into the capability root.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use super::Format;
use crate::commands::failure;

/// The largest template accepted.
const MAX_TEMPLATE_BYTES: u64 = 16 * 1024 * 1024;

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
        return Err(invalid(path, &format!("the extension must be .{wanted}")));
    }
    let resolved = fs::canonicalize(path).map_err(|error| io_error(path, error))?;
    let file = File::open(&resolved).map_err(|error| io_error(path, error))?;
    let metadata = file.metadata().map_err(|error| io_error(path, error))?;
    if !metadata.is_file() {
        return Err(invalid(path, "not a regular file"));
    }
    // One byte past the limit tells an oversized file from one at the limit.
    let mut bytes = Vec::new();
    file.take(MAX_TEMPLATE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| io_error(path, error))?;
    if bytes.len() as u64 > MAX_TEMPLATE_BYTES {
        return Err(invalid(path, "larger than 16 MiB"));
    }
    Ok(bytes)
}

fn invalid(path: &Path, cause: &str) -> anyhow::Error {
    failure::invalid(anyhow::anyhow!("--template '{}': {cause}", path.display()))
}

fn io_error(path: &Path, error: std::io::Error) -> anyhow::Error {
    failure::input(
        anyhow::Error::from(error)
            .context(format!("--template '{}' cannot be read", path.display())),
    )
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
        fs::write(&big, vec![0u8; MAX_TEMPLATE_BYTES as usize + 1]).expect("write");
        assert!(load(&big, Format::Pdf).is_err());
        let exact = dir.path().join("exact.typ");
        fs::write(&exact, vec![0u8; MAX_TEMPLATE_BYTES as usize]).expect("write");
        assert!(load(&exact, Format::Pdf).is_ok());
    }
}
