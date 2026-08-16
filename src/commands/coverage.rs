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
    /// Repository root. Two roots derive from it and are never interchanged
    /// (quire-rs FR-050, CR-045): documents are read from `<scope>/spec` and
    /// trace tags from the source tree at `<scope>`, excluding `spec/`.
    /// Defaults to the current directory.
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

    // Two roots, one scope (CR-045): the document walk is bounded to
    // `<scope>/spec`; the code walk covers `<scope>` minus the document
    // root. `compute_coverage` still relativizes against `<scope>`, so
    // report paths keep their `spec/` prefix and output is byte-identical
    // for a compliant repo.
    // Path-safety on the derived document root, the same guard
    // `validate --okf` has always applied to its bundle root. `coverage`
    // skipped it, so the two commands disagreed about what a `..` in the
    // resolved root meant (agent-ix/quire-rs#113).
    let spec_root = super::spec_root_of(&scope)?;
    let spec_root = safety::validate_dir_path("document root", &spec_root.display().to_string())
        .with_context(|| format!("validating document root '{}'", spec_root.display()))?;
    let spec = Spec::from_path(&spec_root);
    // The exclusion is derived from the same constant the root is, rather
    // than a second literal `"spec"` that can drift from it
    // (agent-ix/quire-rs#113). The engine compares by canonicalized
    // identity, so a case-insensitive filesystem or a symlinked root still
    // excludes what the walk actually reads (quire-rs CR-056).
    let extraction =
        quire_rs::symbols::extract_tree_excluding(&scope, &[Path::new(super::DOCUMENT_ROOT_DIR)]);
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

    if args.strict {
        // FR-050-AC-14 (CR-035): a model that matched nothing is the one state
        // where the two lists below are empty for the *wrong* reason. Checked
        // first, and reported as itself — a gate told "0 unbacked rows" learns
        // the opposite of the truth.
        if report.totals.total == 0 {
            bail!(
                "the declared traceability model matched no rows in this scope, \
                 so nothing was reconciled (--strict); check that the model's \
                 trace targets name documents and sections this repo actually has"
            );
        }
        if !report.unbacked_rows.is_empty() || !report.status_lies.is_empty() {
            bail!(
                "{} unbacked row(s) and {} contradicted status(es) (--strict)",
                report.unbacked_rows.len(),
                report.status_lies.len()
            );
        }
    }
    Ok(())
}

/// A backed/total pair as a percentage, or `None` when nothing was counted.
///
/// FR-050-AC-14 (CR-035): the empty denominator is not 100%. It used to be —
/// `checked_div(total).unwrap_or(100)` — so a scope where the model matched
/// nothing printed `0/0 rows backed (100%)` and a `--strict` gate over it
/// passed. "Found nothing" and "all covered" are opposite states and must not
/// render alike.
fn percent(backed: usize, total: usize) -> Option<usize> {
    (backed * 100).checked_div(total)
}

/// `44%`, or a marker naming the empty denominator for what it is.
fn percent_label(backed: usize, total: usize) -> String {
    match percent(backed, total) {
        Some(pct) => format!("{pct}%"),
        None => "no rows matched".to_string(),
    }
}

fn emit_human(ctx: &Ctx, report: &quire_rs::CoverageReport) {
    let t = &report.totals;
    io::emit_diagnostic(
        ctx.diagnostics,
        "Coverage",
        &format!(
            "{}/{} rows backed ({})",
            t.backed,
            t.total,
            percent_label(t.backed, t.total)
        ),
    );
    for g in &report.groups {
        io::emit_diagnostic(
            ctx.diagnostics,
            "CoverageGroup",
            &format!(
                "{}: {}/{} ({})",
                g.document,
                g.backed,
                g.total,
                percent_label(g.backed, g.total)
            ),
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
