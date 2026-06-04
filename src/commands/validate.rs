//! `quire validate <DOC.md|-> [--module <PATH>] [--archetype <NAME>]`        (default: markdown)
//! `quire validate <ARCHETYPE> --module <PATH> --json <FILE|->`              (context/data)
//!
//! Default markdown mode surfaces `quire_rs::validate_document` (upstream
//! FR-032): structural validation of an authored document against a
//! unified archetype. The `--json` context mode preserves the legacy
//! JSON-object validation via `quire_rs::validate` (upstream FR-002).
//!
//! Exit 0 on success (no stdout). Exit 1 on validation failure, with the
//! quire-rs diagnostics surfaced verbatim on stderr. NEVER writes stdout.
//! All validation logic lives in quire-rs (StR-004 thin boundary).

use anyhow::{anyhow, bail, Context};
use clap::Parser;

use quire_cli::io::{self, emit_quire_diagnostics};
use quire_cli::safety;
use quire_rs::{Registry, ValidationResult};

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Markdown document path (or `-` for stdin) in the default mode; the
    /// archetype name when `--json` selects context mode.
    pub target: String,

    /// Path to the module directory (containing `manifest.yaml`).
    #[arg(long, value_name = "PATH")]
    pub module: String,

    /// Override the archetype resolved from the document frontmatter
    /// `artifact_type` (markdown mode only).
    #[arg(long, value_name = "NAME")]
    pub archetype: Option<String>,

    /// Validate a JSON context object (file or `-`) against the archetype
    /// schema instead of a markdown document. Selects context mode.
    #[arg(long, value_name = "FILE")]
    pub json: Option<String>,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;
    let registry = Registry::load_module(&module).context("loading module registry")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());

    match args.json.as_deref() {
        Some(json) => run_context(ctx, &args, &registry, json),
        None => run_markdown(ctx, &args, &registry),
    }
}

/// Default markdown mode: validate an authored document against its
/// archetype via `quire_rs::validate_document` (FR-032).
fn run_markdown(ctx: &Ctx, args: &Args, registry: &Registry) -> anyhow::Result<()> {
    if args.target != "-" {
        safety::validate_input_path("document", &args.target)
            .with_context(|| format!("validating document '{}'", args.target))?;
    }
    let text = io::read_text(&args.target).with_context(|| format!("reading '{}'", args.target))?;

    let archetype_name = match &args.archetype {
        Some(name) => name.clone(),
        None => archetype_from_frontmatter(&text)?,
    };
    let archetype = registry
        .archetype(&archetype_name)
        .ok_or_else(|| anyhow!("UnknownArchetype: '{archetype_name}' is not registered"))?;

    let result = quire_rs::validate_document(archetype, &text);
    surface_result(ctx, &result)
}

/// Context mode (`--json`): validate a JSON object against the archetype
/// schema via `quire_rs::validate` (FR-002).
fn run_context(ctx: &Ctx, args: &Args, registry: &Registry, json: &str) -> anyhow::Result<()> {
    if json != "-" {
        safety::validate_input_path("--json", json)
            .with_context(|| format!("validating --json '{json}'"))?;
    }
    let data = io::read_data(json).context("reading --json")?;

    let archetype = registry
        .archetype(&args.target)
        .ok_or_else(|| anyhow!("UnknownArchetype: '{}' is not registered", args.target))?;

    quire_rs::validate(archetype, &data)
        .with_context(|| format!("validating against archetype '{}'", args.target))?;
    let _ = ctx;
    Ok(())
}

/// Read the archetype name from the document's frontmatter
/// `artifact_type` field (markdown mode default resolution).
fn archetype_from_frontmatter(text: &str) -> anyhow::Result<String> {
    let doc = quire_rs::parse_document(text);
    let frontmatter = doc
        .frontmatter
        .as_ref()
        .ok_or_else(|| anyhow!("document has no frontmatter; cannot resolve archetype"))?;
    frontmatter
        .get("artifact_type")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            anyhow!("frontmatter has no string 'artifact_type'; pass --archetype to override")
        })
}

/// Exit 0 when valid; on failure surface each quire-rs diagnostic
/// verbatim on stderr (line-numbered) and exit 1 via a user error.
fn surface_result(ctx: &Ctx, result: &ValidationResult) -> anyhow::Result<()> {
    if result.is_valid {
        return Ok(());
    }
    for error in &result.errors {
        let line = match error.line {
            Some(l) => format!("line {l}: "),
            None => String::new(),
        };
        let message = format!("{line}{} [{}]", error.message, error.reason.as_str());
        io::emit_diagnostic(ctx.diagnostics, "ValidationError", &message);
    }
    bail!("document failed structural validation")
}
