//! `quire relations` — the two-way relation coverage report (quire-rs FR-084).
//!
//! The engine owns the rows, the states and the verification statuses; this
//! command resolves roots, applies path safety, calls
//! `quire_rs::relation_report` and renders. With `--design` it calls
//! `quire_rs::design_join` instead (FR-041). It holds no archetype, verb or
//! chain name: which rows exist comes from the loaded modules'
//! `traceability.required_relations`, and with `--design` from the relations
//! they name in `traceability.design_relations`.

use std::path::PathBuf;

use clap::Parser;
use quire_cli::safety;
use quire_rs::traceability::{DeclarationKind, RelationDirection};
use quire_rs::{
    design_join, relation_report, LinkedEntry, RelationReport, RelationReportMember, RelationRow,
    RowDocument, RowState, Verification,
};
use serde::Serialize;
use serde_json::Value;

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

    /// Module directory supplying `traceability.required_relations`,
    /// `design_relations` with `--design`, or `flow_rules` with `--flow`.
    /// Repeatable; explicit roots replace ambient discovery, with only exact
    /// installed providers from their `semantic.imports` added as dependencies.
    /// Without it, ambient discovery is bounded by the first `--repo` member,
    /// so `--repo` order decides which modules load.
    #[arg(long, value_name = "PATH")]
    pub module: Vec<String>,

    /// Restrict the report to the named declaration. Repeatable.
    #[arg(long, value_name = "NAME", conflicts_with = "design")]
    pub relation: Vec<String>,

    /// Emit the health-data-flow projection of the generic relation report.
    #[arg(long)]
    pub health_data_flows: bool,

    /// Emit the engine's generic flow report (FR-044).
    #[arg(long)]
    pub flow: bool,

    /// Render the design-element join (FR-041) instead of the relation
    /// selection: for each requirement the design elements that realize it, and
    /// a gap when there are none. Which relations are design relations is the
    /// loaded modules' `traceability.design_relations`.
    #[arg(long, conflicts_with_all = ["relation", "strict"])]
    pub design: bool,

    /// Emit the engine's report as JSON on stdout instead of the human form.
    #[arg(long)]
    pub json: bool,

    /// Emit the complete report, then exit 1 when a flow denial exists in
    /// `--flow` mode, or when relation rows are `uncovered`, `unverified` or
    /// `partial`, no rows were selected (`empty-population`), or every row is
    /// `not-tracked` (`nothing-measured`).
    #[arg(long)]
    pub strict: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    if args.health_data_flows {
        let mut conflicting = Vec::new();
        if !args.relation.is_empty() {
            conflicting.push("--relation");
        }
        if args.design {
            conflicting.push("--design");
        }
        if args.flow {
            conflicting.push("--flow");
        }
        if !conflicting.is_empty() {
            invalid_request!(
                "--health-data-flows conflicts with {}",
                conflicting.join(", ")
            );
        }
    }
    if args.flow {
        let mut conflicting = Vec::new();
        if !args.relation.is_empty() {
            conflicting.push("--relation");
        }
        if args.design {
            conflicting.push("--design");
        }
        if args.health_data_flows {
            conflicting.push("--health-data-flows");
        }
        if !conflicting.is_empty() {
            invalid_request!("--flow conflicts with {}", conflicting.join(", "));
        }
    }

    let corpus = if args.repo.is_empty() {
        let raw = args.scope.as_deref().unwrap_or(".");
        let scope = safety::validate_dir_path("--scope", raw)
            .map_err(|e| anyhow::Error::new(e).context(format!("validating --scope '{raw}'")))?;
        Corpus::from_scope(&scope)?
    } else {
        Corpus::from_repos(&args.repo)?
    };

    let discovery: PathBuf = corpus.members[0].root.clone();
    let registry = super::coverage::load_registry_for_relations(ctx, &args.module, &discovery)?;
    if args.flow {
        if registry
            .traceability()
            .is_none_or(|model| model.flow_rules.is_empty())
        {
            if let Some(failure) = registry.failures().first() {
                invalid_request!(
                    "module load failed: {} ({})",
                    failure.reason,
                    failure.path.display()
                );
            }
            invalid_request!(
                "no loaded module declares `traceability.flow_rules`, so there is \
                 no flow report to render; install a module that declares one or pass --module"
            );
        }
    } else if args.design {
        if registry
            .traceability()
            .is_none_or(|model| model.design_relations.is_empty())
        {
            invalid_request!(
                "no loaded module declares `traceability.design_relations`, so there is \
                 no design join to report; install a module that declares one or pass --module"
            );
        }
    } else if registry
        .traceability()
        .is_none_or(|model| model.required_relations.is_empty())
        && !registry
            .dropped_declarations()
            .any(|(_, declaration)| declaration.kind == DeclarationKind::RequiredRelation)
    {
        invalid_request!(
            "no loaded module declares `traceability.required_relations`, so there is \
             nothing to report; install a module that declares one or pass --module"
        );
    }

    if let Some(unknown) = args.relation.iter().find(|name| {
        !registry
            .traceability()
            .is_some_and(|model| model.required_relations.iter().any(|r| &r.name == *name))
            && !registry.dropped_declarations().any(|(_, declaration)| {
                declaration.kind == DeclarationKind::RequiredRelation
                    && declaration.item == name.as_str()
            })
    }) {
        invalid_request!("unknown --relation '{unknown}': no loaded module declares it");
    }

    let dropped_relations = registry
        .dropped_declarations()
        .any(|(_, declaration)| declaration.kind == DeclarationKind::RequiredRelation);
    let mut matrices = Vec::with_capacity(corpus.members.len());
    for member in &corpus.members {
        if dropped_relations {
            // Coverage computation rejects any incomplete model, including a
            // dropped relation that the relation report can safely isolate.
            // Keep valid relation rows visible and mark their unavailable
            // verification data as not-tracked.
            matrices.push(Vec::new());
        } else {
            matrices.push(
                super::coverage::compute_report_over(
                    ctx,
                    &member.root,
                    &registry,
                    &Corpus::member_spec(member),
                )?
                .coverage_matrix,
            );
        }
    }
    let inputs: Vec<RelationReportMember<'_>> = corpus
        .members
        .iter()
        .zip(&matrices)
        .map(|(member, matrix)| RelationReportMember {
            root: &member.root,
            matrix,
        })
        .collect();
    let report = if args.design {
        design_join(&corpus.spec, &registry, &inputs)
    } else {
        relation_report(&corpus.spec, &registry, &inputs, &args.relation)
    }
    .map_err(|e| invalid_error!("{e}"))?;

    if args.flow {
        let flow = report.flow.as_ref().ok_or_else(|| {
            anyhow::anyhow!("flow rules were declared but the engine returned no flow report")
        })?;
        if args.json {
            primary_line!("{}", ix_cli_kit::json::encode(flow, ctx.pretty)?);
        } else {
            primary!("{}", render_flow_human(flow));
        }
        if !report.errors.is_empty() {
            emit_dropped_relation_errors(ctx, &report);
            partial_report!("{} dropped required relation(s)", report.errors.len());
        }
        if args.strict {
            if !flow.denials.is_empty() {
                partial_report!("{} flow denial(s) (--strict)", flow.denials.len());
            }
        } else {
            let errors = flow
                .denials
                .iter()
                .filter(|denial| denial.severity == "error")
                .count();
            if errors > 0 {
                partial_report!("{} error-severity flow denial(s)", errors);
            }
        }
        return Ok(());
    }

    if args.health_data_flows {
        let projected = project_health_data_flows(&report, args.repo.is_empty())?;
        if args.json {
            primary_line!("{}", ix_cli_kit::json::encode(&projected, ctx.pretty)?);
        } else {
            primary!("{}", render_health_human(&projected));
        }
        if !report.errors.is_empty() {
            emit_dropped_relation_errors(ctx, &report);
            partial_report!("{} dropped required relation(s)", report.errors.len());
        }
        if args.strict && !projected.gaps.is_empty() {
            partial_report!(
                "{} health-data-flow gap(s) (--strict)",
                projected.gaps.len()
            );
        }
        return Ok(());
    }

    if args.json {
        primary_line!("{}", ix_cli_kit::json::encode(&report, ctx.pretty)?);
    } else {
        primary!("{}", render_human(&report, args.design));
    }

    if !report.errors.is_empty() {
        emit_dropped_relation_errors(ctx, &report);
        partial_report!("{} dropped required relation(s)", report.errors.len());
    }

    if args.strict {
        let problems = strict_problems(&report);
        if !problems.is_empty() {
            partial_report!("{} (--strict)", problems.join("; "));
        }
    }
    Ok(())
}

fn emit_dropped_relation_errors(ctx: &Ctx, report: &RelationReport) {
    for error in &report.errors {
        let message = format!(
            "error: dropped relation {} from module {}: {}",
            error.item, error.module, error.reason
        );
        ix_cli_kit::streams::Record::error("DroppedRelation", &message).emit(ctx.diagnostics);
    }
}

#[derive(Debug, Serialize)]
struct HealthOutput {
    name: &'static str,
    kind: &'static str,
    description: &'static str,
}

#[derive(Debug, Serialize)]
struct HealthReport {
    schema: &'static str,
    output: HealthOutput,
    flows: Vec<HealthFlow>,
    gaps: Vec<HealthGap>,
}

#[derive(Debug, Clone, Serialize)]
struct ModelDocument {
    repo: Option<String>,
    id: String,
    archetype: String,
    path: String,
}

#[derive(Debug, Clone, Serialize)]
struct DeclaredReference {
    repo: Option<String>,
    id: String,
    archetype: String,
    declared_by: ModelDocument,
}

#[derive(Debug, Clone, Serialize)]
struct HealthCrossing {
    boundary: ModelDocument,
    trust_level: String,
}

#[derive(Debug, Clone, Serialize)]
struct HealthFlow {
    connection: ModelDocument,
    source: DeclaredReference,
    destination: DeclaredReference,
    classifications: Vec<ModelDocument>,
    crossings: Vec<HealthCrossing>,
}

#[derive(Debug, Clone, Serialize)]
struct MissingReference {
    field: &'static str,
    reference: Option<DeclaredReference>,
}

#[derive(Debug, Clone, Serialize)]
struct HealthGap {
    connection: ModelDocument,
    missing: Vec<MissingReference>,
}

#[derive(Default)]
struct ConnectionFacts {
    document: Option<ModelDocument>,
    source: Option<DeclaredReference>,
    destination: Option<DeclaredReference>,
    endpoint_facts_seen: bool,
    connection_record_unavailable: bool,
    classifications: Vec<ModelDocument>,
    crossings: Vec<HealthCrossing>,
    missing: Vec<MissingReference>,
    health_classified: bool,
}

const HEALTH_OUTPUT: HealthOutput = HealthOutput {
    name: "health-data-flows",
    kind: "record",
    description: "Health-data flow records; PLAT-1756 in the Data-Flow Model project owns the producer definition.",
};

/// Build the FR-046 view from the generic FR-084 relation report. No corpus
/// documents are read here; every input is a selected relation fact.
fn project_health_data_flows(report: &RelationReport, scope: bool) -> anyhow::Result<HealthReport> {
    let mut connections =
        std::collections::BTreeMap::<(Option<String>, String), ConnectionFacts>::new();

    for section in &report.relations {
        let relation_name = section.name.to_ascii_lowercase();
        let section_is_crossing =
            relation_name.contains("trust") && relation_name.contains("boundary");
        for row in &section.rows {
            let key = (row.document.repo.clone(), row.document.id.clone());
            let facts = connections.entry(key).or_default();
            let connection = model_document(&row.document)?;
            facts.document = Some(connection.clone());

            if let Some(record) = row.document.record.as_ref() {
                let source = endpoint(record.get("sourceEnd"), &connection, scope);
                let destination = endpoint(record.get("targetEnd"), &connection, scope);
                if source.is_some()
                    || destination.is_some()
                    || record.contains_key("sourceEnd")
                    || record.contains_key("targetEnd")
                {
                    facts.endpoint_facts_seen = true;
                    facts.source = source;
                    facts.destination = destination;
                }
            }

            let mut has_boundary_link = false;
            for entry in &row.linked {
                let LinkedEntry::Resolved { document, .. } = entry else {
                    continue;
                };
                if is_health_classification(document) {
                    facts.health_classified = true;
                    facts.connection_record_unavailable |= row.document.record.is_none();
                    facts.classifications.push(model_document(document)?);
                }
                if let Some(trust_level) = document
                    .record
                    .as_ref()
                    .and_then(|record| record.get("trust_level"))
                    .and_then(Value::as_str)
                {
                    has_boundary_link = true;
                    facts.crossings.push(HealthCrossing {
                        boundary: model_document(document)?,
                        trust_level: trust_level.to_owned(),
                    });
                } else if is_trust_boundary(document) {
                    has_boundary_link = true;
                    facts.missing.push(MissingReference {
                        field: "trust_level",
                        reference: Some(declared_document_reference(document, &connection)?),
                    });
                }
            }

            if section_is_crossing || has_boundary_link {
                for entry in &row.dangling {
                    let reference = parse_declared_reference(
                        &entry.reference,
                        &connection,
                        scope,
                        "TrustBoundary",
                    );
                    facts.missing.push(MissingReference {
                        field: "crossing",
                        reference,
                    });
                }
            }
        }
    }

    let mut flows = Vec::new();
    let mut gaps = Vec::new();
    for (_, mut facts) in connections {
        if !facts.health_classified {
            continue;
        }
        let connection = facts
            .document
            .take()
            .ok_or_else(|| anyhow::anyhow!("health flow has no connection document"))?;
        if facts.connection_record_unavailable {
            facts
                .missing
                .retain(|missing| !matches!(missing.field, "source" | "destination"));
            facts.missing.push(MissingReference {
                field: "connection_record",
                reference: None,
            });
        } else {
            if !facts.endpoint_facts_seen || facts.source.is_none() {
                facts.missing.push(MissingReference {
                    field: "source",
                    reference: None,
                });
            }
            if !facts.endpoint_facts_seen || facts.destination.is_none() {
                facts.missing.push(MissingReference {
                    field: "destination",
                    reference: None,
                });
            }
        }
        sort_dedup_documents(&mut facts.classifications);
        sort_dedup_crossings(&mut facts.crossings);
        sort_missing(&mut facts.missing);
        facts.missing.dedup_by(|left, right| {
            left.field == right.field
                && left.reference.as_ref().map(|r| (&r.repo, &r.id))
                    == right.reference.as_ref().map(|r| (&r.repo, &r.id))
        });

        if facts.missing.is_empty() {
            if let (Some(source), Some(destination)) = (facts.source, facts.destination) {
                flows.push(HealthFlow {
                    connection,
                    source,
                    destination,
                    classifications: facts.classifications,
                    crossings: facts.crossings,
                });
            }
        } else {
            gaps.push(HealthGap {
                connection,
                missing: facts.missing,
            });
        }
    }

    flows.sort_by(|left, right| {
        document_key(&left.connection).cmp(&document_key(&right.connection))
    });
    gaps.sort_by(|left, right| {
        document_key(&left.connection).cmp(&document_key(&right.connection))
    });
    Ok(HealthReport {
        schema: "quire.health-data-flows/v1",
        output: HEALTH_OUTPUT,
        flows,
        gaps,
    })
}

fn model_document(document: &RowDocument) -> anyhow::Result<ModelDocument> {
    let archetype = document.archetype.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "relation report document '{}' has no archetype",
            document.id
        )
    })?;
    Ok(ModelDocument {
        repo: document.repo.clone(),
        id: document.id.clone(),
        archetype,
        path: document.path.clone(),
    })
}

fn endpoint(
    value: Option<&Value>,
    connection: &ModelDocument,
    scope: bool,
) -> Option<DeclaredReference> {
    let id = value?.get("type")?.as_str()?;
    parse_declared_reference(id, connection, scope, "port")
}

fn parse_declared_reference(
    target: &str,
    declared_by: &ModelDocument,
    scope: bool,
    archetype: &str,
) -> Option<DeclaredReference> {
    let (id, target_repo) = if let Some(uri) = target.strip_prefix("ix://") {
        let parts: Vec<&str> = uri.split('/').collect();
        if parts.len() < 2 {
            return None;
        }
        let id = *parts.last()?;
        let repo = (parts.len() >= 3).then(|| format!("{}/{}", parts[0], parts[1]));
        (id, repo)
    } else {
        (target, declared_by.repo.clone())
    };
    if id.is_empty() {
        return None;
    }
    Some(DeclaredReference {
        repo: if scope { None } else { target_repo },
        id: id.to_owned(),
        archetype: archetype.to_owned(),
        declared_by: declared_by.clone(),
    })
}

fn declared_document_reference(
    document: &RowDocument,
    declared_by: &ModelDocument,
) -> anyhow::Result<DeclaredReference> {
    Ok(DeclaredReference {
        repo: document.repo.clone(),
        id: document.id.clone(),
        archetype: document
            .archetype
            .clone()
            .ok_or_else(|| anyhow::anyhow!("boundary '{}' has no archetype", document.id))?,
        declared_by: declared_by.clone(),
    })
}

fn is_health_classification(document: &RowDocument) -> bool {
    document
        .record
        .as_ref()
        .and_then(|record| record.get("regulatedCategories"))
        .and_then(Value::as_array)
        .is_some_and(|categories| {
            categories
                .iter()
                .any(|category| category.as_str() == Some("health"))
        })
}

fn is_trust_boundary(document: &RowDocument) -> bool {
    document.archetype.as_deref().is_some_and(|archetype| {
        let lower = archetype.to_ascii_lowercase();
        lower.contains("trust") && lower.contains("boundary")
    })
}

fn document_key(document: &ModelDocument) -> (Option<&str>, &str) {
    (document.repo.as_deref(), &document.id)
}

fn sort_dedup_documents(documents: &mut Vec<ModelDocument>) {
    documents.sort_by(|left, right| document_key(left).cmp(&document_key(right)));
    documents.dedup_by(|left, right| document_key(left) == document_key(right));
}

fn sort_dedup_crossings(crossings: &mut Vec<HealthCrossing>) {
    crossings
        .sort_by(|left, right| document_key(&left.boundary).cmp(&document_key(&right.boundary)));
    crossings.dedup_by(|left, right| document_key(&left.boundary) == document_key(&right.boundary));
}

fn sort_missing(missing: &mut [MissingReference]) {
    missing.sort_by(|left, right| {
        let left_key = left
            .reference
            .as_ref()
            .map(|reference| (reference.repo.as_deref(), reference.id.as_str()));
        let right_key = right
            .reference
            .as_ref()
            .map(|reference| (reference.repo.as_deref(), reference.id.as_str()));
        left.field
            .cmp(right.field)
            .then_with(|| left_key.cmp(&right_key))
    });
}

fn render_health_human(report: &HealthReport) -> String {
    let mut out = String::new();
    for flow in &report.flows {
        out.push_str(&format!(
            "{}: {} -> {}\n",
            flow.connection.id, flow.source.id, flow.destination.id
        ));
    }
    for gap in &report.gaps {
        let fields = gap
            .missing
            .iter()
            .map(|missing| missing.field)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("{}: missing {fields}\n", gap.connection.id));
    }
    out
}

fn render_flow_human(report: &quire_rs::corpus::FlowReport) -> String {
    let mut out = String::new();
    for denial in &report.denials {
        let subject = match &denial.subject.repo {
            Some(repo) => format!("{repo}:{}", denial.subject.id),
            None => denial.subject.id.clone(),
        };
        let matched = denial
            .matched
            .iter()
            .map(|(archetype, documents)| {
                let ids = documents
                    .iter()
                    .map(|document| match &document.repo {
                        Some(repo) => format!("{repo}:{}", document.id),
                        None => document.id.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{archetype}: [{ids}]")
            })
            .collect::<Vec<_>>()
            .join("; ");
        let remediations = denial
            .remediations
            .iter()
            .map(|remediation| remediation.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "{}: {} ({}) — {} [matched: {}] [remediations: {}]\n",
            denial.check, subject, denial.severity, denial.rationale, matched, remediations
        ));
    }
    out
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

fn direction_word(direction: RelationDirection) -> &'static str {
    match direction {
        RelationDirection::Incoming => "incoming",
        RelationDirection::Outgoing => "outgoing",
        RelationDirection::Either => "either",
    }
}

/// How the human report names a kind-mismatch target that has no `type:`; the
/// engine's own `validate --okf` finding spells it the same way.
pub(crate) const UNTYPED: &str = "(untyped)";

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

/// A design element as `id (kind)`, with its repo when present, followed by the
/// criterion ids it realizes.
fn design_element(document: &RowDocument, realized: Option<&[String]>) -> String {
    let kind = document.archetype.as_deref().unwrap_or(UNTYPED);
    let mut text = format!("{} ({kind})", label(document));
    if let Some(criteria) = realized.filter(|c| !c.is_empty()) {
        text.push_str(&format!(" realizes {}", criteria.join(", ")));
    }
    text
}

/// One block per relation, one line per row (document, state, verification,
/// linked documents with their own verification), dangling and kind-mismatch
/// entries beneath, and the relation's summary last.
fn render_human(report: &RelationReport, design: bool) -> String {
    let mut out = String::new();
    for error in &report.errors {
        out.push_str(&format!(
            "error: dropped relation {} from module {}: {}\n",
            error.item, error.module, error.reason
        ));
    }
    for section in &report.relations {
        out.push_str(&format!(
            "relation {} ({}, check {})\n",
            section.name,
            direction_word(section.direction),
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
                let gap = if design { "no design element" } else { "GAP" };
                line.push_str(&format!("  {gap} {}", row.check));
            }
            let linked: Vec<String> = row
                .linked
                .iter()
                .map(|entry| match entry {
                    LinkedEntry::Resolved {
                        document,
                        realized_criteria,
                        ..
                    } if design => design_element(document, realized_criteria.as_deref()),
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
                let actual = entry.archetype.as_deref().unwrap_or(UNTYPED);
                out.push_str(&format!(
                    "    kind mismatch: {target} is {actual} (via {}), accepted: {}\n",
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
    if let Some(flow) = &report.flow {
        out.push_str("flow:\n");
        out.push_str(&render_flow_human(flow));
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ix_trace_rs::trace;
    use quire_rs::corpus::{
        FlowDenial, FlowDenialRemediation, FlowRemediationChange, FlowReport, FlowSubjectRef,
    };
    use quire_rs::traceability::FlowSetRemediation;

    use super::render_flow_human;

    #[test]
    #[trace("FR-044-AC-2")]
    fn human_flow_output_renders_denial_remediation_names() {
        let report = FlowReport {
            schema: "quire.flow-report/v1",
            rules: vec![],
            vocabularies: BTreeMap::new(),
            facts: BTreeMap::new(),
            denials: vec![FlowDenial {
                rule: "generic-rule".into(),
                check: "generic-check".into(),
                subject: FlowSubjectRef {
                    id: "A-1".into(),
                    repo: None,
                    archetype: "alpha".into(),
                },
                matched: BTreeMap::new(),
                remediations: vec![FlowDenialRemediation {
                    name: "generic-fix".into(),
                    description: "Set the authored state.".into(),
                    change: FlowRemediationChange {
                        adds: None,
                        sets: Some(FlowSetRemediation {
                            field: "state".into(),
                            value: "covered".into(),
                        }),
                        removes: None,
                    },
                }],
                rationale: "A declared condition holds.".into(),
                severity: "warning",
            }],
            remediations: vec![],
        };

        let output = render_flow_human(&report);
        assert!(output.contains("generic-check: A-1 (warning)"));
        assert!(output.contains("[remediations: generic-fix]"));
    }
}
