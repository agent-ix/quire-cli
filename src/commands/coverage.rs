//! `quire coverage` — the declarative AC→TC→code rollup (quire-rs FR-050).
//!
//! The engine owns the whole computation: a module declares a `traceability:`
//! block, the corpus supplies the declared reference rows, and the source tree
//! supplies the trace tags. This command is deliberately thin — it resolves
//! paths, calls `quire_rs::compute_coverage`, and prints. **Verdict policy stays
//! in quoin**: the report says what is unbacked, what a status claims against
//! what the tests show, and what is untracked; it does not decide whether that
//! is acceptable.
//!
//! Output is byte-identical for identical input (FR-050-AC-7), so it is safe to
//! diff between runs or commit as an artifact.

use std::path::Path;

use anyhow::{bail, Context};
use clap::Parser;
use quire_cli::io;
use quire_cli::safety;
use quire_rs::{compute_coverage, Registry, Spec};

use crate::commands::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    /// Repository root to scan: the spec bundle *and* the source tree the
    /// trace tags live in. Defaults to the current directory.
    #[arg(long, default_value = ".")]
    pub scope: String,

    /// Module directory supplying the `traceability:` model. When omitted the
    /// module set is discovered exactly as `quire validate` discovers it.
    #[arg(long)]
    pub module: Option<String>,

    /// Emit the report as JSON on stdout instead of the human summary.
    /// The JSON is the stable interface; the human form may change.
    #[arg(long)]
    pub json: bool,

    /// Exit 1 when any row is unbacked or any status is contradicted. Off by
    /// default: the rollup is a report, and whether a gap blocks is the
    /// consuming workflow's policy, not this command's.
    #[arg(long)]
    pub strict: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let scope = safety::validate_dir_path("--scope", &args.scope)
        .with_context(|| format!("validating --scope '{}'", args.scope))?;
    let registry = load_registry(ctx, &args, &scope)?;

    // FR-050: the model is module data. Without it there is nothing to
    // reconcile against, and guessing would be exactly the agent-grep behaviour
    // this command replaces.
    if registry.traceability().is_none() {
        bail!(
            "no module in scope declares a `traceability:` model, so there is \
             nothing to reconcile; install a module that declares one (e.g. \
             spec-artifacts-process) or pass --module"
        );
    }

    let spec = Spec::from_path(&scope);
    let extraction = quire_rs::symbols::extract_tree(&scope);
    let graph = quire_rs::symbols::trace::bind(
        &extraction,
        registry
            .traceability()
            .expect("traceability model checked above"),
    );

    let report =
        compute_coverage(&spec, &registry, &graph, &scope).map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        println!("{}", report.to_json());
    } else {
        emit_human(ctx, &report);
    }

    if args.strict && (!report.unbacked_rows.is_empty() || !report.status_lies.is_empty()) {
        bail!(
            "{} unbacked row(s) and {} contradicted status(es) (--strict)",
            report.unbacked_rows.len(),
            report.status_lies.len()
        );
    }
    Ok(())
}

fn emit_human(ctx: &Ctx, report: &quire_rs::CoverageReport) {
    let t = &report.totals;
    let pct = (t.backed * 100).checked_div(t.total).unwrap_or(100);
    io::emit_diagnostic(
        ctx.diagnostics,
        "Coverage",
        &format!("{}/{} rows backed ({pct}%)", t.backed, t.total),
    );
    for g in &report.groups {
        let gp = (g.backed * 100).checked_div(g.total).unwrap_or(100);
        io::emit_diagnostic(
            ctx.diagnostics,
            "CoverageGroup",
            &format!("{}: {}/{} ({gp}%)", g.document, g.backed, g.total),
        );
    }
    for r in &report.unbacked_rows {
        io::emit_diagnostic(
            ctx.diagnostics,
            "UnbackedRow",
            &format!("{} ({}) has no backing symbol", r.reference, r.document),
        );
    }
    for l in &report.status_lies {
        io::emit_diagnostic(
            ctx.diagnostics,
            "StatusLie",
            &format!(
                "{} ({}) claims `{}` but is not backed",
                l.reference, l.document, l.status
            ),
        );
    }
    for u in &report.untracked_symbols {
        io::emit_diagnostic(
            ctx.diagnostics,
            "UntrackedSymbol",
            &format!(
                "{} in {} traces to `{}`, which matches no declared row",
                u.symbol, u.path, u.trace_id
            ),
        );
    }
}

/// Same module resolution as `validate`: an explicit `--module`, else a
/// `manifest.yaml` at the scope root, else scoped discovery.
fn load_registry(ctx: &Ctx, args: &Args, scope: &Path) -> anyhow::Result<Registry> {
    if let Some(raw) = &args.module {
        let module = safety::validate_module_path(raw)
            .with_context(|| format!("validating --module '{raw}'"))?;
        return super::load_module_registry(ctx, &module);
    }
    if scope.join("manifest.yaml").is_file() {
        return super::load_module_registry(ctx, scope);
    }
    let registry = Registry::from_env().context("loading modules")?;
    io::emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    Ok(registry)
}
