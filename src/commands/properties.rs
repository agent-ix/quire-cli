//! `quire properties` — per-criterion property-shape classification
//! (quire-rs FR-052).
//!
//! The engine owns the whole computation. This command resolves paths and
//! archetypes exactly as `quire validate` does, calls
//! `quire_rs::classify_document_criteria`, and prints. It is the surface a
//! downstream property-test generator reads: `quire coverage --json` emits
//! only per-document *counts*, and the per-criterion records — `row_id`, the
//! `{domain, precondition, oracle}` spans, the shape, the signal trail — were
//! reachable only from Rust or PyO3 before this existed.
//!
//! **This command never emits a finding.** Classification carries no severity
//! and no check id, and it is not addressable by the FR-048 `grammar_severity`
//! registry (FR-052-CON-1), so nothing here can fail a build or be promoted to
//! an error. It reports; whether a low extractable ratio matters is the
//! consuming workflow's policy, not this command's.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use clap::Parser;
use quire_cli::io;
use quire_cli::safety;
use quire_rs::{AcClassification, Extraction, PropertySpan, Registry};
use serde_json::{json, Value};

use crate::commands::validate::{
    archetype_from_frontmatter, emit_frontmatter_failure, expand_documents,
};
use crate::commands::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    /// Documents to classify: paths, globs, or `-` for stdin. Relative paths
    /// resolve under `--scope` unless `--module` pins the module set.
    #[arg(value_name = "DOC_OR_GLOB", required = true)]
    pub documents: Vec<String>,

    /// Repository root the documents and the module discovery are scoped to.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub scope: String,

    /// Module directory supplying the archetypes and the `property_idioms`
    /// registry. When omitted the module set is discovered exactly as
    /// `quire validate` discovers it.
    #[arg(long, value_name = "PATH")]
    pub module: Option<String>,

    /// Override the archetype instead of reading frontmatter `type`.
    #[arg(long, value_name = "NAME")]
    pub archetype: Option<String>,

    /// Emit the records as JSON on stdout instead of the human census. The
    /// JSON is the stable interface; the human form may change.
    #[arg(long)]
    pub json: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let scoped = args.module.is_none();
    let scope = safety::validate_dir_path("--scope", &args.scope)
        .with_context(|| format!("validating --scope '{}'", args.scope))?;
    let registry = load_registry(ctx, &args, &scope)?;
    let inputs = expand_documents(&args.documents, &scope, scoped)?;

    let mut documents: Vec<Value> = Vec::new();
    let mut census = Census::default();
    let mut failures = 0usize;

    for input in inputs {
        let label = input.label();
        let text = input.read().with_context(|| format!("reading '{label}'"))?;

        // Same discriminator resolution as `validate`: a missing or unknown
        // `type` is a per-document failure, so a batch reports every bad
        // document rather than aborting on the first.
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
            None if args.archetype.is_some() => {
                bail!("UnknownArchetype: '{archetype_name}' is not registered");
            }
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

        // A non-binding archetype (US, IT, …) yields no records, exactly as it
        // yields no `ac` findings. That is data, not an error.
        // The path, so an obligation source's `exclude:` binds this surface as
        // well as the rollup (quire-rs FR-053-AC-14). Without it a criterion in
        // an excluded fixture states no obligation in `coverage --json` and
        // states one here — and this payload is what a generator reads.
        let relative = input.scope_relative(&scope);
        let records =
            quire_rs::classify_document_criteria(&registry, archetype, &text, relative.as_deref());
        census.add(&records);
        documents.push(json!({
            "document": label,
            "archetype": archetype_name,
            "criteria": records.iter().map(record_json).collect::<Vec<_>>(),
        }));
    }

    if args.json {
        let payload = json!({ "documents": documents });
        let rendered = if ctx.pretty {
            serde_json::to_string_pretty(&payload)?
        } else {
            serde_json::to_string(&payload)?
        };
        println!("{rendered}");
    } else {
        census.emit(ctx);
    }

    if failures > 0 {
        bail!("{failures} document(s) could not be resolved to an archetype");
    }
    Ok(())
}

/// Project one record to JSON.
///
/// The field set is written out rather than derived so this surface is a
/// deliberate contract: adding a field to `AcClassification` should be a
/// decision about what consumers see, not a side effect. `as_str()` supplies
/// every enum label, so the strings here cannot drift from the engine's.
fn record_json(r: &AcClassification) -> Value {
    json!({
        "row_id": r.row_id,
        "statement": r.statement,
        "line": r.line,
        "shape": r.shape.as_str(),
        "property": r.property.as_str(),
        "extractable": r.extractable,
        "extraction": r.extraction.as_str(),
        "domain": span_json(r.domain.as_ref()),
        "precondition": span_json(r.precondition.as_ref()),
        "oracle": span_json(r.oracle.as_ref()),
        "signals": r.signals,
        // FR-053. `null` for a module declaring no `obligations:` source, so a
        // corpus that has not adopted them sees the key with a null rather than
        // a shape change. The nested object carries no `id`, `statement` or
        // `document`: `row_id` and `statement` are right here, and the document
        // is the enclosing object.
        "obligation": obligation_json(r.obligation.as_ref()),
    })
}

fn obligation_json(obligation: Option<&quire_rs::obligation::CriterionObligation>) -> Value {
    match obligation {
        None => Value::Null,
        Some(o) => {
            let mut out = json!({
                "source": o.source,
                "statement_hash": o.statement_hash,
                "method": o.method,
                "criticality": o.criticality,
            });
            // Parameters are omitted rather than emitted empty: a declared key
            // with no cell is absent, and an empty map would read as "declared
            // and empty" (FR-053-AC-6).
            if !o.parameters.is_empty() {
                out["parameters"] = json!(o.parameters);
            }
            out
        }
    }
}

/// A span carries both byte offsets and its own text: offsets so a consumer
/// can assert the decomposition partitions the statement, text because handing
/// a UTF-16 consumer raw UTF-8 byte offsets is a defect generator.
fn span_json(span: Option<&PropertySpan>) -> Value {
    match span {
        None => Value::Null,
        Some(s) => json!({ "start": s.start, "end": s.end, "text": s.text }),
    }
}

/// The human census. A count is data, not a verdict.
#[derive(Default)]
struct Census {
    criteria: usize,
    extractable: usize,
    candidate: usize,
    by_property: std::collections::BTreeMap<&'static str, usize>,
}

impl Census {
    fn add(&mut self, records: &[AcClassification]) {
        for r in records {
            self.criteria += 1;
            *self.by_property.entry(r.property.as_str()).or_default() += 1;
            match r.extraction {
                Extraction::Extractable => self.extractable += 1,
                Extraction::Candidate => self.candidate += 1,
                Extraction::NotExtractable => {}
            }
        }
    }

    fn emit(&self, ctx: &Ctx) {
        let pct = (self.extractable * 100)
            .checked_div(self.criteria)
            .unwrap_or(0);
        io::emit_diagnostic(
            ctx.diagnostics,
            "Properties",
            &format!(
                "{}/{} criteria extractable ({pct}%), {} candidate",
                self.extractable, self.criteria, self.candidate
            ),
        );
        if self.criteria > 0 {
            let histogram = self
                .by_property
                .iter()
                .map(|(shape, n)| format!("{shape}={n}"))
                .collect::<Vec<_>>()
                .join(" ");
            io::emit_diagnostic(ctx.diagnostics, "PropertyShapes", &histogram);
        }
    }
}

/// Same module resolution as `validate` and `coverage`: an explicit
/// `--module`, else a `manifest.yaml` at the scope root, else discovery.
fn load_registry(ctx: &Ctx, args: &Args, scope: &Path) -> anyhow::Result<Registry> {
    if let Some(raw) = &args.module {
        let module = safety::validate_module_path(raw)
            .with_context(|| format!("validating --module '{raw}'"))?;
        return super::load_module_registry(ctx, &module);
    }
    if scope.join("manifest.yaml").is_file() {
        return super::load_module_registry(ctx, scope);
    }
    let _: PathBuf = scope.to_path_buf();
    let registry = Registry::from_env().context("loading modules")?;
    io::emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    Ok(registry)
}
