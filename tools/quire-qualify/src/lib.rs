//! Native, repository-local qualification for `quire-cli`.
//!
//! This crate owns qualification inputs and verdicts only. It has no dependency
//! on `quire-rs` and does not implement product behavior.

pub mod benchmark;
pub mod coverage;
mod error;
pub mod source;
pub mod unsafe_comments;
mod walk;

use std::fs;
use std::path::Path;

pub use error::{Error, FindingList, Result};

/// Read a governed input with its path retained in any diagnostic.
pub(crate) fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })
}
