//! `quire validate <DOC.md|GLOB|->... [--scope <DIR>] [--module <PATH>] [--archetype <NAME>] [--strict]`
//!
//! Markdown-only structural validation: surfaces
//! `quire_rs::validate_document_in_registry` (upstream FR-032-AC-11..13) —
//! composed validation of the `type` archetype AND the frontmatter `object:`
//! archetype: `body_extraction` asserts + frontmatter-schema + per-level
//! heading uniqueness over an authored document. The context/data mode was
//! removed with the render retirement (FR-004 CR note, 2026-06-04); no
//! backward-compatibility layer.
//!
//! The engine result carries `errors` (exit-failing) and `warnings`
//! (advisory — today only the unknown-`object:` case). Both are surfaced on
//! stderr; warnings are clearly marked (`warning:` prefix / distinct JSON
//! `severity`). Exit 0 on success; exit 1 on any error, or — with `--strict`
//! — on any warning too. NEVER writes stdout. All validation logic lives in
//! quire-rs (StR-004 thin boundary).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use clap::Parser;
use glob::glob;

use quire_cli::io;
use quire_cli::safety;
use quire_rs::{BundlePosture, BundleReport, Registry, ValidationResult};

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Markdown document path, glob, or `-` for stdin. Relative globs are
    /// resolved under --scope when --module is omitted. With --okf, an
    /// optional bundle directory (defaults to --scope).
    #[arg(value_name = "DOC_OR_GLOB", required_unless_present = "okf")]
    pub documents: Vec<String>,

    /// Path to one exact module directory (containing `manifest.yaml`).
    /// Kept for explicit single-module validation and stdin use.
    #[arg(long, value_name = "PATH")]
    pub module: Option<String>,

    /// Directory that bounds relative document globs and repo-local module
    /// discovery. Scoped mode also searches the default install root
    /// (~/.ix/filament/modules) and the IX_FILAMENT_MODULES_PATH /
    /// IX_SCHEMA_PATH env vars; when nothing is found it lazy-installs the
    /// default module set via `quoin plugin ensure-defaults`.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub scope: String,

    /// Override the archetype resolved from the document frontmatter
    /// `type`.
    #[arg(long, value_name = "NAME")]
    pub archetype: Option<String>,

    /// Validate a directory as an OKF bundle under the permissive posture:
    /// `type` is still required, but unknown types, broken `ix://` links,
    /// and `index.md` completeness gaps are warnings, not errors. Operates
    /// on the positional bundle directory, or --scope when none is given.
    #[arg(long)]
    pub okf: bool,

    /// Treat advisory warnings as failures: with --strict, any warning
    /// (today only the unknown-`object:` case from composed type+object
    /// validation, FR-032-AC-12) makes `validate` exit 1. Warnings are
    /// always printed; --strict only changes the exit code.
    #[arg(long)]
    pub strict: bool,

    /// Print an EARS requirement-grammar summary (FR-042) after validation:
    /// documents scanned, how many are grammar-clean (doc-level conformance),
    /// and a per-check finding histogram. Advisory — never changes the exit
    /// code. Findings are also printed inline as warnings regardless.
    #[arg(long)]
    pub summary: bool,
}

/// The EARS check id inside a grammar warning message (`[ears:<check>] …`),
/// or `None` for a non-grammar warning.
fn grammar_check_name(message: &str) -> Option<&str> {
    let rest = message.strip_prefix("[ears:")?;
    let end = rest.find(']')?;
    Some(&rest[..end])
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let scoped = args.module.is_none();
    let scope = safety::validate_dir_path("--scope", &args.scope)
        .with_context(|| format!("validating --scope '{}'", args.scope))?;
    let registry = load_registry(ctx, &args, &scope)?;

    if args.okf {
        return run_okf(ctx, &args, &scope, scoped, &registry);
    }

    // clap guarantees a non-empty `documents` here (required_unless_present).
    let inputs = expand_documents(&args.documents, &scope, scoped)?;

    // FR-044: harvest the scope repo's project Ubiquitous-Language terms (a
    // `Glossary` `## Terms` table or `## Ubiquitous Language` sections) once,
    // and compose the combined (module ∪ project) lexicon the EARS grammar
    // check consumes for every validated file. Empty when the repo has none.
    // Scans only glossary-bearing docs — never a full-corpus parse.
    let project_terms = quire_rs::glossary_terms_from_path(&scope);
    let lexicon = registry.lexicon_with(&project_terms);

    let mut failures = 0usize;
    let mut warned = 0usize;
    // EARS grammar summary accumulators (FR-042) — populated only to print
    // the optional --summary block; never affect the exit code.
    let mut docs_scanned = 0usize;
    let mut docs_grammar_clean = 0usize;
    let mut grammar_checks: std::collections::BTreeMap<String, usize> = Default::default();
    for input in inputs {
        let label = input.label();
        let text = input.read().with_context(|| format!("reading '{label}'"))?;
        // Discriminator resolution (the one piece that must be code: a
        // schema can't select itself). Missing/unknown `type` is a
        // per-document validation failure surfaced as a `frontmatter`
        // diagnostic — not a run-aborting bail — so a batch reports every
        // bad document, not just the first.
        let archetype_name = match &args.archetype {
            Some(name) => name.clone(),
            None => match archetype_from_frontmatter(&text) {
                Some(name) => name,
                None => {
                    emit_frontmatter_failure(
                        ctx,
                        &label,
                        "required 'type' is missing from frontmatter (add `type:`, or pass --archetype <NAME>)",
                    );
                    failures += 1;
                    continue;
                }
            },
        };
        let archetype = match registry.archetype(&archetype_name) {
            Some(a) => a,
            // An explicit `--archetype` that doesn't exist is a usage
            // error, not document data → fail fast (IT-013/IT-050).
            None if args.archetype.is_some() => {
                bail!("UnknownArchetype: '{archetype_name}' is not registered");
            }
            // Resolved from frontmatter `type` but unregistered: a
            // per-document data error, surfaced like any frontmatter fault.
            None => {
                emit_frontmatter_failure(
                    ctx,
                    &label,
                    &format!("unknown type '{archetype_name}' (no archetype registered for it)"),
                );
                failures += 1;
                continue;
            }
        };

        // Composed type+object validation (FR-032-AC-11..13): the registry
        // is available, so resolve the frontmatter `object:` archetype too.
        let result = quire_rs::validate_document_in_registry_with_lexicon(
            &registry, archetype, &text, &lexicon,
        );
        let outcome = surface_result(ctx, &label, &result);
        if outcome.had_errors {
            failures += 1;
        }
        if outcome.had_warnings {
            warned += 1;
        }
        // Tally EARS grammar findings for the optional summary.
        docs_scanned += 1;
        let mut doc_grammar = 0usize;
        for w in &result.warnings {
            if let Some(check) = grammar_check_name(&w.message) {
                *grammar_checks.entry(check.to_string()).or_default() += 1;
                doc_grammar += 1;
            }
        }
        if doc_grammar == 0 {
            docs_grammar_clean += 1;
        }
    }

    if args.summary {
        emit_grammar_summary(ctx, docs_scanned, docs_grammar_clean, &grammar_checks);
    }

    if failures > 0 {
        bail!("{failures} document(s) failed structural validation");
    }
    // --strict escalates advisory warnings to a failing exit code; warnings
    // were already printed above (FR-004-AC-10/AC-11).
    if args.strict && warned > 0 {
        bail!("{warned} document(s) emitted warnings (--strict)");
    }
    Ok(())
}

/// Emit the EARS requirement-grammar summary (FR-042) on stderr via the shared
/// diagnostic channel (human or JSON per --diagnostics-format). Advisory: a
/// one-line histogram + doc-level conformance, never affecting the exit code.
fn emit_grammar_summary(
    ctx: &Ctx,
    docs_scanned: usize,
    docs_clean: usize,
    checks: &std::collections::BTreeMap<String, usize>,
) {
    let total_findings: usize = checks.values().sum();
    let pct = if docs_scanned == 0 {
        100
    } else {
        docs_clean * 100 / docs_scanned
    };
    let histogram = if checks.is_empty() {
        "none".to_string()
    } else {
        checks
            .iter()
            .map(|(c, n)| format!("{c}={n}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let message = format!(
        "{docs_clean}/{docs_scanned} docs grammar-clean ({pct}%); \
         {total_findings} EARS finding(s): {histogram}"
    );
    io::emit_diagnostic(ctx.diagnostics, "GrammarSummary", &message);
}

/// OKF bundle validation (permissive posture). Validates each bundle
/// directory wholesale via `quire_rs::validate_bundle_at`, surfacing
/// warnings and errors on stderr. Exit 1 only when there are hard errors
/// (untyped documents) — unknown types / broken links / index gaps warn.
fn run_okf(
    ctx: &Ctx,
    args: &Args,
    scope: &Path,
    scoped: bool,
    registry: &Registry,
) -> anyhow::Result<()> {
    let roots: Vec<PathBuf> = if args.documents.is_empty() {
        vec![scope.to_path_buf()]
    } else {
        args.documents
            .iter()
            .map(|raw| scoped_path(scope, scoped, raw))
            .collect()
    };

    let mut errors = 0usize;
    for root in roots {
        let root = safety::validate_dir_path("bundle", &root.display().to_string())
            .with_context(|| format!("validating bundle root '{}'", root.display()))?;
        let report = quire_rs::validate_bundle_at(&root, registry, BundlePosture::Okf);
        surface_bundle(ctx, &report);
        errors += report.errors.len();
    }

    if errors > 0 {
        bail!("{errors} OKF bundle validation error(s)");
    }
    Ok(())
}

/// Surface a [`BundleReport`] on stderr: warnings first (non-fatal),
/// then errors, both in the shared `quire_rs` diagnostic shape.
fn surface_bundle(ctx: &Ctx, report: &BundleReport) {
    for w in &report.warnings {
        io::emit_diagnostic(
            ctx.diagnostics,
            "Diagnostic",
            &format!("{}: {} [{}]", w.path.display(), w.message, w.reason),
        );
    }
    for e in &report.errors {
        io::emit_diagnostic(
            ctx.diagnostics,
            "ValidationError",
            &format!("{}: {} [{}]", e.path.display(), e.message, e.reason),
        );
    }
}

fn load_registry(ctx: &Ctx, args: &Args, scope: &Path) -> anyhow::Result<Registry> {
    if let Some(raw) = &args.module {
        let module = safety::validate_module_path(raw)
            .with_context(|| format!("validating --module '{raw}'"))?;
        return super::load_module_registry(ctx, &module);
    }

    if scope.join("manifest.yaml").is_file() {
        return super::load_module_registry(ctx, scope);
    }

    // Scoped discovery. Load once; if nothing is found, lazy-install the
    // default module set via quoin and reload exactly once before failing —
    // so a fresh machine validates without any manual `quoin` step or env var.
    let mut registry = load_scoped_registry(scope)?;
    let mut installed = false;
    if registry.module_names().count() == 0 && lazy_init_default_modules(ctx) {
        installed = true;
        registry = load_scoped_registry(scope)?;
    }
    io::emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    if registry.module_names().count() == 0 {
        if let Some(f) = registry.failures().first() {
            bail!("module load failed: {} ({})", f.reason, f.path.display());
        }
        if installed {
            bail!(
                "no modules found after installing the default set via quoin; \
                 check `quoin plugin ensure-defaults`"
            );
        }
        bail!(
            "no modules found for scoped validation, and automatic install via \
             quoin was unavailable; install quoin and run `quoin plugin \
             ensure-defaults` (modules install to ~/.ix/filament/modules), or set \
             IX_FILAMENT_MODULES_PATH"
        );
    }
    Ok(registry)
}

/// Build the scoped search roots and load a [`Registry`] from them.
fn load_scoped_registry(scope: &Path) -> anyhow::Result<Registry> {
    let roots = scoped_registry_roots(scope);
    let refs: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
    Registry::load_from(&refs).context("loading scoped module registry")
}

/// Best-effort lazy install of the default Filament module set by shelling out
/// to `quoin plugin ensure-defaults`. Returns `true` only when quoin ran and
/// exited successfully. A missing `quoin` (or any failure) returns `false`, so
/// the caller falls through to the standard "no modules" guidance. The child's
/// stdout is captured (never forwarded) to preserve the stdout-silent contract.
fn lazy_init_default_modules(ctx: &Ctx) -> bool {
    io::emit_diagnostic(
        ctx.diagnostics,
        "Diagnostic",
        "no spec modules found; installing the default set via `quoin plugin ensure-defaults`",
    );
    let output = match std::process::Command::new("quoin")
        .args(["plugin", "ensure-defaults"])
        .output()
    {
        Ok(output) => output,
        Err(_) => {
            io::emit_diagnostic(
                ctx.diagnostics,
                "Diagnostic",
                "quoin not found on PATH; cannot auto-install default modules",
            );
            return false;
        }
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        io::emit_diagnostic(
            ctx.diagnostics,
            "Diagnostic",
            &format!("quoin failed to install default modules: {}", stderr.trim()),
        );
        return false;
    }
    true
}

fn scoped_registry_roots(scope: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut seen = HashSet::new();
    push_root(&mut roots, &mut seen, scope.to_path_buf());

    let ix_modules = scope.join(".ix").join("modules");
    if ix_modules.is_dir() {
        push_root(&mut roots, &mut seen, ix_modules);
    }

    // Honour the engine's module-path env vars: IX_FILAMENT_MODULES_PATH is
    // preferred, IX_SCHEMA_PATH is the legacy alias (mirrors quire-rs
    // loader::paths::module_path_env). Both are unioned into the search set.
    for var in ["IX_FILAMENT_MODULES_PATH", "IX_SCHEMA_PATH"] {
        if let Some(paths) = std::env::var_os(var) {
            for path in std::env::split_paths(&paths) {
                if path.is_dir() {
                    push_root(&mut roots, &mut seen, path);
                }
            }
        }
    }

    // The canonical install root quoin materializes the default module set
    // into, and the same directory quire-rs reads by default. Including it
    // here lets scoped validation find installed defaults with no env var set.
    if let Some(root) = quire_rs::loader::paths::default_module_root() {
        push_root(&mut roots, &mut seen, root);
    }

    roots
}

fn push_root(roots: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>, root: PathBuf) {
    if seen.insert(root.clone()) {
        roots.push(root);
    }
}

#[derive(Debug)]
enum DocumentInput {
    Stdin,
    Path(PathBuf),
}

impl DocumentInput {
    fn label(&self) -> String {
        match self {
            Self::Stdin => "-".to_string(),
            Self::Path(path) => path.display().to_string(),
        }
    }

    fn read(&self) -> anyhow::Result<String> {
        match self {
            Self::Stdin => io::read_text("-"),
            Self::Path(path) => std::fs::read_to_string(path).map_err(Into::into),
        }
    }
}

fn expand_documents(
    raw_documents: &[String],
    scope: &Path,
    scoped: bool,
) -> anyhow::Result<Vec<DocumentInput>> {
    let mut inputs = Vec::new();
    let mut seen = HashSet::new();

    for raw in raw_documents {
        if raw == "-" {
            inputs.push(DocumentInput::Stdin);
            continue;
        }

        if contains_glob(raw) {
            let pattern = scoped_path(scope, scoped, raw);
            let pattern_string = pattern.display().to_string();
            let mut matched = 0usize;
            for entry in
                glob(&pattern_string).with_context(|| format!("expanding document glob '{raw}'"))?
            {
                let path = entry.with_context(|| format!("reading document glob '{raw}'"))?;
                if !path.is_file() {
                    continue;
                }
                let path = safety::validate_input_path("document", &path.display().to_string())
                    .with_context(|| format!("validating document '{}'", path.display()))?;
                if seen.insert(path.clone()) {
                    inputs.push(DocumentInput::Path(path));
                }
                matched += 1;
            }
            if matched == 0 {
                bail!("document glob matched no files: '{raw}'");
            }
            continue;
        }

        let path = scoped_path(scope, scoped, raw);
        let path = safety::validate_input_path("document", &path.display().to_string())
            .with_context(|| format!("validating document '{}'", path.display()))?;
        if seen.insert(path.clone()) {
            inputs.push(DocumentInput::Path(path));
        }
    }

    Ok(inputs)
}

fn scoped_path(scope: &Path, scoped: bool, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if scoped && path.is_relative() {
        scope.join(path)
    } else {
        path
    }
}

fn contains_glob(raw: &str) -> bool {
    raw.chars().any(|c| matches!(c, '*' | '?' | '['))
}

/// Resolve the archetype name from the document's frontmatter `type`
/// (default resolution, FR-004-AC-4/AC-5) via the one canonical
/// discriminator read. `None` when the document carries no `type`.
fn archetype_from_frontmatter(text: &str) -> Option<String> {
    let doc = quire_rs::parse_document(text);
    quire_rs::concept_type(&doc).map(str::to_string)
}

/// Surface a missing/unknown-`type` resolution failure in the same
/// line-numbered shape as a `quire_rs::ValidationError` with reason
/// `frontmatter`, so callers see one consistent diagnostic vocabulary.
fn emit_frontmatter_failure(ctx: &Ctx, label: &str, message: &str) {
    io::emit_diagnostic(
        ctx.diagnostics,
        "ValidationError",
        &format!("{label}: {message} [frontmatter]"),
    );
}

/// What a single document's validation surfaced.
struct SurfaceOutcome {
    had_errors: bool,
    had_warnings: bool,
}

/// Surface each quire-rs diagnostic verbatim on stderr (line-numbered):
/// errors as errors, warnings clearly marked (`warning:` prefix / distinct
/// JSON severity). Errors are exit-failing; warnings are advisory and
/// printed regardless of validity (FR-004-AC-10/AC-12). Returns which
/// severities appeared so the caller can compute the exit code.
fn surface_result(ctx: &Ctx, label: &str, result: &ValidationResult) -> SurfaceOutcome {
    for error in &result.errors {
        let line = line_prefix(error.line);
        let message = format!(
            "{label}: {line}{} [{}]",
            error.message,
            error.reason.as_str()
        );
        io::emit_diagnostic(ctx.diagnostics, "ValidationError", &message);
    }
    for warning in &result.warnings {
        let line = line_prefix(warning.line);
        let message = format!(
            "{label}: {line}{} [{}]",
            warning.message,
            warning.reason.as_str()
        );
        io::emit_warning(ctx.diagnostics, &message);
    }
    SurfaceOutcome {
        had_errors: !result.errors.is_empty(),
        had_warnings: !result.warnings.is_empty(),
    }
}

/// Render an optional 1-based line number as a `line N: ` prefix.
fn line_prefix(line: Option<usize>) -> String {
    match line {
        Some(l) => format!("line {l}: "),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_roots_include_scope_and_default_install_root() {
        let scope = Path::new("/tmp/quire-cli-scope-roots-test");
        let roots = scoped_registry_roots(scope);
        // The --scope directory is always the first search root.
        assert_eq!(roots.first(), Some(&scope.to_path_buf()));
        // The canonical install root (~/.ix/filament/modules) is included so
        // scoped validation finds quoin-installed defaults with no env var set.
        if let Some(default_root) = quire_rs::loader::paths::default_module_root() {
            assert!(
                roots.contains(&default_root),
                "default install root {default_root:?} missing from {roots:?}"
            );
        }
    }
}
