//! `quire extract <doc|-> --module <path>`
//!
//! Surfaces `quire_rs::extract` + `quire_rs::harvest_edges`
//! (upstream FR-011 / FR-015).
//!
//! Selection: the archetype is read from the document's frontmatter
//! `artifact_type` field (or `object_type`), or overridden via
//! `--archetype`. The DSL is read from
//! `CompiledArchetype::body_extraction()` — no manifest re-read.
//!
//! Per F-5 (plan.md), this command does NOT auto-validate the document.

use std::path::PathBuf;

use anyhow::{anyhow, Context};
use clap::Parser;
use serde::Serialize;

use quire_cli::io::{self, emit_quire_diagnostics};
use quire_cli::safety;
use quire_rs::{harvest_edges, LoadedDocument};

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Document path (or `-` for stdin).
    pub doc: String,

    /// Module directory (containing `manifest.yaml`).
    #[arg(long, value_name = "PATH")]
    pub module: String,

    /// Override the archetype lookup; default reads frontmatter
    /// `artifact_type` (or `object_type`).
    #[arg(long, value_name = "NAME")]
    pub archetype: Option<String>,
}

#[derive(Serialize)]
struct ExtractEnvelope<'a> {
    extraction: ExtractionShape<'a>,
    edges: Vec<EdgeShape>,
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

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;

    let text = io::read_text(&args.doc).with_context(|| format!("reading '{}'", args.doc))?;
    let parsed = quire_rs::parse_document(&text);

    let archetype_name = match args.archetype.as_deref() {
        Some(name) => name.to_string(),
        None => parsed
            .frontmatter
            .as_ref()
            .and_then(|fm| {
                fm.get("object_type")
                    .or_else(|| fm.get("artifact_type"))
                    .and_then(|v| v.as_str())
            })
            .ok_or_else(|| {
                anyhow!(
                    "could not infer archetype: frontmatter has neither \
                     'object_type' nor 'artifact_type'; pass --archetype"
                )
            })?
            .to_string(),
    };

    let registry = super::load_module_registry(ctx, &module)?;

    let compiled = registry.archetype(&archetype_name).ok_or_else(|| {
        anyhow!(
            "archetype '{}' not registered in module '{}'",
            archetype_name,
            module.display()
        )
    })?;
    let dsl = compiled.body_extraction().ok_or_else(|| {
        anyhow!(
            "archetype '{}' has no 'body_extraction' DSL — nothing to extract",
            archetype_name
        )
    })?;

    let extraction = quire_rs::extract(&parsed, dsl).context("evaluating extraction DSL")?;
    emit_quire_diagnostics(ctx.diagnostics, extraction.diagnostics.iter());

    // Wrap the parsed doc in a `LoadedDocument` so `harvest_edges` can
    // walk frontmatter relationships + body `ix://` links.
    let doc_id = parsed
        .frontmatter
        .as_ref()
        .and_then(|fm| fm.get("id").and_then(|v| v.as_str()))
        .unwrap_or(&archetype_name)
        .to_string();
    let loaded = LoadedDocument {
        path: PathBuf::from(&args.doc),
        id: doc_id,
        uuid: None,
        doc: parsed,
    };
    let edges: Vec<EdgeShape> = harvest_edges(&loaded)
        .into_iter()
        .map(|(target, r#type)| EdgeShape { target, r#type })
        .collect();

    let envelope = ExtractEnvelope {
        extraction: ExtractionShape {
            records: &extraction.records,
        },
        edges,
    };

    let payload = io::encode_json(&envelope, ctx.pretty).context("encoding extract envelope")?;
    io::write_primary_stdout(payload.as_bytes()).context("writing extract output")?;
    io::write_primary_stdout(b"\n").ok();
    Ok(())
}
