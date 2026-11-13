//! `quire matrix` — renders the engine's computed coverage matrix with its
//! design column (quire-rs FR-050-AC-47..51 and FR-092, `design_matrix`).
//!
//! Requirement → design elements → criterion → binder symbols → computed
//! status, derived from the corpus and its trace tags on every run. The engine
//! owns the computation; this command resolves the same scope and module set
//! `coverage` does, computes each member's `coverage_matrix` through the same
//! shared computation, hands them to `design_matrix` and renders the entries.
//!
//! The rendering is a view, never an artifact: it goes to stdout and nowhere
//! else. There is no output-path flag and no write anywhere under the scope.

use anyhow::Context;
use clap::Parser;
use quire_cli::safety;
use quire_rs::coverage::{CoverageMatrixBinder, CoverageMatrixStatus};
use quire_rs::{design_matrix, CoverageReport, DesignMatrixEntry, RelationReportMember};
use serde::Serialize;

use crate::commands::corpus_input::{parse_repo, Corpus, RepoArg};
use crate::commands::coverage::{compute_report_over, load_registry_for, tsv_cell, Target};
use crate::commands::matrix_design as design;
use crate::commands::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    #[command(flatten)]
    pub target: Target,

    /// Load several repositories as one corpus, each as `<ORG/REPO>=<DIR>`
    /// (repeatable), so design elements held in a sibling repository join.
    /// Conflicts with `--scope`. Module discovery without `--module` starts at
    /// the first `<DIR>`.
    #[arg(long, value_name = "ORG/REPO=DIR", conflicts_with = "scope", value_parser = parse_repo)]
    pub repo: Vec<RepoArg>,

    /// Output form: a `markdown` table per requirement (default), plain-text
    /// `human` blocks, the engine's matrix value as `json`, or one `tsv`
    /// record per criterion. Every form goes to stdout.
    #[arg(long, value_enum, value_name = "FORMAT", default_value = "markdown")]
    pub format: Format,

    /// Exit 1 unless there is at least one criterion, every criterion
    /// computes `qualified-record`, `tagged`, `retired`, or `method-without-symbol`,
    /// and no live requirement has a design relation left uncovered.
    #[arg(long)]
    pub strict: bool,
}

/// How the matrix leaves the process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// One `## <document>` table per requirement.
    Markdown,
    /// One plain-text block per requirement.
    Human,
    /// `{"coverage_matrix": …, "engine": {…}}`, the engine's value verbatim.
    Json,
    /// One tab-separated record per criterion.
    Tsv,
}

/// The single line markdown and human render when there are zero criteria.
const EMPTY_MARKDOWN: &str = "No obligations matched this scope.";

/// The markdown statement budget, in Unicode scalar values.
const STATEMENT_BUDGET: usize = 80;
/// Appended to a statement cut to fit [`STATEMENT_BUDGET`].
const TRUNCATION_MARKER: &str = "...";

/// The reporting label for a requirement whose engine entry has no
/// `criticality` key (FR-028). Rendered formats only; JSON passes through.
const UNASSIGNED: &str = "unassigned";

/// What the renderers show beyond FR-026's columns.
#[derive(Debug, Clone, Copy)]
struct View {
    /// A loaded module declares `design_relations`: the design column exists.
    design: bool,
    /// `--repo` was given: the repo label and the TSV `repo` column exist.
    repo: bool,
}

/// The JSON payload before provenance is attached. The key follows the
/// engine's own rule — absent when there is no population — so the field
/// reads here exactly as it does in `coverage --json`.
#[derive(Serialize)]
struct Payload {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    coverage_matrix: Vec<DesignMatrixEntry>,
}

/// The corpus the flags name: the `--repo` members in flag order, or the one
/// `--scope` repository with no identity.
fn resolve_corpus(args: &Args) -> anyhow::Result<Corpus> {
    if args.repo.is_empty() {
        let root = safety::validate_dir_path("--scope", &args.target.scope)
            .with_context(|| format!("validating --scope '{}'", args.target.scope))?;
        Corpus::from_scope(&root)
    } else {
        Corpus::from_repos(&args.repo)
    }
}

/// Emit the engine's own explanation of tags it refused or could only partly
/// read (FR-026-AC-16 / PLAT-1150), so selecting one view never discards it.
fn emit_report_diagnostics(ctx: &Ctx, report: &CoverageReport) {
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
}

/// Compute the design matrix over every member (FR-092): each member's
/// `coverage_matrix` through the shared computation, then the one join.
/// Each member's documents are loaded once, by the corpus, and reused for its
/// coverage computation.
fn compute_matrix(
    ctx: &Ctx,
    corpus: &Corpus,
    registry: &quire_rs::Registry,
) -> anyhow::Result<Vec<DesignMatrixEntry>> {
    let mut reports = Vec::with_capacity(corpus.members.len());
    for member in &corpus.members {
        let report = compute_report_over(ctx, &member.root, registry, &Corpus::member_spec(member))
            .with_context(|| format!("member at {}", member.root.display()))?;
        emit_report_diagnostics(ctx, &report);
        reports.push(report);
    }
    let inputs: Vec<RelationReportMember<'_>> = corpus
        .members
        .iter()
        .zip(&reports)
        .map(|(member, report)| RelationReportMember {
            root: &member.root,
            matrix: &report.coverage_matrix,
        })
        .collect();
    design_matrix(&corpus.spec, registry, &inputs).map_err(|e| invalid_error!("{e}"))
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let corpus = resolve_corpus(&args)?;
    let registry = load_registry_for(ctx, &args.target.module, &corpus.members[0].root)?;
    let matrix = compute_matrix(ctx, &corpus, &registry)?;
    let view = View {
        design: registry
            .traceability()
            .is_some_and(|model| !model.design_relations.is_empty()),
        repo: !args.repo.is_empty(),
    };
    let verdict = strict_verdict(&matrix);
    let gaps = design::gate_failures(&matrix);

    match args.format {
        Format::Markdown => primary!("{}", render_markdown(&matrix, view)),
        Format::Human => primary!("{}", render_human(&matrix, view)),
        Format::Tsv => primary!("{}", render_tsv(&matrix, view)),
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
        for gap in &gaps {
            ix_cli_kit::streams::Record::error("DesignGap", gap).emit(ctx.diagnostics);
        }
        let mut reasons = Vec::new();
        match verdict {
            Verdict::Pass => {}
            Verdict::Empty => reasons.push(
                "no obligations matched this scope, so nothing was measured".to_string(),
            ),
            Verdict::Gaps {
                untagged,
                ignored_only,
            } => reasons.push(format!(
                "{untagged} untagged criterion(s) and {ignored_only} criterion(s) tagged only by ignored tests"
            )),
        }
        if !gaps.is_empty() {
            reasons.push(format!("{} design gap(s)", gaps.len()));
        }
        if !reasons.is_empty() {
            partial_report!("{} (--strict)", reasons.join("; "));
        }
    }
    Ok(())
}

/// What `--strict` concludes from the criteria alone; the design gate is
/// counted beside it.
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

fn strict_verdict(matrix: &[DesignMatrixEntry]) -> Verdict {
    let mut total = 0usize;
    let (mut untagged, mut ignored_only) = (0usize, 0usize);
    for criterion in matrix.iter().flat_map(|r| &r.criteria) {
        total += 1;
        match criterion.criterion.status {
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
fn criticality_text(entry: &DesignMatrixEntry) -> String {
    match &entry.criticality {
        None => UNASSIGNED.to_string(),
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
    }
}

/// The entries a rendered form shows: those with criteria, and those a design
/// relation obliges even when no criterion exists (FR-043 zero population).
fn shown(entries: &[DesignMatrixEntry]) -> Vec<&DesignMatrixEntry> {
    entries
        .iter()
        .filter(|e| !e.criteria.is_empty() || e.design.is_some())
        .collect()
}

fn binders_or_none(binders: &[CoverageMatrixBinder]) -> String {
    if binders.is_empty() {
        "(none)".to_string()
    } else {
        binder_list(binders, ", ")
    }
}

fn render_markdown(matrix: &[DesignMatrixEntry], view: View) -> String {
    let entries = shown(matrix);
    if entries.is_empty() {
        return format!("{EMPTY_MARKDOWN}\n");
    }
    let tables: Vec<String> = entries
        .into_iter()
        .map(|entry| {
            let mut table = format!("## {}\n\n", entry.document);
            let lines = design::header_lines(entry);
            if !lines.is_empty() {
                table.push_str(&lines.join("\n"));
                table.push_str("\n\n");
            }
            if entry.criteria.is_empty() {
                // Nothing to tabulate: the heading and its gap lines stand.
                table.truncate(table.trim_end().len());
                table.push('\n');
                return table;
            }
            if view.design {
                table.push_str(
                    "| Criterion | Statement | Design | Binders | Status | Requirement Criticality |\n|---|---|---|---|---|---|\n",
                );
            } else {
                table.push_str(
                    "| Criterion | Statement | Binders | Status | Requirement Criticality |\n|---|---|---|---|---|\n",
                );
            }
            let criticality = markdown_cell(&criticality_text(entry));
            for c in &entry.criteria {
                let binders = markdown_cell(&binders_or_none(&c.criterion.binders));
                let design_cell = if view.design {
                    format!("{} | ", markdown_cell(&design::criterion_cell(entry, c)))
                } else {
                    String::new()
                };
                table.push_str(&format!(
                    "| {} | {} | {}{} | {} | {} |\n",
                    markdown_cell(&c.criterion.id),
                    statement_cell(&c.criterion.statement),
                    design_cell,
                    binders,
                    status_token(c.criterion.status),
                    criticality
                ));
            }
            table
        })
        .collect();
    tables.join("\n")
}

fn render_human(matrix: &[DesignMatrixEntry], view: View) -> String {
    let entries = shown(matrix);
    if entries.is_empty() {
        return format!("{EMPTY_MARKDOWN}\n");
    }
    let blocks: Vec<String> = entries
        .into_iter()
        .map(|entry| {
            let mut block = format!("{}\n", entry.document);
            for line in design::header_lines(entry) {
                block.push_str(&format!("{line}\n"));
            }
            if view.design {
                block.push_str(&format!("  design: {}\n", design::entry_cell(entry)));
            }
            for c in &entry.criteria {
                let design_cell = if view.design {
                    format!("design: {}  ", design::criterion_cell(entry, c))
                } else {
                    String::new()
                };
                block.push_str(&format!(
                    "  {}  {}  {}tests: {}\n",
                    tsv_cell(&c.criterion.id),
                    status_token(c.criterion.status),
                    design_cell,
                    tsv_cell(&binders_or_none(&c.criterion.binders)),
                ));
            }
            block
        })
        .collect();
    blocks.join("\n")
}

fn render_tsv(matrix: &[DesignMatrixEntry], view: View) -> String {
    let mut header: Vec<&str> = Vec::new();
    if view.repo {
        header.push("repo");
    }
    header.extend(["document", "criterion"]);
    if view.design {
        header.extend(["design", "design_state"]);
    }
    header.extend(["status", "binders", "statement", "requirement_criticality"]);
    let mut out = header.join("\t");
    out.push('\n');
    for entry in matrix {
        let mut records: Vec<Vec<String>> = entry
            .criteria
            .iter()
            .map(|c| {
                let mut cells = vec![tsv_cell(&entry.document), tsv_cell(&c.criterion.id)];
                if view.design {
                    cells.push(tsv_cell(&design::criterion_field(entry, c)));
                    cells.push(design::state(entry).to_string());
                }
                cells.extend([
                    status_token(c.criterion.status).to_string(),
                    tsv_cell(&binder_list(&c.criterion.binders, ",")),
                    tsv_cell(&c.criterion.statement),
                    tsv_cell(&criticality_text(entry)),
                ]);
                cells
            })
            .collect();
        if entry.criteria.is_empty() && entry.design.is_some() {
            records.push(vec![
                tsv_cell(&entry.document),
                String::new(),
                tsv_cell(&design::entry_field(entry)),
                design::state(entry).to_string(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ]);
        }
        for mut cells in records {
            if view.repo {
                cells.insert(0, entry.repo.as_deref().map(tsv_cell).unwrap_or_default());
            }
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
    use quire_rs::DesignMatrixCriterion;

    const PLAIN: View = View {
        design: false,
        repo: false,
    };

    fn criterion(id: &str, status: CoverageMatrixStatus) -> DesignMatrixCriterion {
        DesignMatrixCriterion {
            criterion: CoverageMatrixCriterion {
                id: id.to_string(),
                statement: "s".to_string(),
                method: None,
                binders: Vec::new(),
                record_binders: None,
                identity: None,
                status,
            },
            design_elements: None,
        }
    }

    fn requirement(criteria: Vec<DesignMatrixCriterion>) -> DesignMatrixEntry {
        DesignMatrixEntry {
            document: "spec/FR-001.md".to_string(),
            repo: None,
            id: None,
            criticality: None,
            carried: None,
            criteria,
            design: None,
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
            render_markdown(&matrix, PLAIN)
                .contains("| FR-001-AC-1 | s | (none) | qualified-record | unassigned |"),
            "qualified backing must not manufacture a source binder"
        );
        assert_eq!(
            render_tsv(&matrix, PLAIN),
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
