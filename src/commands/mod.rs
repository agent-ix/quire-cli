//! Subcommand implementations. Each module owns one `quire <verb>` subcommand.
//! The quire-rs-backed verbs stay thin wrappers over the engine; `update` is a
//! thin wrapper over the package-agnostic `self_update` engine instead.

pub mod clauses;
pub mod coverage;
pub mod edit;
pub mod extract;
pub mod fix;
pub mod lint;
pub mod lookup;
pub mod parse;
pub mod properties;
pub mod provenance;
pub mod schema;
pub mod symbols;
pub mod update;
pub mod validate;

use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use quire_cli::io::{emit_quire_diagnostics, Diagnostics};
use quire_rs::Registry;

/// Cross-command context plumbed in from global flags.
pub struct Ctx {
    pub diagnostics: Diagnostics,
    pub pretty: bool,
}

/// The name of the document root under a scope. One constant, so the
/// derivation below and the code walk's exclusion cannot drift apart
/// (agent-ix/quire-rs#113).
pub const DOCUMENT_ROOT_DIR: &str = "spec";

/// Why a scope has no usable document root — a typed variant rather than a
/// formatted string, so `--diagnostics json` carries a stable `kind` and a
/// test can assert the interpolated path instead of `contains("spec")`
/// (agent-ix/quire-rs#113).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentRootError {
    /// `<scope>/spec` does not exist, or is not a directory.
    Missing { path: PathBuf },
}

impl DocumentRootError {
    /// Stable machine token for `--diagnostics json`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Missing { .. } => "MissingDocumentRoot",
        }
    }
}

impl std::fmt::Display for DocumentRootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { path } => write!(
                f,
                "no document root at '{}': spec documents live in `{DOCUMENT_ROOT_DIR}/` \
                 under --scope (quire-rs FR-050 two-roots); point --scope at the \
                 repository root, or create {DOCUMENT_ROOT_DIR}/",
                path.display()
            ),
        }
    }
}

impl std::error::Error for DocumentRootError {}

/// Derive the document root from a scope (quire-rs FR-050 two-roots,
/// CR-045): documents live in `<scope>/spec`, never at the repository root.
/// A missing document root is a **named error** — silently falling back to
/// walking the scope is how the repository-wide crawl survived unnoticed.
/// `spec/` is convention, not configuration: no manifest key, no flag.
///
/// The path is **canonicalized** when it resolves. `is_dir()` follows
/// symlinks while the corpus walker sets `follow_links(false)`, so a
/// symlinked `spec/` passed this check and then produced zero documents,
/// `total: 0` and exit 0 — no error anywhere. Canonicalizing here also makes
/// `coverage` and `validate --okf` agree on the root: `validate` already
/// canonicalized via `safety::validate_dir_path` while `coverage` did not,
/// so the two commands resolved *different* document roots for the same
/// repository (agent-ix/quire-rs#113).
pub fn spec_root_of(scope: &Path) -> anyhow::Result<PathBuf> {
    let root = scope.join(DOCUMENT_ROOT_DIR);
    if !root.is_dir() {
        return Err(DocumentRootError::Missing { path: root }.into());
    }
    // A resolvable directory that cannot be canonicalized is left as-is
    // rather than failing: the walk can still read it, and inventing a
    // failure here would be worse than the drift canonicalizing prevents.
    Ok(root.canonicalize().unwrap_or(root))
}

/// Load a single module registry for a `--module <PATH>` argument and
/// surface load problems eagerly (FR-004 CR note / upstream
/// FR-013-AC-13).
///
/// The tolerant engine load reports a missing `manifest.yaml` (or an
/// unloadable manifest) as an `ArchetypeLoadFailure` while returning an
/// EMPTY registry; commands that ignored `failures()` then died later
/// with a misleading `UnknownArchetype`. When the load produced zero
/// modules and at least one failure, fail fast with the real reason.
pub fn load_module_registry(ctx: &Ctx, module: &Path) -> anyhow::Result<Registry> {
    let registry = Registry::load_module(module).context("loading module registry")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    if registry.module_names().count() == 0 {
        if let Some(f) = registry.failures().first() {
            bail!("module load failed: {} ({})", f.reason, f.path.display());
        }
    }
    Ok(registry)
}
