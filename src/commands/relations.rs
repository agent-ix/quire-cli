//! `quire relations` — the two-way relation coverage report (quire-rs FR-084).
//!
//! The engine owns the rows, the states and the verification statuses; this
//! command resolves roots, applies path safety, calls
//! `quire_rs::relation_report` and renders. It holds no archetype, verb or
//! chain name: which rows exist comes from the loaded modules'
//! `traceability.required_relations`.

use std::path::PathBuf;

use clap::Parser;
use quire_cli::safety;
use quire_rs::{
    relation_report, LinkedEntry, RelationReport, RelationReportMember, RelationRow, RowDocument,
    RowState, Verification,
};

use super::corpus_input::{parse_repo, Corpus, RepoArg};
use super::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    /// Repository root, loaded as a corpus of one member with no identity.
    /// Documents are read from `<DIR>/spec`. Defaults to the current directory
    /// when no `--repo` is given; not accepted together with `--repo`.
    #[arg(long, value_name = "DIR", conflicts_with = "repo")]
    pub scope: Option<String>,

    /// A corpus member: `<ORG/REPO>=<DIR>`, repeatable. The members load as one
    /// corpus, so an edge into a sibling repository resolves.
    #[arg(long, value_name = "ORG/REPO=DIR", value_parser = parse_repo)]
    pub repo: Vec<RepoArg>,

    /// Module directory supplying `traceability.required_relations`.
    /// Repeatable; the roots REPLACE ambient discovery (see `quire coverage`).
    #[arg(long, value_name = "PATH")]
    pub module: Vec<String>,

    /// Restrict the report to the named declaration. Repeatable.
    #[arg(long, value_name = "NAME")]
    pub relation: Vec<String>,

    /// Emit the engine's report as JSON on stdout instead of the human form.
    #[arg(long)]
    pub json: bool,

    /// Emit the complete report, then exit 1 when any row is `uncovered`,
    /// `unverified` or `partial`, when no rows were selected
    /// (`empty-population`), or when every row is `not-tracked`
    /// (`nothing-measured`).
    #[arg(long)]
    pub strict: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let corpus = if args.repo.is_empty() {
        let raw = args.scope.as_deref().unwrap_or(".");
        let scope = safety::validate_dir_path("--scope", raw)
            .map_err(|e| anyhow::Error::new(e).context(format!("validating --scope '{raw}'")))?;
        Corpus::from_scope(&scope)?
    } else {
        Corpus::from_repos(&args.repo)?
    };

    let discovery: PathBuf = corpus.members[0].root.clone();
    let registry = super::coverage::load_registry_for(ctx, &args.module, &discovery)?;
    if registry
        .traceability()
        .is_none_or(|model| model.required_relations.is_empty())
    {
        invalid_request!(
            "no loaded module declares `traceability.required_relations`, so there is \
             nothing to report; install a module that declares one or pass --module"
        );
    }

    let mut coverages = Vec::with_capacity(corpus.members.len());
    for member in &corpus.members {
        coverages.push(super::coverage::compute_report(
            ctx,
            &member.root,
            &registry,
        )?);
    }
    let inputs: Vec<RelationReportMember<'_>> = corpus
        .members
        .iter()
        .zip(&coverages)
        .map(|(member, coverage)| RelationReportMember {
            root: &member.root,
            matrix: &coverage.coverage_matrix,
        })
        .collect();
    let report = relation_report(&corpus.spec, &registry, &inputs, &args.relation)
        .map_err(|e| invalid_error!("{e}"))?;

    if args.json {
        primary_line!("{}", ix_cli_kit::json::encode(&report, ctx.pretty)?);
    } else {
        primary!("{}", render_human(&report));
    }

    if args.strict {
        let problems = strict_problems(&report);
        if !problems.is_empty() {
            partial_report!("{} (--strict)", problems.join("; "));
        }
    }
    Ok(())
}

/// Why `--strict` fails this report, one entry per distinct cause.
fn strict_problems(report: &RelationReport) -> Vec<String> {
    let rows: Vec<&RelationRow> = report.relations.iter().flat_map(|s| &s.rows).collect();
    if rows.is_empty() {
        return vec![
            "empty-population: the selected relations produced no rows, so nothing was measured"
                .to_string(),
        ];
    }
    let mut problems = Vec::new();
    if rows
        .iter()
        .all(|r| r.verification == Verification::NotTracked)
    {
        problems.push(
            "nothing-measured: every row is not-tracked, so no verification was measured"
                .to_string(),
        );
    }
    let uncovered = rows
        .iter()
        .filter(|r| r.state == RowState::Uncovered)
        .count();
    let weak = rows
        .iter()
        .filter(|r| {
            matches!(
                r.verification,
                Verification::Unverified | Verification::Partial
            )
        })
        .count();
    if uncovered > 0 || weak > 0 {
        problems.push(format!(
            "{uncovered} uncovered row(s) and {weak} unverified or partial row(s)"
        ));
    }
    problems
}

fn state_word(state: RowState) -> &'static str {
    match state {
        RowState::Uncovered => "uncovered",
        RowState::Covered => "covered",
    }
}

fn verification_word(verification: Verification) -> &'static str {
    match verification {
        Verification::Verified => "verified",
        Verification::Partial => "partial",
        Verification::Unverified => "unverified",
        Verification::NotTracked => "not-tracked",
    }
}

/// `repo:id` when the document's member declares an identity, else `id`.
fn label(document: &RowDocument) -> String {
    match &document.repo {
        Some(repo) => format!("{repo}:{}", document.id),
        None => document.id.clone(),
    }
}

/// One block per relation, one line per row (document, state, verification,
/// linked documents with their own verification), dangling and kind-mismatch
/// entries beneath, and the relation's summary last.
fn render_human(report: &RelationReport) -> String {
    let mut out = String::new();
    for section in &report.relations {
        out.push_str(&format!(
            "relation {} ({}, check {})\n",
            section.name,
            format!("{:?}", section.direction).to_lowercase(),
            section.check
        ));
        for row in &section.rows {
            let mut line = format!(
                "  {}  {}  {}",
                label(&row.document),
                state_word(row.state),
                verification_word(row.verification)
            );
            if row.state == RowState::Uncovered {
                line.push_str(&format!("  GAP {}", row.check));
            }
            let linked: Vec<String> = row
                .linked
                .iter()
                .map(|entry| match entry {
                    LinkedEntry::Resolved { document, .. } => format!(
                        "{} ({})",
                        label(document),
                        verification_word(document.verification)
                    ),
                    LinkedEntry::Dangling { reference, .. } => {
                        format!("{reference} (dangling)")
                    }
                })
                .collect();
            if !linked.is_empty() {
                line.push_str(&format!("  linked: {}", linked.join(", ")));
            }
            out.push_str(&line);
            out.push('\n');
            for entry in &row.dangling {
                out.push_str(&format!(
                    "    dangling: {} (via {})\n",
                    entry.reference, entry.verb
                ));
            }
            for entry in &row.kind_mismatch {
                let target = match &entry.repo {
                    Some(repo) => format!("{repo}:{}", entry.id),
                    None => entry.id.clone(),
                };
                out.push_str(&format!(
                    "    kind mismatch: {target} is {} (via {}), accepted: {}\n",
                    entry.archetype,
                    entry.verb,
                    entry.accepted.join("/")
                ));
            }
        }
        let s = &section.summary;
        out.push_str(&format!(
            "  summary: {} row(s); uncovered {}, covered {}; verified {}, partial {}, \
             unverified {}, not-tracked {}\n",
            s.rows,
            s.state.uncovered,
            s.state.covered,
            s.verification.verified,
            s.verification.partial,
            s.verification.unverified,
            s.verification.not_tracked
        ));
    }
    out
}
