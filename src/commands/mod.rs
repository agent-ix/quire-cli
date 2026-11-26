//! Subcommand implementations. Each module owns one `quire <verb>` subcommand.
//! The quire-rs-backed verbs stay thin wrappers over the engine; `update` is a
//! thin wrapper over ix-cli-kit's `self_update` engine instead. `check_lock`
//! owns the explicitly scoped developer policy (FR-027).

macro_rules! primary_line {
    ($($arg:tt)*) => {
        ix_cli_kit::streams::write_result(&mut std::io::stdout().lock(), &format!($($arg)*))?
    };
}
macro_rules! primary {
    ($($arg:tt)*) => {
        ix_cli_kit::streams::write_primary_stdout(format!($($arg)*).as_bytes())?
    };
}
macro_rules! invalid_error {
    ($($arg:tt)*) => {
        crate::commands::failure::invalid(anyhow::anyhow!($($arg)*))
    };
}
macro_rules! invalid_request {
    ($($arg:tt)*) => {
        return Err(crate::commands::failure::invalid(anyhow::anyhow!($($arg)*)))
    };
}
macro_rules! partial_report {
    ($($arg:tt)*) => {
        return Err(crate::commands::failure::classified(ix_cli_kit::exit::Outcome::Partial, anyhow::anyhow!($($arg)*)))
    };
}
macro_rules! policy_refusal {
    ($($arg:tt)*) => {
        return Err(crate::commands::failure::refused(anyhow::anyhow!($($arg)*)))
    };
}

pub(crate) mod failure;

pub mod assurance;
pub mod check_lock;
pub mod clauses;
pub mod corpus_input;
pub mod coverage;
pub mod edit;
pub mod export;
pub mod extract;
pub mod fix;
pub mod impact;
pub mod lint;
pub mod lookup;
pub mod matrix;
mod matrix_design;
pub mod parse;
pub mod properties;
pub mod provenance;
pub mod publish_check;
pub mod relations;
pub mod schema;
pub mod selftest;
pub mod symbols;
pub mod trace;
pub mod update;
pub mod validate;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use ix_cli_kit::streams::Diagnostics;
use quire_cli::io::emit_quire_diagnostics;
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
                 under the repository root (quire-rs FR-050 two-roots); name the \
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

/// Print each extraction diagnostic on the diagnostics stream, a warning for
/// warning severity and an error for error severity.
pub fn emit_extraction_diagnostics(ctx: &Ctx, diagnostics: &[quire_rs::symbols::SymbolDiagnostic]) {
    use quire_rs::symbols::DiagnosticSeverity;
    for d in diagnostics {
        let message = format!("{}: {}", d.path, d.reason);
        match d.severity {
            DiagnosticSeverity::Warning => {
                ix_cli_kit::streams::Record::warning("SymbolExtraction", &message)
            }
            DiagnosticSeverity::Error => {
                ix_cli_kit::streams::Record::error("SymbolExtraction", &message)
            }
        }
        .emit(ctx.diagnostics);
    }
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
/// Load a closed registry for repeatable `--module <PATH>` arguments
/// (upstream FR-013, agent-ix/quire-rs#405).
///
/// The explicit roots replace ambient discovery: neither the module-path
/// environment variables nor the default install root is consulted.
///
/// Load problems surface eagerly for the same reason as
/// [`load_module_registry`]: the tolerant engine load reports an unloadable
/// manifest as an `ArchetypeLoadFailure` while returning an EMPTY registry,
/// and a caller that ignored `failures()` then died later with a misleading
/// `UnknownArchetype`.
pub fn load_module_set_registry(ctx: &Ctx, modules: &[String]) -> anyhow::Result<Registry> {
    let roots = validate_module_roots(modules)?;
    load_module_set_roots(ctx, &roots)
}

/// Load explicit relation modules and only the installed semantic providers
/// their imports declare, preserving the explicit roots first.
pub fn load_relation_module_set_registry(
    ctx: &Ctx,
    modules: &[String],
) -> anyhow::Result<Registry> {
    let mut roots = validate_module_roots(modules)?;
    add_semantic_import_providers(&mut roots);
    load_module_set_roots(ctx, &roots)
}

fn validate_module_roots(modules: &[String]) -> anyhow::Result<Vec<PathBuf>> {
    modules
        .iter()
        .map(|raw| {
            quire_cli::safety::validate_module_path(raw)
                .with_context(|| format!("validating --module '{raw}'"))
        })
        .collect()
}

fn load_module_set_roots(ctx: &Ctx, roots: &[PathBuf]) -> anyhow::Result<Registry> {
    let refs: Vec<&Path> = roots.iter().map(|p| p.as_path()).collect();
    let registry = Registry::load_module_set(&refs)
        .map_err(failure::invalid)
        .context("loading module set")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    if registry.module_names().count() == 0 {
        if let Some(f) = registry.failures().first() {
            invalid_request!("module load failed: {} ({})", f.reason, f.path.display());
        }
    }
    Ok(registry)
}

#[derive(Debug)]
struct ModuleSemanticMetadata {
    root: PathBuf,
    package: String,
    version: String,
    imports: BTreeMap<String, String>,
}

/// Resolve the transitive `semantic.imports` closure from installed modules.
/// The engine must see explicit roots and providers in one load so imported
/// vocabularies participate in its ordinary merge and diagnostic behavior.
fn add_semantic_import_providers(roots: &mut Vec<PathBuf>) {
    let mut available = BTreeSet::new();
    let mut pending = BTreeSet::new();
    for root in roots.iter() {
        if let Some(module) = semantic_metadata(root) {
            available.insert((module.package, module.version));
            pending.extend(module.imports);
        }
    }
    if pending.is_empty() {
        return;
    }

    let mut candidates = Vec::new();
    let module_path_env = quire_rs::loader::paths::module_path_env(
        std::env::var_os("IX_FILAMENT_MODULES_PATH"),
        std::env::var_os("IX_SCHEMA_PATH"),
    );
    for search in quire_rs::loader::paths::resolve_search_paths(&[], module_path_env) {
        let quire_rs::loader::paths::PathDiagnostic::Ok(search_root) = search else {
            continue;
        };
        let Ok(entries) = std::fs::read_dir(search_root) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir() && path.join("manifest.yaml").is_file())
            .collect();
        paths.sort();
        candidates.extend(
            paths
                .into_iter()
                .filter_map(|path| semantic_metadata(&path)),
        );
    }

    let mut resolved = BTreeSet::new();
    while let Some(dependency) = pending.pop_first() {
        if !resolved.insert(dependency.clone()) || available.contains(&dependency) {
            continue;
        }
        let Some(provider) = candidates.iter().find(|module| {
            (module.package.as_str(), module.version.as_str()) == (&dependency.0, &dependency.1)
        }) else {
            continue;
        };
        if !roots.iter().any(|root| root == &provider.root) {
            roots.push(provider.root.clone());
        }
        available.insert(dependency);
        pending.extend(
            provider
                .imports
                .iter()
                .map(|(package, version)| (package.clone(), version.clone())),
        );
    }
}

/// Read only the manifest identity and semantic dependency map needed to
/// select a provider. Invalid manifests are left for the final engine load to
/// diagnose with its normal, complete error context.
fn semantic_metadata(module_root: &Path) -> Option<ModuleSemanticMetadata> {
    let manifest = quire_rs::loader::manifest::load_manifest(module_root).ok()?;
    let semantic = serde_json::to_value(manifest.semantic?).ok()?;
    let package = semantic.get("package")?.as_str()?.to_owned();
    let imports: BTreeMap<String, String> =
        serde_json::from_value(semantic.get("imports")?.clone()).ok()?;
    Some(ModuleSemanticMetadata {
        root: module_root.to_path_buf(),
        package,
        version: manifest.version?,
        imports,
    })
}

pub fn load_module_registry(ctx: &Ctx, module: &Path) -> anyhow::Result<Registry> {
    let registry = Registry::load_module(module)
        .map_err(failure::invalid)
        .context("loading module registry")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    if registry.module_names().count() == 0 {
        if let Some(f) = registry.failures().first() {
            invalid_request!("module load failed: {} ({})", f.reason, f.path.display());
        }
    }
    Ok(registry)
}

/// Refuse a registry whose model lost declarations at load, naming every one
/// (quire-rs FR-058-AC-47). Checked before any emptiness test of the model, so
/// a wholly dropped `traceability:` section is named rather than reported as an
/// undeclared model.
pub fn require_complete_model(registry: &Registry) -> anyhow::Result<()> {
    let Err(dropped) = registry.require_complete_model() else {
        return Ok(());
    };
    let details: Vec<String> = registry
        .failures()
        .iter()
        .filter(|failure| {
            failure.declaration.as_deref().is_some_and(|declaration| {
                dropped.0.iter().any(|dropped| {
                    dropped.module == failure.module
                        && dropped.kind == declaration.kind
                        && dropped.item == declaration.item
                })
            })
        })
        .map(|failure| format!("{}: {}", failure.path.display(), failure.reason))
        .collect();
    let message = if details.is_empty() {
        dropped.to_string()
    } else {
        format!("{dropped}; load details: {}", details.join("; "))
    };
    Err(invalid_error!("{message}"))
}

/// The `ModelUndeclared`-shaped refusal `coverage` and `trace` both need:
/// neither has anything to reconcile without a declared `traceability:`
/// model. Shared, not restated, so the two commands cannot independently
/// drift on its wording the way an inline copy-pasted `bail!` would let them
/// (review finding, PLAT-879 PR #99) — and returns the model directly rather
/// than requiring a caller to re-derive it with a second `.expect()` after
/// the check, which was itself a finding against the first draft of `trace`.
pub fn require_traceability_model(
    registry: &Registry,
) -> anyhow::Result<&quire_rs::traceability::TraceabilityModel> {
    require_complete_model(registry)?;
    registry.traceability().ok_or_else(|| {
        invalid_error!(
            "no module in scope declares a `traceability:` model, so there is \
             nothing to reconcile; install a module that declares one (e.g. \
             spec-artifacts-process) or pass --module"
        )
    })
}
