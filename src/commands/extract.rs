//! `quire extract <doc|-> --module <path>`
//!
//! Surfaces `quire_rs::extract` + `quire_rs::harvest_edges`
//! (upstream FR-011 / FR-015).
//!
//! Selection: the archetype is read from the document's frontmatter
//! `artifact_type` field. The DSL is read out of the module's
//! `manifest.yaml` via `quire_rs::loader::manifest::load_manifest`,
//! because `quire_rs::CompiledArchetype` does not retain the parsed
//! DSL after load (it is structurally validated and dropped). Re-
//! parsing the manifest is a fixed-cost startup tax acceptable for
//! the agent extract path (see F-5 / R-4 in plan).
//!
//! Per F-5 (plan.md), this command does NOT auto-validate the document.

use anyhow::{anyhow, Context};
use clap::Parser;
use serde::Serialize;

use quire_cli::io::{self, emit_quire_diagnostics};
use quire_cli::safety;
use quire_rs::loader::manifest::load_manifest;
use quire_rs::{harvest_edges, IdentityResolver};

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
    r#type: String,
    target: String,
    metadata: serde_json::Map<String, serde_json::Value>,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;

    let text = io::read_text(&args.doc).with_context(|| format!("reading '{}'", args.doc))?;
    let doc = quire_rs::parse_document(&text);

    let archetype_name = match args.archetype.as_deref() {
        Some(name) => name.to_string(),
        None => doc
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

    let manifest = load_manifest(&module).map_err(|e| anyhow!("loading manifest.yaml: {e}"))?;
    let object_type = manifest
        .object_types
        .iter()
        .find(|ot| ot.name == archetype_name)
        .ok_or_else(|| {
            anyhow!(
                "archetype '{}' is not an object_type in module '{}' \
                 (extract requires an object_type with body_extraction)",
                archetype_name,
                module.display()
            )
        })?;
    let dsl = object_type.body_extraction.as_ref().ok_or_else(|| {
        anyhow!(
            "object_type '{}' has no 'body_extraction' DSL — nothing to extract",
            archetype_name
        )
    })?;

    let extraction = quire_rs::extract(&doc, dsl).context("evaluating extraction DSL")?;
    emit_quire_diagnostics(ctx.diagnostics, extraction.diagnostics.iter());

    // Use the document's `id` from frontmatter as the source ref if
    // present; otherwise the archetype name. Bare-target resolution is
    // identity for the CLI (no project-aware org/repo mapping here).
    let source_ref = doc
        .frontmatter
        .as_ref()
        .and_then(|fm| fm.get("id").and_then(|v| v.as_str()))
        .unwrap_or(&archetype_name)
        .to_string();

    let resolver = IdentityResolver;
    let harvest = harvest_edges(&doc, &source_ref, Some(&extraction), &resolver);
    emit_quire_diagnostics(ctx.diagnostics, harvest.diagnostics.iter());

    let envelope = ExtractEnvelope {
        extraction: ExtractionShape {
            records: &extraction.records,
        },
        edges: harvest
            .edges
            .iter()
            .map(|e| EdgeShape {
                r#type: e.r#type.clone(),
                target: e.target.clone(),
                metadata: e
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            })
            .collect(),
    };

    let payload = io::encode_json(&envelope, ctx.pretty).context("encoding extract envelope")?;
    io::write_primary_stdout(payload.as_bytes()).context("writing extract output")?;
    io::write_primary_stdout(b"\n").ok();
    Ok(())
}
