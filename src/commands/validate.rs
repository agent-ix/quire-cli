//! `quire validate <DOC.md|GLOB|->... [--scope <DIR>] [--module <PATH>] [--archetype <NAME>]`
//!
//! Markdown-only structural validation: surfaces `quire_rs::validate_document`
//! (upstream FR-032) — `body_extraction` asserts + frontmatter-schema +
//! per-level heading uniqueness over an authored document. The context/data
//! mode was removed with the render retirement (FR-004 CR note, 2026-06-04);
//! no backward-compatibility layer.
//!
//! Exit 0 on success (no stdout). Exit 1 on validation failure, with the
//! quire-rs diagnostics surfaced verbatim on stderr. NEVER writes stdout.
//! All validation logic lives in quire-rs (StR-004 thin boundary).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use clap::Parser;
use glob::glob;

use quire_cli::io;
use quire_cli::safety;
use quire_rs::{Registry, ValidationResult};

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Markdown document path, glob, or `-` for stdin. Relative globs are
    /// resolved under --scope when --module is omitted.
    #[arg(value_name = "DOC_OR_GLOB", required = true)]
    pub documents: Vec<String>,

    /// Path to one exact module directory (containing `manifest.yaml`).
    /// Kept for explicit single-module validation and stdin use.
    #[arg(long, value_name = "PATH")]
    pub module: Option<String>,

    /// Directory that bounds relative document globs and repo-local module
    /// discovery. Scoped mode also reads IX_SCHEMA_PATH for installed modules.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub scope: String,

    /// Override the archetype resolved from the document frontmatter
    /// `artifact_type`.
    #[arg(long, value_name = "NAME")]
    pub archetype: Option<String>,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let scoped = args.module.is_none();
    let scope = safety::validate_dir_path("--scope", &args.scope)
        .with_context(|| format!("validating --scope '{}'", args.scope))?;
    let registry = load_registry(ctx, &args, &scope)?;
    let inputs = expand_documents(&args.documents, &scope, scoped)?;

    let mut failures = 0usize;
    for input in inputs {
        let label = input.label();
        let text = input.read().with_context(|| format!("reading '{label}'"))?;
        let archetype_name = match &args.archetype {
            Some(name) => name.clone(),
            None => archetype_from_frontmatter(&text)?,
        };
        let archetype = registry
            .archetype(&archetype_name)
            .ok_or_else(|| anyhow!("UnknownArchetype: '{archetype_name}' is not registered"))?;

        let result = quire_rs::validate_document(archetype, &text);
        if !surface_result(ctx, &label, &result) {
            failures += 1;
        }
    }

    if failures > 0 {
        bail!("{failures} document(s) failed structural validation");
    }
    Ok(())
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

    let roots = scoped_registry_roots(scope);
    let refs: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
    let registry = Registry::load_from(&refs).context("loading scoped module registry")?;
    io::emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    if registry.module_names().count() == 0 {
        if let Some(f) = registry.failures().first() {
            bail!("module load failed: {} ({})", f.reason, f.path.display());
        }
        bail!(
            "no modules found for scoped validation; add modules under --scope, \
             ~/.ix/schemas, or set IX_SCHEMA_PATH"
        );
    }
    Ok(registry)
}

fn scoped_registry_roots(scope: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut seen = HashSet::new();
    push_root(&mut roots, &mut seen, scope.to_path_buf());

    let ix_modules = scope.join(".ix").join("modules");
    if ix_modules.is_dir() {
        push_root(&mut roots, &mut seen, ix_modules);
    }

    if let Some(paths) = std::env::var_os("IX_SCHEMA_PATH") {
        for path in std::env::split_paths(&paths) {
            if path.is_dir() {
                push_root(&mut roots, &mut seen, path);
            }
        }
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

/// Read the archetype name from the document's frontmatter
/// `artifact_type` field (default resolution, FR-004-AC-4/AC-5).
fn archetype_from_frontmatter(text: &str) -> anyhow::Result<String> {
    let doc = quire_rs::parse_document(text);
    let frontmatter = doc.frontmatter.as_ref().ok_or_else(|| {
        anyhow!(
            "document has no frontmatter from which to resolve the archetype; \
             add a frontmatter block with `artifact_type`, or pass --archetype <NAME>"
        )
    })?;
    frontmatter
        .get("artifact_type")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            anyhow!(
                "frontmatter has no string `artifact_type` from which to resolve the archetype; \
                 add `artifact_type`, or pass --archetype <NAME>"
            )
        })
}

/// Exit 0 when valid; on failure surface each quire-rs diagnostic
/// verbatim on stderr (line-numbered) and exit 1 via a user error.
fn surface_result(ctx: &Ctx, label: &str, result: &ValidationResult) -> bool {
    if result.is_valid {
        return true;
    }
    for error in &result.errors {
        let line = match error.line {
            Some(l) => format!("line {l}: "),
            None => String::new(),
        };
        let message = format!(
            "{label}: {line}{} [{}]",
            error.message,
            error.reason.as_str()
        );
        io::emit_diagnostic(ctx.diagnostics, "ValidationError", &message);
    }
    false
}
