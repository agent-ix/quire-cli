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
    /// computes `qualified-record`, `tagged`, `retired`, or `method-without-symbol`.
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

/// The reporting label for a requirement whose engine entry has no
/// `criticality` key (FR-028). Rendered formats only; JSON passes through.
const UNASSIGNED: &str = "unassigned";

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
    // FR-026-AC-16 / PLAT-1150: selecting one view must not discard the
    // engine's explanation of tags it refused or could only partly read.
    for diagnostic in &report.diagnostics {
        let locus = match (diagnostic.path.as_deref(), diagnostic.line) {
            (Some(path), Some(line)) => format!("{path}:{line}"),
            (Some(path), None) => path.to_string(),
            (None, _) => "scope".to_string(),
        };
        let message = format!("[{}] {} ({locus})", diagnostic.reason, diagnostic.message);
        ix_cli_kit::streams::Record::warning("MatrixDiagnostic", &message)
            .with_fields(ix_cli_kit::streams::DiagnosticFields {
                reason: Some(&diagnostic.reason),
                path: diagnostic.path.as_deref(),
                line: diagnostic.line,
                change_target: Some(&diagnostic.declaration),
                ..Default::default()
            })
            .emit(ctx.diagnostics);
    }
    for tag in &report.unmatched_tags {
        let message = format!(
            "{}:{}: `{}` on `{}` was not read by the declared trace grammar; \
             write it in the module's declared marker form",
            tag.path, tag.line, tag.trace_id, tag.symbol,
        );
        ix_cli_kit::streams::Record::warning("UnmatchedTraceTag", &message)
            .with_fields(ix_cli_kit::streams::DiagnosticFields {
                reason: Some("unmatched-trace-tag"),
                path: Some(&tag.path),
                line: Some(tag.line),
                subject: Some(&tag.trace_id),
                ..Default::default()
            })
            .emit(ctx.diagnostics);
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
    /// At least one criterion, every one `qualified-record`, `tagged`, `retired`, or `method-without-symbol`.
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
            CoverageMatrixStatus::QualifiedRecord
            | CoverageMatrixStatus::Tagged
            | CoverageMatrixStatus::Retired
            | CoverageMatrixStatus::MethodWithoutSymbol => {}
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
        CoverageMatrixStatus::QualifiedRecord => "qualified-record",
        CoverageMatrixStatus::Retired => "retired",
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

/// The requirement's criticality cell text: `unassigned` when the key is
/// absent, a string verbatim, any other JSON value as its compact text.
fn criticality_text(requirement: &CoverageMatrixRequirement) -> String {
    match &requirement.criticality {
        None => UNASSIGNED.to_string(),
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
    }
}

fn render_markdown(matrix: &[CoverageMatrixRequirement]) -> String {
    if matrix.iter().all(|r| r.criteria.is_empty()) {
        return format!("{EMPTY_MARKDOWN}\n");
    }
    let tables: Vec<String> = matrix
        .iter()
        .map(|requirement| {
            let mut table = format!(
                "## {}\n\n| Criterion | Statement | Binders | Status | Requirement Criticality |\n|---|---|---|---|---|\n",
                requirement.document
            );
            let criticality = markdown_cell(&criticality_text(requirement));
            for c in &requirement.criteria {
                let binders = if c.binders.is_empty() {
                    "(none)".to_string()
                } else {
                    binder_list(&c.binders, ", ")
                };
                table.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    markdown_cell(&c.id),
                    statement_cell(&c.statement),
                    markdown_cell(&binders),
                    status_token(c.status),
                    criticality
                ));
            }
            table
        })
        .collect();
    tables.join("\n")
}

fn render_tsv(matrix: &[CoverageMatrixRequirement]) -> String {
    let mut out =
        String::from("document\tcriterion\tstatus\tbinders\tstatement\trequirement_criticality\n");
    for requirement in matrix {
        for c in &requirement.criteria {
            let cells = [
                tsv_cell(&requirement.document),
                tsv_cell(&c.id),
                status_token(c.status).to_string(),
                tsv_cell(&binder_list(&c.binders, ",")),
                tsv_cell(&c.statement),
                tsv_cell(&criticality_text(requirement)),
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
    use ix_trace_rs::trace;
    use quire_rs::coverage::CoverageMatrixCriterion;

    fn criterion(id: &str, status: CoverageMatrixStatus) -> CoverageMatrixCriterion {
        CoverageMatrixCriterion {
            id: id.to_string(),
            statement: "s".to_string(),
            method: None,
            binders: Vec::new(),
            record_binders: None,
            identity: None,
            status,
        }
    }

    fn requirement(criteria: Vec<CoverageMatrixCriterion>) -> CoverageMatrixRequirement {
        CoverageMatrixRequirement {
            document: "spec/FR-001.md".to_string(),
            criticality: None,
            criteria,
        }
    }

    // The token table and the engine's serde spelling are one fact; this binds
    // them so a renamed upstream variant cannot render under a stale name.
    #[test]
    fn status_tokens_match_the_engine_wire_spelling() {
        for status in [
            CoverageMatrixStatus::QualifiedRecord,
            CoverageMatrixStatus::Retired,
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

    #[trace("FR-026-AC-5", "FR-026-AC-7", "FR-026-AC-8", "FR-026-AC-15")]
    #[test]
    fn qualified_record_renders_verbatim_without_source_binders() {
        let matrix = vec![requirement(vec![criterion(
            "FR-001-AC-1",
            CoverageMatrixStatus::QualifiedRecord,
        )])];
        assert!(
            render_markdown(&matrix)
                .contains("| FR-001-AC-1 | s | (none) | qualified-record | unassigned |"),
            "qualified backing must not manufacture a source binder"
        );
        assert_eq!(
            render_tsv(&matrix),
            "document\tcriterion\tstatus\tbinders\tstatement\trequirement_criticality\n\
             spec/FR-001.md\tFR-001-AC-1\tqualified-record\t\ts\tunassigned\n"
        );
        let payload = serde_json::to_value(Payload {
            coverage_matrix: matrix,
        })
        .expect("matrix payload serializes");
        let row = &payload["coverage_matrix"][0]["criteria"][0];
        assert_eq!(row["status"], "qualified-record");
        assert_eq!(row["binders"], serde_json::json!([]));
    }

    #[trace("FR-026-AC-10", "FR-026-AC-15")]
    #[test]
    fn qualified_record_passes_strict_without_hiding_live_gaps() {
        assert_eq!(
            strict_verdict(&[requirement(vec![criterion(
                "FR-001-AC-1",
                CoverageMatrixStatus::QualifiedRecord,
            )])]),
            Verdict::Pass,
            "a nonempty qualified population needs no source binder"
        );
        for (status, expected) in [
            (
                CoverageMatrixStatus::Untagged,
                Verdict::Gaps {
                    untagged: 1,
                    ignored_only: 0,
                },
            ),
            (
                CoverageMatrixStatus::TaggedByIgnoredTest,
                Verdict::Gaps {
                    untagged: 0,
                    ignored_only: 1,
                },
            ),
        ] {
            assert_eq!(
                strict_verdict(&[requirement(vec![
                    criterion("FR-001-AC-1", CoverageMatrixStatus::QualifiedRecord),
                    criterion("FR-001-AC-2", status),
                ])]),
                expected,
                "qualified backing must not exempt another criterion: {status:?}"
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
                criterion("C", CoverageMatrixStatus::Retired),
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
