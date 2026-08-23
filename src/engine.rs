//! Which instrument produced a payload (agent-ix/quire-cli#68).
//!
//! `quire --version` reported the **CLI crate** version. The engine is a git
//! dependency pinned by tag, and no surface reported it at all — so a current
//! CLI could link a stale engine and still print a confident number.
//!
//! Measured, and the reason this module exists: the installed CLI **0.29.0**
//! pins engine **v0.42.0**, while `binding_census` — the only signal answering
//! "did the binder read a single test?" — landed in **v0.43.0**. Four
//! battle-testing passes reported ecosystem figures from a binary that could
//! not emit it. Upgrading the binary fixes that instance; this fixes the class,
//! because the provenance now travels *with the payload* and survives being
//! saved to disk.
//!
//! This is **not** the contract version. Which schema describes a payload lives
//! in that schema's `$id` (quire-rs FR-055-CON-2, as narrowed by CR-104); a
//! payload asserting its own conformance is a different and worse idea. This
//! says which build did the measuring.

use serde::Serialize;

/// The CLI crate version — this crate's own.
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The `quire-rs` version actually linked, resolved from the lockfile by
/// `build.rs` rather than from a constant in either tree.
///
/// `unknown` when the lockfile could not be read. Deliberately not a fallback
/// to [`CLI_VERSION`]: a plausible-looking substitute is the failure this
/// module exists to end.
pub const ENGINE_VERSION: &str = env!("QUIRE_ENGINE_VERSION");

/// What this build can emit, as tokens.
///
/// **Compile-checked by construction.** Every token names an engine surface
/// this binary calls; a build linking an engine without one does not compile,
/// so the list cannot claim a capability the linked engine lacks. That is the
/// property a hand-maintained list would not have — and it is why the tokens
/// are asserted rather than derived from a version comparison.
///
/// **A token, never version arithmetic.** A consumer asserts it needs
/// `binding_census`; it must not assert `engine >= 0.43.0`. A version
/// comparison in a consumer is a second place the contract lives, and it goes
/// stale in a repository nobody thinks to update.
///
/// The vocabulary is **open**: adding a token here must not break a consumer
/// written against an older list, which is why the published schemas do not
/// enumerate it.
pub const CAPABILITIES: &[&str] = &[
    // `CoverageReport.binding_census` — what the trace binder examined and what
    // bound, per language (quire-rs FR-050-AC-27, v0.43.0).
    "binding_census",
    // `CoverageReport.metrics` — every headline ratio with its unit,
    // population, `examined` and `matched` (quire-rs FR-063, v0.44.0).
    "metrics_envelope",
    // `CoverageReport.suspicions` — advisory shape findings (quire-rs FR-064).
    "suspicions",
    // `AcClassification::property.is_specific()` — the catch-all split out of
    // the extractable headline (quire-rs CR-095).
    "specific_shaped",
];

/// The provenance block carried by every JSON payload.
///
/// Serialized under the `engine` key. Both published schemas
/// (`coverage-v1.schema.json`, `properties-v1.schema.json`) define it as an
/// **optional** object with all three members required — optional because an
/// in-process `CoverageReport::to_json` caller cannot know a CLI version.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Provenance {
    /// The `quire-cli` crate version.
    pub cli: &'static str,
    /// The linked `quire-rs` version, verbatim — a `-<n>-g<sha>` describe
    /// suffix is never rounded to the nearest tag.
    pub engine: &'static str,
    /// Capability tokens; see [`CAPABILITIES`].
    pub capabilities: &'static [&'static str],
}

impl Provenance {
    pub const fn current() -> Self {
        Self {
            cli: CLI_VERSION,
            engine: ENGINE_VERSION,
            capabilities: CAPABILITIES,
        }
    }
}

impl Default for Provenance {
    fn default() -> Self {
        Self::current()
    }
}

/// Attach the provenance block to an already-serialized payload.
///
/// Takes the payload as a [`serde_json::Value`] rather than wrapping it in a
/// generic envelope struct so the inner shape is emitted **unmodified** — the
/// FR-008 rule that the CLI adds structure around engine output and never
/// rewrites it. A non-object payload (an array, a bare string) is returned
/// untouched: there is nowhere to put the key, and inventing a wrapper would
/// change a shape consumers pin.
pub fn attach(mut payload: serde_json::Value) -> serde_json::Value {
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "engine".to_string(),
            serde_json::to_value(Provenance::current()).expect("provenance serializes"),
        );
    }
    payload
}

/// The `--version` line: both versions, distinctly.
///
/// One line, because `quire --version` is read by humans and scraped by
/// scripts, and a version string that grew a second line would break every
/// caller doing so. The engine is parenthesised and labelled, so neither
/// number can be mistaken for the other — which is the whole defect.
pub fn version_line() -> String {
    format!("{CLI_VERSION} (engine {ENGINE_VERSION})")
}

#[cfg(test)]
mod tests {
    use super::*;

    // The lockfile pins the engine by tag, and the tag is not the engine
    // crate's manifest version — `quire-rs`'s own Cargo.toml says 0.33.0 while
    // it ships v0.45.0. If this ever reports the manifest number, build.rs has
    // silently started reading the wrong field and every payload is lying
    // again.
    #[test]
    fn the_engine_version_is_resolved_and_is_not_the_cli_version() {
        assert_ne!(ENGINE_VERSION, "", "the engine version must be resolved");
        assert_ne!(
            ENGINE_VERSION, "unknown",
            "the lockfile must be readable at build time",
        );
        assert_ne!(
            ENGINE_VERSION, CLI_VERSION,
            "reporting the CLI version as the engine version is the defect, not the fix",
        );
    }

    #[test]
    fn the_version_line_names_both_distinctly() {
        let line = version_line();
        assert!(line.contains(CLI_VERSION), "{line}");
        assert!(line.contains(ENGINE_VERSION), "{line}");
        assert!(
            line.contains("engine"),
            "the engine number must be labelled, or the two are indistinguishable: {line}",
        );
    }

    #[test]
    fn attach_adds_the_block_and_changes_nothing_else() {
        let before = serde_json::json!({"documents": [], "totals": {"backed": 1}});
        let after = super::attach(before.clone());

        let object = after.as_object().expect("object");
        assert_eq!(object["documents"], before["documents"]);
        assert_eq!(object["totals"], before["totals"]);

        let engine = &object["engine"];
        assert_eq!(engine["cli"], CLI_VERSION);
        assert_eq!(engine["engine"], ENGINE_VERSION);
        assert!(
            engine["capabilities"]
                .as_array()
                .expect("capabilities array")
                .iter()
                .any(|t| t == "binding_census"),
            "the token the whole ticket is about must be present: {engine}",
        );
    }

    #[test]
    fn a_non_object_payload_is_returned_untouched() {
        // Nowhere to put the key, and wrapping would change a pinned shape.
        let array = serde_json::json!([1, 2, 3]);
        assert_eq!(super::attach(array.clone()), array);
    }

    #[test]
    fn capability_tokens_are_unique_and_non_empty() {
        let mut sorted = CAPABILITIES.to_vec();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(before, sorted.len(), "duplicate capability token");
        assert!(!CAPABILITIES.is_empty());
        assert!(CAPABILITIES.iter().all(|t| !t.is_empty()));
    }
}
