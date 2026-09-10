//! Typed assurance-coverage qualification.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::{read, Error, Result};

#[derive(Debug, Deserialize)]
struct MintedTarget {
    id: String,
    backed: bool,
    #[serde(flatten)]
    _extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct UnmatchedTag {
    trace_id: String,
    #[serde(flatten)]
    _extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct CoverageReport {
    minted_targets: Vec<MintedTarget>,
    unmatched_tags: Vec<UnmatchedTag>,
    status_lies: Vec<Value>,
    untracked_symbols: Vec<Value>,
    #[serde(flatten)]
    _extensions: BTreeMap<String, Value>,
}

/// The closed issue-74 target population governed by this repository.
pub fn required_targets() -> BTreeSet<String> {
    let mut required = BTreeSet::new();
    required.extend((1..=9).map(|index| format!("FR-020-AC-{index}")));
    required.extend((136..=145).map(|index| format!("IT-{index}")));
    required.insert("TC-814".to_owned());
    required.insert("StR-004-VC-2".to_owned());
    required.insert("StR-004-VC-3".to_owned());
    required
}

/// Qualify a coverage report and return the stable success observation.
pub fn check_bytes(bytes: &[u8]) -> Result<String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let report: CoverageReport = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|source| Error::CoverageJson { source })?;
    let required = required_targets();
    let mut minted = BTreeMap::new();
    let mut duplicates = BTreeSet::new();
    for target in &report.minted_targets {
        if minted.insert(target.id.as_str(), target).is_some() {
            duplicates.insert(target.id.clone());
        }
    }
    if !duplicates.is_empty() {
        return Err(Error::DuplicateTargets {
            ids: duplicates.into_iter().collect(),
        });
    }

    let missing: Vec<_> = required
        .iter()
        .filter(|target| !minted.contains_key(target.as_str()))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(Error::MissingTargets { ids: missing });
    }

    let unbacked: Vec<_> = required
        .iter()
        .filter(|target| {
            minted
                .get(target.as_str())
                .is_none_or(|record| !record.backed)
        })
        .cloned()
        .collect();
    if !unbacked.is_empty() {
        return Err(Error::UnbackedTargets { ids: unbacked });
    }

    let invalid_examples: Vec<_> = minted
        .keys()
        .filter(|target| target.starts_with("US-006-AC-"))
        .copied()
        .collect();
    if !invalid_examples.is_empty() {
        return Err(Error::PromotedExamples {
            ids: invalid_examples.into_iter().map(str::to_owned).collect(),
        });
    }

    let unmatched: Vec<_> = report
        .unmatched_tags
        .iter()
        .map(|entry| entry.trace_id.as_str())
        .filter(|trace_id| trace_id.starts_with("FR-020-AC-") || trace_id.starts_with("US-006-AC-"))
        .map(str::to_owned)
        .collect();
    if !unmatched.is_empty() {
        return Err(Error::UnmatchedTags { ids: unmatched });
    }
    if !report.status_lies.is_empty() {
        return Err(Error::StatusLies {
            count: report.status_lies.len(),
        });
    }
    if !report.untracked_symbols.is_empty() {
        return Err(Error::UntrackedSymbols {
            count: report.untracked_symbols.len(),
        });
    }

    Ok(format!(
        "assurance traceability ok: {0}/{0} required targets backed",
        required.len()
    ))
}

/// Read and qualify a coverage report.
pub fn check_file(path: &Path) -> Result<String> {
    check_bytes(&read(path)?)
}
