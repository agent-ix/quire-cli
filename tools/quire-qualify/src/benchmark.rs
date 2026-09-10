//! Typed hyperfine-result qualification.

use std::path::Path;

use serde::Deserialize;

use crate::{read, Error, Result};

#[derive(Debug, Deserialize)]
struct HyperfineReport {
    results: Vec<HyperfineResult>,
}

#[derive(Debug, Deserialize)]
struct HyperfineResult {
    times: Vec<f64>,
}

/// A computed benchmark observation. A measurement is distinct from its
/// threshold verdict so the CLI can retain the legacy observation-on-failure.
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub p95_ms: f64,
    pub threshold_ms: f64,
    pub sample_count: usize,
}

impl Observation {
    #[must_use]
    pub fn passes(&self) -> bool {
        self.p95_ms <= self.threshold_ms
    }

    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "BENCH-001: p95={:.2} ms (threshold {:.0} ms)",
            self.p95_ms, self.threshold_ms
        )
    }
}

/// Parse hyperfine JSON and compute its first result's nearest-rank p95.
pub fn evaluate_bytes(bytes: &[u8], threshold_ms: f64) -> Result<Observation> {
    if !threshold_ms.is_finite() || threshold_ms < 0.0 {
        return Err(Error::InvalidThreshold {
            value: threshold_ms,
        });
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let report: HyperfineReport = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|source| Error::HyperfineJson { source })?;
    let first = report.results.first().ok_or(Error::EmptyResults)?;
    if first.times.is_empty() {
        return Err(Error::EmptySamples);
    }
    if let Some(value) = first
        .times
        .iter()
        .find(|value| !value.is_finite() || **value < 0.0)
    {
        return Err(Error::InvalidSample { value: *value });
    }

    let mut times = first.times.clone();
    times.sort_by(f64::total_cmp);
    let rank = (95 * times.len()).div_ceil(100);
    let p95_ms = times.get(rank - 1).copied().ok_or(Error::EmptySamples)? * 1000.0;
    Ok(Observation {
        p95_ms,
        threshold_ms,
        sample_count: times.len(),
    })
}

/// Read a hyperfine report and compute its p95 observation.
pub fn evaluate_file(path: &Path, threshold_ms: f64) -> Result<Observation> {
    evaluate_bytes(&read(path)?, threshold_ms)
}
