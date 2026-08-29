//! `quire clauses` — evaluate and compare module-supplied clause sets.
//!
//! The engine owns the model, validation, applicability semantics, and diff.
//! This command resolves safe paths and exact versions, then renders the
//! resulting contract as human text, JSON, or TSV.

use std::collections::BTreeMap;

use anyhow::{bail, Context};
use clap::{Parser, Subcommand, ValueEnum};
use quire_rs::{
    diff_clause_sets, BindingOutcome, Clause, ClauseBindingReport, ClauseForce, ClauseSetDiff,
    Registry,
};

use crate::commands::Ctx;
use quire_cli::{engine, io, safety};

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    command: ClauseCommand,
}

#[derive(Debug, Subcommand)]
enum ClauseCommand {
    /// Evaluate one exact clause-set version against declared context.
    Evaluate(EvaluateArgs),
    /// Compare two exact versions of the same clause set.
    Diff(DiffArgs),
}

#[derive(Debug, Parser)]
struct EvaluateArgs {
    /// Module directory containing manifest.yaml and the referenced set.
    #[arg(long)]
    module: String,
    /// Clause-set authority.
    #[arg(long)]
    authority: String,
    /// Clause-set id.
    #[arg(long = "set")]
    set_id: String,
    /// Exact clause-set version.
    #[arg(long)]
    version: String,
    /// Context dimension as KEY=VALUE. Repeat for multiple dimensions.
    #[arg(long = "context", value_name = "KEY=VALUE")]
    context: Vec<String>,
    /// Output form. Human is the default.
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,
    /// Alias for `--format json`.
    #[arg(long, conflicts_with = "format")]
    json: bool,
}

#[derive(Debug, Parser)]
struct DiffArgs {
    /// Module directory containing manifest.yaml and both referenced sets.
    #[arg(long)]
    module: String,
    /// Clause-set authority.
    #[arg(long)]
    authority: String,
    /// Clause-set id.
    #[arg(long = "set")]
    set_id: String,
    /// Exact earlier clause-set version.
    #[arg(long)]
    before_version: String,
    /// Exact later clause-set version.
    #[arg(long)]
    after_version: String,
    /// Output form. Human is the default.
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,
    /// Alias for `--format json`.
    #[arg(long, conflicts_with = "format")]
    json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Tsv,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    match args.command {
        ClauseCommand::Evaluate(args) => evaluate(ctx, args),
        ClauseCommand::Diff(args) => diff(ctx, args),
    }
}

fn evaluate(ctx: &Ctx, args: EvaluateArgs) -> anyhow::Result<()> {
    let registry = load_registry(ctx, &args.module)?;
    let set = exact_set(&registry, &args.authority, &args.set_id, &args.version)?;
    let report = set.evaluate(&parse_context(&args.context)?);
    match selected_format(args.format, args.json) {
        OutputFormat::Human => io::write_primary_stdout(render_binding_human(&report).as_bytes())?,
        OutputFormat::Json => write_json(ctx, &report)?,
        OutputFormat::Tsv => io::write_primary_stdout(render_binding_tsv(&report).as_bytes())?,
    }
    Ok(())
}

fn diff(ctx: &Ctx, args: DiffArgs) -> anyhow::Result<()> {
    let registry = load_registry(ctx, &args.module)?;
    let before = exact_set(
        &registry,
        &args.authority,
        &args.set_id,
        &args.before_version,
    )?;
    let after = exact_set(
        &registry,
        &args.authority,
        &args.set_id,
        &args.after_version,
    )?;
    let report = diff_clause_sets(before, after).map_err(|error| anyhow::anyhow!(error))?;
    match selected_format(args.format, args.json) {
        OutputFormat::Human => io::write_primary_stdout(render_diff_human(&report).as_bytes())?,
        OutputFormat::Json => write_json(ctx, &report)?,
        OutputFormat::Tsv => io::write_primary_stdout(render_diff_tsv(&report).as_bytes())?,
    }
    Ok(())
}

fn load_registry(ctx: &Ctx, raw: &str) -> anyhow::Result<Registry> {
    let module = safety::validate_module_path(raw)
        .with_context(|| format!("validating --module {raw:?}"))?;
    let registry = Registry::load_module_strict(&module).context("loading clause-set module")?;
    io::emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    if let Some(failure) = registry.failures().first() {
        bail!(
            "module load failed: {} ({})",
            failure.reason,
            failure.path.display()
        );
    }
    Ok(registry)
}

fn exact_set<'a>(
    registry: &'a Registry,
    authority: &str,
    id: &str,
    version: &str,
) -> anyhow::Result<&'a quire_rs::ClauseSet> {
    registry.clause_set(authority, id, version).ok_or_else(|| {
        let available = registry
            .clause_sets()
            .map(|set| format!("{}/{}/{}", set.authority, set.id, set.version))
            .collect::<Vec<_>>();
        anyhow::anyhow!(
            "clause set {authority}/{id}/{version} is not loaded; available exact sets: {}",
            if available.is_empty() {
                "none".to_string()
            } else {
                available.join(", ")
            }
        )
    })
}

fn parse_context(entries: &[String]) -> anyhow::Result<BTreeMap<String, String>> {
    let mut context = BTreeMap::new();
    for entry in entries {
        let Some((key, value)) = entry.split_once('=') else {
            bail!("--context {entry:?} must be KEY=VALUE");
        };
        if key.trim().is_empty() || value.trim().is_empty() {
            bail!("--context {entry:?} must have a non-empty key and value");
        }
        let (key, value) = (key.trim(), value.trim());
        if context.insert(key.to_string(), value.to_string()).is_some() {
            bail!("--context declares {key:?} more than once");
        }
    }
    Ok(context)
}

fn selected_format(format: OutputFormat, json: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else {
        format
    }
}

fn write_json<T: serde::Serialize>(ctx: &Ctx, value: &T) -> anyhow::Result<()> {
    let body = io::encode_json(&engine::attach(value), ctx.pretty)?;
    io::write_primary_stdout(format!("{body}\n").as_bytes())?;
    Ok(())
}

fn render_binding_human(report: &ClauseBindingReport) -> String {
    let mut out = format!(
        "Clause set {}/{}/{} ({})\n",
        report.clause_set.authority,
        report.clause_set.id,
        report.clause_set.version,
        report.clause_set_digest
    );
    for clause in &report.clauses {
        out.push_str(&format!(
            "{}\t{}\t{}",
            clause.clause_id,
            force_name(&clause.force),
            outcome_name(&clause.outcome)
        ));
        if !clause.expected_outputs.is_empty() {
            out.push_str(&format!("\texpects {}", clause.expected_outputs.join(",")));
        }
        if !clause.reasons.is_empty() {
            out.push_str(&format!(
                "\t{}",
                clause
                    .reasons
                    .iter()
                    .map(|reason| reason.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        out.push('\n');
    }
    out
}

fn render_binding_tsv(report: &ClauseBindingReport) -> String {
    let mut out = String::from("clause_id\tforce\toutcome\texpected_outputs\treason_codes\n");
    for clause in &report.clauses {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            tsv_cell(&clause.clause_id),
            force_name(&clause.force),
            outcome_name(&clause.outcome),
            tsv_cell(&clause.expected_outputs.join(",")),
            tsv_cell(
                &clause
                    .reasons
                    .iter()
                    .map(|reason| reason.code.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        ));
    }
    out
}

fn render_diff_human(report: &ClauseSetDiff) -> String {
    let mut out = format!(
        "Clause set {}/{}: {} -> {}\nAdded: {}  Removed: {}  Changed: {}\n",
        report.before.authority,
        report.before.id,
        report.before.version,
        report.after.version,
        report.added.len(),
        report.removed.len(),
        report.changed.len()
    );
    for clause in &report.added {
        out.push_str(&format!("added\t{}\n", clause.id));
    }
    for clause in &report.removed {
        out.push_str(&format!("removed\t{}\n", clause.id));
    }
    for clause in &report.changed {
        out.push_str(&format!("changed\t{}\n", clause.clause_id));
    }
    out
}

fn render_diff_tsv(report: &ClauseSetDiff) -> String {
    let mut out = String::from("change\tclause_id\tbefore_force\tafter_force\n");
    for clause in &report.added {
        push_diff_row(&mut out, "added", clause, None, Some(&clause.force));
    }
    for clause in &report.removed {
        push_diff_row(&mut out, "removed", clause, Some(&clause.force), None);
    }
    for clause in &report.changed {
        push_diff_row(
            &mut out,
            "changed",
            &clause.after,
            Some(&clause.before.force),
            Some(&clause.after.force),
        );
    }
    out
}

fn push_diff_row(
    out: &mut String,
    change: &str,
    clause: &Clause,
    before: Option<&ClauseForce>,
    after: Option<&ClauseForce>,
) {
    out.push_str(&format!(
        "{change}\t{}\t{}\t{}\n",
        tsv_cell(&clause.id),
        before.map(force_name).unwrap_or(""),
        after.map(force_name).unwrap_or("")
    ));
}

fn force_name(force: &ClauseForce) -> &'static str {
    match force {
        ClauseForce::Mandatory => "mandatory",
        ClauseForce::Recommended => "recommended",
        ClauseForce::Permitted => "permitted",
    }
}

fn outcome_name(outcome: &BindingOutcome) -> &'static str {
    match outcome {
        BindingOutcome::Binding => "binding",
        BindingOutcome::NotBinding => "not_binding",
        BindingOutcome::Unresolved => "unresolved",
    }
}

fn tsv_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_rejects_duplicates_and_malformed_entries() {
        assert!(parse_context(&["impact=high".into(), "impact=low".into()]).is_err());
        assert!(parse_context(&["impact".into()]).is_err());
        assert!(parse_context(&["=high".into()]).is_err());
        assert_eq!(
            parse_context(&[" impact = high ".into()]).unwrap(),
            BTreeMap::from([("impact".into(), "high".into())])
        );
    }

    #[test]
    fn tsv_cells_keep_one_record_per_line() {
        assert_eq!(tsv_cell("a\tb\nc\rd\\e"), "a\\tb\\nc\\rd\\\\e");
    }
}
