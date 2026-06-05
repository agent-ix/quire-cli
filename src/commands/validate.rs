//! `quire validate <DOC.md|-> --module <PATH> [--archetype <NAME>]`
//!
//! Markdown-only structural validation: surfaces `quire_rs::validate_document`
//! (upstream FR-032) — `body_extraction` asserts + frontmatter-schema +
//! per-level heading uniqueness over an authored document. The `--json`
//! context/data mode was removed with the render retirement (FR-004 CR note,
//! 2026-06-04); no backward-compatibility layer.
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
    /// Markdown document path (or `-` for stdin).
    pub document: String,

    /// Path to the module directory (containing `manifest.yaml`).
    #[arg(long, value_name = "PATH")]
    pub module: String,

    /// Override the archetype resolved from the document frontmatter
    /// `artifact_type`.
    #[arg(long, value_name = "NAME")]
    pub archetype: Option<String>,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;
    let registry = Registry::load_module(&module).context("loading module registry")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());

    // A positional `-` is path-safety-exempt (stdin); any other value is a
    // document path and is checked under the `document` argument label.
    if args.document != "-" {
        safety::validate_input_path("document", &args.document)
            .with_context(|| format!("validating document '{}'", args.document))?;
    }
    let text =
        io::read_text(&args.document).with_context(|| format!("reading '{}'", args.document))?;

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
