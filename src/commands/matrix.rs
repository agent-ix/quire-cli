//! `quire matrix` — renders the engine's computed coverage matrix
//! (quire-rs FR-050-AC-47..51, `CoverageReport.coverage_matrix`).
//!
//! Requirement → criterion → binder symbols → computed status, derived from the
//! corpus and its trace tags on every run. The engine owns the computation;
//! this command resolves the same scope and module set `coverage` does, calls
//! the same shared computation, and renders one field of the result.
//!
//! The rendering is a view, never an artifact: it goes to stdout and nowhere
//! else. There is no output-path flag and no write anywhere under the scope.

use clap::Parser;
use quire_rs::coverage::{CoverageMatrixBinder, CoverageMatrixRequirement, CoverageMatrixStatus};
use serde::Serialize;

use crate::commands::coverage::{compute_report, tsv_cell, Target};
use crate::commands::Ctx;
use quire_cli::io;

#[derive(Debug, Parser)]
pub struct Args {
    #[command(flatten)]
    pub target: Target,

    /// Output form: a `markdown` table per requirement (default), the engine's
    /// `coverage_matrix` value as `json`, or one `tsv` record per criterion.
    /// Every form goes to stdout.
    #[arg(long, value_enum, value_name = "FORMAT", default_value = "markdown")]
    pub format: Format,

    /// Exit 1 unless there is at least one criterion and every criterion
    /// computes `tagged` or `method-without-symbol`.
    #[arg(long)]
    pub strict: bool,
}

/// How the matrix leaves the process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// One `## <document>` table per requirement.
    Markdown,
    /// `{"coverage_matrix": …, "engine": {…}}`, the engine's value verbatim.
    Json,
    /// One tab-separated record per criterion.
    Tsv,
}

/// The single line markdown renders when there are zero criteria.
const EMPTY_MARKDOWN: &str = "No obligations matched this scope.";

/// The markdown statement budget, in Unicode scalar values.
const STATEMENT_BUDGET: usize = 80;
/// Appended to a statement cut to fit [`STATEMENT_BUDGET`].
const TRUNCATION_MARKER: &str = "...";

/// The JSON payload before provenance is attached. The key follows the
/// engine's own rule for `CoverageReport.coverage_matrix` — absent when there
/// is no population — so the field reads here exactly as it does in
/// `coverage --json`.
#[derive(Serialize)]
struct Payload {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    coverage_matrix: Vec<CoverageMatrixRequirement>,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let (scope, registry) = args.target.load(ctx)?;
    let report = compute_report(ctx, &scope, &registry)?;
    // FR-026-AC-15 / PLAT-1150: selecting one view must not discard the
    // engine's explanation of tags it refused or could only partly read.
    for diagnostic in &report.diagnostics {
        let locus = match (diagnostic.path.as_deref(), diagnostic.line) {
            (Some(path), Some(line)) => format!("{path}:{line}"),
            (Some(path), None) => path.to_string(),
            (None, _) => "scope".to_string(),
        };
        io::emit_warning_with_fields(
            ctx.diagnostics,
            &format!("[{}] {} ({locus})", diagnostic.reason, diagnostic.message),
            &io::DiagnosticFields {
                reason: Some(&diagnostic.reason),
                path: diagnostic.path.as_deref(),
                line: diagnostic.line,
                declaration: Some(&diagnostic.declaration),
                ..Default::default()
            },
        );
    }
    for tag in &report.unmatched_tags {
        io::emit_warning_with_fields(
            ctx.diagnostics,
            &format!(
                "{}:{}: `{}` on `{}` was not read by the declared trace grammar; \
                 use the module's declared tag form (comma-separated IDs for Trace: lists)",
                tag.path, tag.line, tag.trace_id, tag.symbol,
            ),
            &io::DiagnosticFields {
                reason: Some("unmatched-trace-tag"),
                path: Some(&tag.path),
                line: Some(tag.line),
                subject: Some(&tag.trace_id),
                ..Default::default()
            },
        );
    }
    let matrix = report.coverage_matrix;
    let verdict = strict_verdict(&matrix);

    match args.format {
        Format::Markdown => primary!("{}", render_markdown(&matrix)),
        Format::Tsv => primary!("{}", render_tsv(&matrix)),
        Format::Json => primary_line!(
            "{}",
            ix_cli_kit::json::encode(
                &quire_cli::engine::attach(Payload {
                    coverage_matrix: matrix
                }),
                ctx.pretty
            )?
        ),
    }

    if args.strict {
        match verdict {
            Verdict::Pass => {}
            Verdict::Empty => partial_report!("no obligations matched this scope, so nothing was measured (--strict)"),
            Verdict::Gaps {
                untagged,
                ignored_only,
            } => partial_report!(
                "{untagged} untagged criterion(s) and {ignored_only} criterion(s) tagged only by ignored tests (--strict)"
            ),
        }
    }
    Ok(())
}

/// What `--strict` concludes from the matrix alone.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// At least one criterion, every one `tagged` or `method-without-symbol`.
    Pass,
    /// Zero criteria: a report that measured nothing.
    Empty,
    /// At least one criterion is not backed by a running test.
    Gaps {
        untagged: usize,
        ignored_only: usize,
    },
}

fn strict_verdict(matrix: &[CoverageMatrixRequirement]) -> Verdict {
    let mut total = 0usize;
    let (mut untagged, mut ignored_only) = (0usize, 0usize);
    for criterion in matrix.iter().flat_map(|r| &r.criteria) {
        total += 1;
        match criterion.status {
            CoverageMatrixStatus::Tagged | CoverageMatrixStatus::MethodWithoutSymbol => {}
            CoverageMatrixStatus::Untagged => untagged += 1,
            CoverageMatrixStatus::TaggedByIgnoredTest => ignored_only += 1,
        }
    }
    if total == 0 {
        Verdict::Empty
    } else if untagged + ignored_only > 0 {
        Verdict::Gaps {
            untagged,
            ignored_only,
        }
    } else {
        Verdict::Pass
    }
}

/// The status as the engine spells it on the wire.
fn status_token(status: CoverageMatrixStatus) -> &'static str {
    match status {
        CoverageMatrixStatus::Tagged => "tagged",
        CoverageMatrixStatus::Untagged => "untagged",
        CoverageMatrixStatus::TaggedByIgnoredTest => "tagged-by-ignored-test",
        CoverageMatrixStatus::MethodWithoutSymbol => "method-without-symbol",
    }
}

/// One binder as `path:line:column`, marked when the engine marks it ignored.
fn binder_label(binder: &CoverageMatrixBinder) -> String {
    let marker = if binder.ignored { " (ignored)" } else { "" };
    format!("{}:{}:{}{marker}", binder.path, binder.line, binder.column)
}

fn binder_list(binders: &[CoverageMatrixBinder], separator: &str) -> String {
    binders
        .iter()
        .map(binder_label)
        .collect::<Vec<_>>()
        .join(separator)
}

/// A statement cut to the markdown budget: whole at or under
/// [`STATEMENT_BUDGET`] scalar values, otherwise the first 77 plus `...`.
fn truncate_statement(statement: &str) -> String {
    if statement.chars().count() <= STATEMENT_BUDGET {
        return statement.to_string();
    }
    let keep = STATEMENT_BUDGET - TRUNCATION_MARKER.chars().count();
    let mut cut: String = statement.chars().take(keep).collect();
    cut.push_str(TRUNCATION_MARKER);
    cut
}

/// A markdown table cell: structural characters become spaces, then every `|`
/// is escaped so it cannot open a new column.
fn markdown_cell(text: &str) -> String {
    tsv_cell(text).replace('|', "\\|")
}

/// The statement cell. Truncation runs on the scrubbed text before escaping,
/// so an escape is never cut in half and never spends the budget.
fn statement_cell(statement: &str) -> String {
    truncate_statement(&tsv_cell(statement)).replace('|', "\\|")
}

fn render_markdown(matrix: &[CoverageMatrixRequirement]) -> String {
    if matrix.iter().all(|r| r.criteria.is_empty()) {
        return format!("{EMPTY_MARKDOWN}\n");
    }
    let tables: Vec<String> = matrix
        .iter()
        .map(|requirement| {
            let mut table = format!(
                "## {}\n\n| Criterion | Statement | Binders | Status |\n|---|---|---|---|\n",
                requirement.document
            );
            for c in &requirement.criteria {
                let binders = if c.binders.is_empty() {
                    "(none)".to_string()
                } else {
                    binder_list(&c.binders, ", ")
                };
                table.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    markdown_cell(&c.id),
                    statement_cell(&c.statement),
                    markdown_cell(&binders),
                    status_token(c.status)
                ));
            }
            table
        })
        .collect();
    tables.join("\n")
}

fn render_tsv(matrix: &[CoverageMatrixRequirement]) -> String {
    let mut out = String::from("document\tcriterion\tstatus\tbinders\tstatement\n");
    for requirement in matrix {
        for c in &requirement.criteria {
            let cells = [
                tsv_cell(&requirement.document),
                tsv_cell(&c.id),
                status_token(c.status).to_string(),
                tsv_cell(&binder_list(&c.binders, ",")),
                tsv_cell(&c.statement),
            ];
            out.push_str(&cells.join("\t"));
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_rs::coverage::CoverageMatrixCriterion;

    fn criterion(id: &str, status: CoverageMatrixStatus) -> CoverageMatrixCriterion {
        CoverageMatrixCriterion {
            id: id.to_string(),
            statement: "s".to_string(),
            method: None,
            binders: Vec::new(),
            status,
        }
    }

    fn requirement(criteria: Vec<CoverageMatrixCriterion>) -> CoverageMatrixRequirement {
        CoverageMatrixRequirement {
            document: "spec/FR-001.md".to_string(),
            criteria,
        }
    }

    // The token table and the engine's serde spelling are one fact; this binds
    // them so a renamed upstream variant cannot render under a stale name.
    #[test]
    fn status_tokens_match_the_engine_wire_spelling() {
        for status in [
            CoverageMatrixStatus::Tagged,
            CoverageMatrixStatus::Untagged,
            CoverageMatrixStatus::TaggedByIgnoredTest,
            CoverageMatrixStatus::MethodWithoutSymbol,
        ] {
            assert_eq!(
                serde_json::to_value(status).expect("serializes"),
                serde_json::Value::String(status_token(status).to_string())
            );
        }
    }

    #[test]
    fn truncation_counts_scalar_values_and_runs_before_escaping() {
        let exact: String = "é".repeat(80);
        assert_eq!(truncate_statement(&exact), exact);
        let long: String = "é".repeat(81);
        let cut = truncate_statement(&long);
        assert_eq!(cut.chars().count(), 80);
        assert_eq!(cut, format!("{}...", "é".repeat(77)));

        // A pipe at scalar 77 survives the cut whole and is escaped after it.
        let piped = format!("{}|{}", "a".repeat(76), "b".repeat(10));
        assert_eq!(statement_cell(&piped), format!("{}\\|...", "a".repeat(76)));
    }

    #[test]
    fn strict_verdict_reads_only_the_computed_status() {
        assert_eq!(strict_verdict(&[]), Verdict::Empty);
        assert_eq!(strict_verdict(&[requirement(Vec::new())]), Verdict::Empty);
        assert_eq!(
            strict_verdict(&[requirement(vec![
                criterion("A", CoverageMatrixStatus::Tagged),
                criterion("B", CoverageMatrixStatus::MethodWithoutSymbol),
            ])]),
            Verdict::Pass
        );
        assert_eq!(
            strict_verdict(&[requirement(vec![
                criterion("A", CoverageMatrixStatus::Untagged),
                criterion("B", CoverageMatrixStatus::TaggedByIgnoredTest),
                criterion("C", CoverageMatrixStatus::Tagged),
            ])]),
            Verdict::Gaps {
                untagged: 1,
                ignored_only: 1
            }
        );
    }
}
