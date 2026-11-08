//! Release tooling for `quire-cli`.
//!
//! npm packaging is owned by the shared `agent-ix/nodejs-actions`
//! `publish-native-npm` action (see ADR-0003). This crate keeps only the
//! pre-publish gate that every built executable reports the release version.
//! It deliberately has no dependency on `quire-rs`.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use semver::Version;

fn checked_version(raw: &str) -> Result<Version> {
    Version::parse(raw).with_context(|| format!("invalid SemVer release version {raw:?}"))
}

/// Execute one native artifact's version surface and compare it exactly.
pub fn verify_binary_version(version: &str, binary: &Path) -> Result<()> {
    let expected = checked_version(version)?.to_string();
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .with_context(|| format!("execute {} --version", binary.display()))?;
    if !output.status.success() {
        bail!(
            "{} --version failed with {}: {}",
            binary.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = std::str::from_utf8(&output.stdout)
        .with_context(|| format!("{} --version stdout is not UTF-8", binary.display()))?;
    let observed = stdout
        .split_whitespace()
        .nth(1)
        .with_context(|| format!("{} --version has no version field", binary.display()))?;
    if observed != expected {
        bail!(
            "binary {} reports version {observed}, expected {expected}",
            binary.display()
        );
    }
    Ok(())
}
