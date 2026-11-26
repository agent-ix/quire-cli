//! `quire extract <doc|-> --module <path>`
//!
//! Surfaces `quire_rs::extract` + `quire_rs::harvest_edges`
//! (upstream FR-011 / FR-015).
//!
//! Selection: the archetype is read from the document's frontmatter
//! `type` field, or overridden via
//! `--archetype`. The DSL is read from
//! `CompiledArchetype::body_extraction()` — no manifest re-read.
//!
//! Default mode does not auto-validate the document. Semantic mode surfaces
//! the engine semantic extractor's typed records and refusals.

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use ix_trace_rs::implements;
use serde::Serialize;

use quire_cli::io::{self, emit_quire_diagnostics};
use quire_rs::{harvest_edges, LoadedDocument};

use super::Ctx;

mod semantic;

#[derive(Parser, Debug)]
pub struct Args {
    /// Document path (or `-` for stdin).
    pub doc: String,

    /// Module directory (containing `manifest.yaml`).
    #[arg(long, value_name = "PATH", required = true)]
    pub module: Vec<String>,

    /// Include the engine semantic record, resolved against explicit corpus members.
    #[arg(long, requires = "repo")]
    pub semantic: bool,

    /// Corpus member `<ORG/REPO>=<DIR>` supplying semantic reference targets.
    #[arg(long, value_name = "ORG/REPO=DIR", value_parser = super::corpus_input::parse_repo, requires = "semantic")]
    pub repo: Vec<super::corpus_input::RepoArg>,

    /// Override the archetype lookup; default reads frontmatter `type`.
    #[arg(long, value_name = "NAME")]
    pub archetype: Option<String>,
}

#[derive(Serialize)]
struct ExtractEnvelope<'a> {
    extraction: ExtractionShape<'a>,
    edges: Vec<EdgeShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic: Option<quire_rs::SemanticExtraction>,
}

#[derive(Serialize)]
struct ExtractionShape<'a> {
    records: &'a [serde_json::Map<String, serde_json::Value>],
}

#[derive(Serialize)]
struct EdgeShape {
    target: String,
    r#type: String,
}

#[implements(
    "FR-003-AC-1",
    "FR-003-AC-2",
    "FR-003-AC-4",
    "FR-003-AC-5",
    "FR-003-AC-6",
    "FR-003-AC-7",
    "FR-003-AC-8",
    "FR-003-AC-9",
    "FR-003-AC-10"
)]
pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    if args.semantic && args.doc == "-" {
        invalid_request!("--semantic requires a file loaded from a declared --repo member");
    }
    let text = io::read_text(&args.doc)
        .map_err(super::failure::input)
        .with_context(|| format!("reading '{}'", args.doc))?;
    let parsed = quire_rs::parse_document(&text);

    let archetype_name = match args.archetype.as_deref() {
        Some(name) => name.to_string(),
        None => match quire_rs::concept_type(&parsed) {
            Some(name) => name.to_string(),
            // Surface the missing discriminator in the same `frontmatter`
            // diagnostic vocabulary `validate` uses, then exit 1.
            None => {
                ix_cli_kit::streams::Record::error(
                    "ValidationError",
                    &format!(
                        "{}: required 'type' is missing from frontmatter \
                         (add `type:`, or pass --archetype <NAME>) [frontmatter]",
                        args.doc
                    ),
                )
                .emit(ctx.diagnostics);
                invalid_request!("missing required 'type'");
            }
        },
    };

    let registry = super::load_module_set_registry(ctx, &args.module)?;

    let compiled = registry.archetype(&archetype_name).ok_or_else(|| {
        invalid_error!(
            "archetype '{}' not registered in module '{}'",
            archetype_name,
            args.module.join(", ")
        )
    })?;
    let dsl = compiled.body_extraction().ok_or_else(|| {
        invalid_error!(
            "archetype '{}' has no 'body_extraction' DSL — nothing to extract",
            archetype_name
        )
    })?;

    let semantic = if args.semantic {
        Some(semantic::extract(ctx, &args, &registry, compiled, &text)?)
    } else {
        None
    };

    let extraction = quire_rs::extract(&parsed, dsl)
        .map_err(super::failure::invalid)
        .context("evaluating extraction DSL")?;
    emit_quire_diagnostics(ctx.diagnostics, extraction.diagnostics.iter());

    // Wrap the parsed doc in a `LoadedDocument` so `harvest_edges` can
    // walk frontmatter relationships + body `ix://` links.
    let doc_id = parsed
        .frontmatter
        .as_ref()
        .and_then(|fm| fm.get("id").and_then(|v| v.as_str()))
        .unwrap_or(&archetype_name)
        .to_string();
    // CR-047: `LoadedDocument.doc` became a lazy tier behind accessors;
    // an already-parsed document enters via `from_parsed` (body pre-seeded).
    let loaded = LoadedDocument::from_parsed(PathBuf::from(&args.doc), doc_id, None, parsed);
    let edges: Vec<EdgeShape> = harvest_edges(&loaded)
        .into_iter()
        .map(|(target, r#type)| EdgeShape { target, r#type })
        .collect();

    let envelope = ExtractEnvelope {
        extraction: ExtractionShape {
            records: &extraction.records,
        },
        edges,
        semantic,
    };

    // #68: provenance is appended to the envelope, not folded into the inner
    // values — `extraction` and `edges` serialize in their own declaration
    // order, which is the FR-008 rule that this crate adds structure around
    // engine output and never rewrites it.
    let envelope = quire_cli::engine::attach(&envelope);
    let payload =
        ix_cli_kit::json::encode(&envelope, ctx.pretty).context("encoding extract envelope")?;
    ix_cli_kit::streams::write_primary_stdout(payload.as_bytes())
        .context("writing extract output")?;
    ix_cli_kit::streams::write_primary_stdout(b"\n")?;
    Ok(())
}
