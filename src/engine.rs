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
pub const ENGINE_MANIFEST_VERSION: &str = env!("QUIRE_ENGINE_MANIFEST_VERSION");
pub const ENGINE_SOURCE_REVISION: &str = env!("QUIRE_ENGINE_SOURCE_REVISION");
pub const CLI_SOURCE_REVISION: &str = env!("QUIRE_CLI_SOURCE_REVISION");
pub const TOOL_PROVENANCE_SCHEMA: &str = "quire-tool-provenance-v1";

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
    // Structured subject/change-target/exclusive next move on findings (#364).
    "action_guidance.structured",
    // `build_assurance_export` / `read_assurance_export` and the closed
    // assurance-v1 schema (quire-rs FR-067/FR-068, quire-cli FR-020).
    "assurance_export.v1",
    // `CoverageReport.binding_census` — what the trace binder examined and what
    // bound, per language (quire-rs FR-050-AC-27, v0.43.0).
    "binding_census",
    // A self-named evidence-symbol subpopulation that comment/attribute
    // bindings cannot mask (quire-rs FR-050-AC-44, #367).
    "binding_census.self_named",
    // `BindingCensus.tagged` and `unmatched_example` — authored absence is
    // distinct from a tag the declared grammar missed (quire-rs #271).
    "binding_census.tagged",
    // Declaration diagnostics retain the exact authored path and line (#365).
    "declaration_origins",
    // `CoverageReport.metrics` — every headline ratio with its unit,
    // population, `examined` and `matched` (quire-rs FR-063, v0.44.0).
    "metrics_envelope",
    // `CoverageReport.minted_targets` — row identity and backed state behind
    // aggregate coverage totals (quire-rs FR-050-AC-38, #361).
    "minted_targets",
    // A specific property classification carries exact spans or a named safe refusal (#241).
    "property_spans.safe_refusal",
    // `TraceTarget.evidence` — reference registries resolve ids without
    // entering source-evidence totals (quire-rs FR-050-AC-40, #363).
    "reference_only_targets",
    // `CoverageReport.unmatched_tags` — row-addressable generic ids from the
    // engine-owned annotation scan (quire-rs FR-050-AC-39, #362).
    // `AcClassification::property.is_specific()` — the catch-all split out of
    // the extractable headline (quire-rs CR-095).
    "specific_shaped",
    // `CoverageReport.suspicions` — advisory shape findings (quire-rs FR-064).
    "suspicions",
    "unmatched_tags",
];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceState {
    Clean,
    Dirty,
    Unknown,
}

impl SourceState {
    fn from_build(value: &str) -> Self {
        match value {
            "clean" => Self::Clean,
            "dirty" => Self::Dirty,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentProvenance {
    pub version: &'static str,
    pub source_revision: Option<&'static str>,
    pub source_state: SourceState,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolProvenance {
    pub schema_version: &'static str,
    pub cli: ComponentProvenance,
    pub engine: ComponentProvenance,
    pub capabilities: Vec<&'static str>,
}

fn known_revision(value: &'static str) -> Option<&'static str> {
    (value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())).then_some(value)
}

impl ToolProvenance {
    pub fn current() -> Self {
        let mut capabilities = CAPABILITIES.to_vec();
        capabilities.sort_unstable();
        Self {
            schema_version: TOOL_PROVENANCE_SCHEMA,
            cli: ComponentProvenance {
                version: CLI_VERSION,
                source_revision: known_revision(CLI_SOURCE_REVISION),
                source_state: SourceState::from_build(env!("QUIRE_CLI_SOURCE_STATE")),
            },
            engine: ComponentProvenance {
                version: ENGINE_MANIFEST_VERSION,
                source_revision: known_revision(ENGINE_SOURCE_REVISION),
                source_state: if known_revision(ENGINE_SOURCE_REVISION).is_some() {
                    SourceState::Clean
                } else {
                    SourceState::Unknown
                },
            },
            capabilities,
        }
    }
}

/// The provenance block carried by every JSON payload.
///
/// Serialized under the `engine` key. Both published schemas
/// (`coverage-v1.schema.json`, `properties-v1.schema.json`) define it as an
/// **optional** object with all three members required — optional because an
/// in-process `CoverageReport::to_json` caller cannot know a CLI version. That
/// definition arrives with quire-rs CR-104 / FR-055-AC-8; a build pinned to an
/// engine predating it emits a payload its own pinned schema rejects, which
/// `tests/output_contract.rs` fails on rather than discovering in a consumer.
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

/// A payload with its provenance block appended.
///
/// **`#[serde(flatten)]`, not a `serde_json::Value` round-trip.** The first
/// draft of this took the payload as a `Value`, inserted the key, and
/// re-encoded — which silently rewrote **every key at every nesting level into
/// alphabetical order**, because `serde_json::Map` is a `BTreeMap` unless the
/// `preserve_order` feature is on, and it is not. Measured on `coverage --json`,
/// the top level went from
/// `[unbacked_rows, status_lies, untracked_symbols, groups, criteria, metrics,
/// totals]` to alphabetical, and so did every nested record. Output stayed
/// byte-identical *across runs* — so FR-050-AC-7 still passed and no test
/// noticed — while every checked-in payload in the ecosystem would have shown a
/// 100%-changed diff carrying no content change, and quire-cli FR-008-AC-4's
/// "field order SHALL match the public Rust struct declaration order" would
/// have been quietly false.
///
/// Flatten streams the inner value's fields in its own `Serialize` order and
/// appends `engine` after them, so the engine's output really is emitted
/// unmodified — which is what FR-008 behaviour rule 5 requires, and what the
/// round-trip only claimed.
#[derive(Serialize)]
pub struct WithProvenance<T: Serialize> {
    #[serde(flatten)]
    inner: T,
    engine: Provenance,
}

impl<T: Serialize> WithProvenance<T> {
    /// Wrap `inner` so it serializes with a trailing `engine` block.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            engine: Provenance::current(),
        }
    }
}

/// Wrap a payload so it serializes with a trailing `engine` block.
///
/// The one call every emitting surface makes. `serde` requires the inner value
/// to serialize as a map for `flatten` to work — every payload this is used on
/// is a struct or a JSON object, and a non-map inner value is a compile-time
/// shape error at the call site rather than a silent drop (the earlier
/// `Value`-based version returned a non-object payload untouched, so provenance
/// vanished with no signal).
pub fn attach<T: Serialize>(inner: T) -> WithProvenance<T> {
    WithProvenance::new(inner)
}

/// The `--version` line: both versions, distinctly.
///
/// One line, because `quire --version` is read by humans and scraped by
/// scripts, and a version string that grew a second line would break every
/// caller doing so. The engine is parenthesised and labelled, so neither
/// number can be mistaken for the other — which is the whole defect.
///
/// A `const`, not a function, because clap needs a `&'static str` for
/// `#[command(version = …)]`. The first draft had this as a `format!` helper
/// AND a separate `concat!` in `main.rs`: two independent assemblies of one
/// string, with the helper having no production caller and no test binding the
/// two. Mutating the `main.rs` copy to drop the engine number left the whole
/// suite green.
pub const VERSION_LINE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (cli ",
    env!("QUIRE_CLI_SOURCE_SHORT"),
    ", engine ",
    env!("QUIRE_ENGINE_MANIFEST_VERSION"),
    "@",
    env!("QUIRE_ENGINE_SOURCE_SHORT"),
    ")"
);

/// Compile-time witnesses for [`CAPABILITIES`].
///
/// Each token claims this build can emit something, and the doc on
/// `CAPABILITIES` says a build linking an engine without it does not compile.
/// Nothing enforced that: the tokens were four strings, and adding a fifth
/// naming a surface this crate never touches would have compiled, passed, and
/// shipped a payload advertising a capability with no engine call behind it.
///
/// Naming each type here is what makes the claim true. Deleting a capability's
/// production render is still possible — but removing the engine surface it
/// rests on now fails the build, which is the half a version comparison could
/// never give.
/// Each `const _` names the engine surface one token claims. The item is
/// evaluated at compile time, so an engine that dropped the field, the type or
/// the method fails the build here — naming the capability that went missing —
/// rather than shipping a payload that advertises it.
mod capability_witnesses {
    use quire_rs::{AcClassification, CoverageReport};

    // `action_guidance.structured` (#364)
    const _: fn(&quire_rs::coverage::CoverageDiagnostic) -> bool = |d| d.guidance.is_some();

    // `binding_census` (quire-rs FR-050-AC-27, v0.43.0)
    const _: fn(&CoverageReport) -> &[quire_rs::symbols::trace::BindingCensus] =
        |r| &r.binding_census;
    // `binding_census.self_named` (quire-rs FR-050-AC-44, #367)
    const _: fn(&CoverageReport) -> usize = |r| r.binding_census.iter().map(|c| c.self_named).sum();
    // `binding_census.tagged` (#271)
    const _: fn(&CoverageReport) -> usize = |r| r.binding_census.iter().map(|c| c.tagged).sum();
    // `metrics_envelope` (FR-063, v0.44.0)
    const _: fn(&CoverageReport) -> &[quire_rs::metric::Metric] = |r| &r.metrics;
    // `minted_targets` (FR-050-AC-38, #361)
    const _: fn(&CoverageReport) -> usize = |r| r.minted_targets.len();
    // `declaration_origins` (#365)
    const _: fn(&quire_rs::Registry) -> bool = |r| r.traceability_origin().is_some();
    // `property_spans.safe_refusal` (#241)
    const _: fn(&AcClassification) -> bool =
        |c| c.domain.is_some() || c.signals.iter().any(|s| s.starts_with("span:refused-"));
    // `reference_only_targets` (FR-050-AC-40, #363)
    const _: fn(
        &quire_rs::traceability::TraceTarget,
    ) -> quire_rs::traceability::TraceTargetEvidence = |target| target.evidence;
    // `unmatched_tags` (FR-050-AC-39, #362)
    const _: fn(&CoverageReport) -> &[quire_rs::symbols::trace::UnmatchedTag] =
        |r| &r.unmatched_tags;
    // `suspicions` (FR-064)
    const _: fn(&CoverageReport) -> usize = |r| r.suspicions.len();
    // `specific_shaped` (CR-095)
    const _: fn(&AcClassification) -> bool = |c| c.property.is_specific();
}

#[cfg(test)]
mod tests {
    use super::*;

    // The lockfile pins the engine by tag, and the tag is not the engine
    // crate's manifest version — `quire-rs`'s own Cargo.toml says 0.33.0 while
    // it ships v0.45.0. If this ever reports the manifest number, build.rs has
    // silently started reading the wrong field and every payload is lying
    // again.
    // The version this build reports must be the one its own lockfile
    // resolves. Asserted against the lockfile rather than against
    // `!= CLI_VERSION`: the review pointed out that comparison encodes a
    // coincidence, and would start failing spuriously the day the CLI reaches
    // 0.45.0 with nothing wrong.
    #[test]
    fn the_reported_engine_version_is_the_one_the_lockfile_resolves() {
        assert_ne!(ENGINE_VERSION, "", "the engine version must be resolved");
        assert_ne!(
            ENGINE_VERSION, "unknown",
            "the lockfile must be readable at build time",
        );

        let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
            .expect("Cargo.lock");
        assert_eq!(
            Some(ENGINE_VERSION.to_string()),
            crate::lockfile::engine_version(&lock),
            "the compiled-in engine version disagrees with this crate's lockfile",
        );
    }

    #[test]
    fn the_version_line_names_both_distinctly() {
        // Checked first: `str::contains("")` is unconditionally true, so an
        // empty ENGINE_VERSION would make the assertion below decorative —
        // exactly the vacuity this whole module exists to prevent.
        assert!(!ENGINE_VERSION.is_empty());
        assert!(VERSION_LINE.contains(CLI_VERSION), "{VERSION_LINE}");
        assert!(
            VERSION_LINE.contains(ENGINE_MANIFEST_VERSION),
            "{VERSION_LINE}"
        );
        assert!(
            VERSION_LINE.contains(env!("QUIRE_ENGINE_SOURCE_SHORT")),
            "{VERSION_LINE}"
        );
        assert!(
            VERSION_LINE.contains("engine"),
            "the engine number must be labelled, or the two are indistinguishable: {VERSION_LINE}",
        );
    }

    #[test]
    fn attach_appends_the_block_in_order_and_adds_nothing_else() {
        #[derive(serde::Serialize)]
        struct Payload {
            // Deliberately NOT alphabetical: the defect this replaced sorted
            // every key, and a fixture in sorted order could not have caught it.
            zebra: u8,
            alpha: u8,
        }

        let rendered = serde_json::to_string(&super::attach(Payload { zebra: 1, alpha: 2 }))
            .expect("serializes");
        assert_eq!(
            rendered,
            format!(
                r#"{{"zebra":1,"alpha":2,"engine":{{"cli":"{CLI_VERSION}","engine":"{ENGINE_VERSION}","capabilities":{}}}}}"#,
                serde_json::to_string(CAPABILITIES).expect("capabilities serialize"),
            ),
            "provenance must be APPENDED, leaving the inner order untouched",
        );

        // ...and nothing else appears. Asserted on the parsed key set rather
        // than by naming two survivors: the earlier version checked that two
        // keys were still there, which a payload that had also grown a
        // `timestamp` would have passed.
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("valid JSON");
        let keys: Vec<&String> = parsed.as_object().expect("object").keys().collect();
        assert_eq!(
            keys,
            ["alpha", "engine", "zebra"].iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_provenance_block_carries_the_capability_the_ticket_is_about() {
        let engine = serde_json::to_value(Provenance::current()).expect("serializes");
        assert_eq!(engine["cli"], CLI_VERSION);
        assert_eq!(engine["engine"], ENGINE_VERSION);
        assert!(
            engine["capabilities"]
                .as_array()
                .expect("capabilities array")
                .iter()
                .any(|t| t == "binding_census"),
            "{engine}",
        );
        assert!(
            engine["capabilities"]
                .as_array()
                .expect("capabilities array")
                .iter()
                .any(|t| t == "binding_census.self_named"),
            "{engine}",
        );
        assert!(
            engine["capabilities"]
                .as_array()
                .expect("capabilities array")
                .iter()
                .any(|t| t == "binding_census.tagged"),
            "{engine}",
        );
        assert!(
            engine["capabilities"]
                .as_array()
                .expect("capabilities array")
                .iter()
                .any(|t| t == "minted_targets"),
            "{engine}",
        );
        assert!(
            engine["capabilities"]
                .as_array()
                .expect("capabilities array")
                .iter()
                .any(|t| t == "reference_only_targets"),
            "{engine}",
        );
        assert!(
            engine["capabilities"]
                .as_array()
                .expect("capabilities array")
                .iter()
                .any(|t| t == "unmatched_tags"),
            "{engine}",
        );
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
        assert_eq!(
            CAPABILITIES, sorted,
            "capabilities must be published sorted"
        );
    }

    #[test]
    fn machine_provenance_is_versioned_complete_and_deterministic() {
        let first = serde_json::to_string(&ToolProvenance::current()).expect("serializes");
        let second = serde_json::to_string(&ToolProvenance::current()).expect("serializes");
        assert_eq!(first, second);
        let parsed: serde_json::Value = serde_json::from_str(&first).expect("valid JSON");
        assert_eq!(parsed["schemaVersion"], TOOL_PROVENANCE_SCHEMA);
        assert_eq!(parsed["cli"]["version"], CLI_VERSION);
        assert_eq!(parsed["engine"]["version"], ENGINE_MANIFEST_VERSION);
        assert_eq!(parsed["engine"]["sourceRevision"], ENGINE_SOURCE_REVISION);
        assert_eq!(parsed["engine"]["sourceState"], "clean");
    }
}
