use std::fmt;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::source::Finding;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct FindingList(Vec<Finding>);

impl From<Vec<Finding>> for FindingList {
    fn from(findings: Vec<Finding>) -> Self {
        Self(findings)
    }
}

impl fmt::Display for FindingList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, finding) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            write!(
                formatter,
                "{}:{}: {}",
                finding.path.display(),
                finding.line,
                finding.reason
            )?;
        }
        Ok(())
    }
}

/// Typed refusal surface for every repository-qualification boundary.
#[derive(Debug, Error)]
pub enum Error {
    #[error("read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("read source directory {path}: {source}")]
    ReadDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("read directory entry in {path}: {source}")]
    ReadDirectoryEntry {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("inspect source path {path}: {source}")]
    InspectPath {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("refuse symlinked Rust source path {path}")]
    SymlinkSource { path: PathBuf },
    #[error("refuse symlinked unsafe-comment baseline {path}")]
    SymlinkBaseline { path: PathBuf },
    #[error("parse Rust source {path}: {source}")]
    ParseRust {
        path: PathBuf,
        #[source]
        source: syn::Error,
    },
    #[error("parse typed coverage JSON: {source}")]
    CoverageJson {
        #[source]
        source: serde_path_to_error::Error<serde_json::Error>,
    },
    #[error("parse typed hyperfine JSON: {source}")]
    HyperfineJson {
        #[source]
        source: serde_path_to_error::Error<serde_json::Error>,
    },
    #[error("parse in-memory Rust source: {source}")]
    ParseRustInput {
        #[source]
        source: syn::Error,
    },
    #[error("required targets were not minted: {ids:?}")]
    MissingTargets { ids: Vec<String> },
    #[error("required targets are not backed: {ids:?}")]
    UnbackedTargets { ids: Vec<String> },
    #[error("duplicate minted target ids: {ids:?}")]
    DuplicateTargets { ids: Vec<String> },
    #[error("user-story examples were incorrectly promoted to binding acceptance: {ids:?}")]
    PromotedExamples { ids: Vec<String> },
    #[error("issue #74 has unmatched trace tags: {ids:?}")]
    UnmatchedTags { ids: Vec<String> },
    #[error("coverage reports {count} non-empty status_lies record(s)")]
    StatusLies { count: usize },
    #[error("coverage reports {count} non-empty untracked_symbols record(s)")]
    UntrackedSymbols { count: usize },
    #[error("threshold_ms must be finite and non-negative, found {value}")]
    InvalidThreshold { value: f64 },
    #[error("hyperfine results array is empty")]
    EmptyResults,
    #[error("first hyperfine result has no samples")]
    EmptySamples,
    #[error("hyperfine sample must be finite and non-negative, found {value}")]
    InvalidSample { value: f64 },
    #[error("BENCH-001 failed: p95 {p95_ms:.2} ms > {threshold_ms:.0} ms threshold")]
    BenchmarkExceeded { p95_ms: f64, threshold_ms: f64 },
    #[error("thin-boundary audit failed:\n{findings}")]
    ThinBoundary { findings: FindingList },
    #[error("unsafe-comment audit failed: missing SAFETY comment at {unreviewed:?}; stale unsafe-comment baseline entries: {stale:?}")]
    UnsafeBaseline {
        unreviewed: Vec<String>,
        stale: Vec<String>,
    },
}
