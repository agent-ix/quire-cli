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
use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
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
    /// Equivalent to `--format json`.
    #[arg(long, conflicts_with = "format")]
    pub json: bool,

    /// Output form: `human` census on stderr (default), `json` payload on
    /// stdout, or `tsv` — one tab-separated record per line on stdout, the
    /// agent-sized projection of the same records (FR-017-AC-14).
    #[arg(long, value_enum, value_name = "FORMAT")]
    pub format: Option<OutputFormat>,

    /// Exit 1 when any row is unbacked or any status is contradicted. Off by
    /// default: the rollup is a report, and whether a gap blocks is the
    /// consuming workflow's policy, not this command's.
    #[arg(long)]
    pub strict: bool,

    /// Override a coverage check's severity: `--severity coverage:<check>=<level>`
    /// where `<check>` is `unbacked-row`, `status-lie`, `untracked-symbol` or
    /// `undeclared-status` and `<level>` is `off`, `warning` or `error` — the
    /// same FR-048 machinery `validate --severity` uses, layered over a
    /// module's `grammar_severity` entries. `off` drops the kind's records
    /// from every output surface (the projection lever, FR-017-AC-13);
    /// `error` exits 1 when the kind has findings. Totals always describe the
    /// full reconciliation, and `--strict` is unaffected by projection.
    /// Repeatable; a malformed entry — or a `coverage:` entry naming a check
    /// outside the pack's four — is rejected before any document is read.
    #[arg(long = "severity", value_name = "PACK:CHECK=LEVEL")]
    pub severity: Vec<String>,
}

/// How the report leaves the process (FR-017-AC-1/AC-2/AC-14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// The human census on stderr; stdout stays empty.
    Human,
    /// The `CoverageReport` JSON payload on stdout — the stable interface.
    Json,
    /// One tab-separated record per line on stdout.
    Tsv,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let scope = safety::validate_dir_path("--scope", &args.scope)
        .with_context(|| format!("validating --scope '{}'", args.scope))?;
    let registry = load_registry(ctx, &args, &scope)?;
    // FR-017-AC-13 (#53): layer `--severity` over the module-declared
    // `grammar_severity` map — the identical call `validate` makes — so a
    // malformed entry is rejected here, before any document is read. The
    // pack's check vocabulary is additionally closed (#57): FR-048 validates
    // only the key's shape, deliberate for `validate` where grammars are
    // module-declared and open, but this command owns the `coverage` pack and
    // its four checks — a typo'd check would otherwise merge, match nothing,
    // and silently not project what the operator asked for.
    reject_unknown_pack_checks(&args.severity)?;
    let registry = super::validate::apply_severity_overrides(&registry, &args.severity)?;

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
    let model = registry
        .traceability()
        .expect("traceability model checked above");
    // CR-085: a module may declare `source_exclude` globs naming fixture trees
    // that hold no traceable source. Two filters, different in kind — the
    // document root is the caller's non-configurable argument (CR-045), the
    // globs are declared data that can only subtract within the code root.
    //
    // The engine shipping the key is inert until this line passes it; that is
    // the failure the last programme phase kept finding, so it is wired in the
    // same release rather than the next one.
    let extraction = quire_rs::symbols::extract_tree_scoped(
        &scope,
        &[Path::new(super::DOCUMENT_ROOT_DIR)],
        &model.source_exclude,
    );
    // FR-017-AC-18 (#51, quire-rs #215): extraction diagnostics reach stderr.
    // A refused `source_exclude` list (FR-050-AC-25) or an unreadable source
    // file used to be computed and dropped here — a walk that silently read
    // less than the operator declared. Stderr on every format: diagnostics
    // are the progress/finding stream, never part of the stdout payload.
    for d in &extraction.diagnostics {
        io::emit_diagnostic(
            ctx.diagnostics,
            "SymbolExtraction",
            &format!("{}: {}", d.path, d.reason),
        );
    }
    let graph = quire_rs::symbols::trace::bind(&extraction, model);

    let report =
        compute_coverage(&spec, &registry, &graph, &scope).map_err(|e| anyhow::anyhow!("{e}"))?;

    // FR-017-AC-13 (#53): the coverage severity pack. Counts are captured on
    // the FULL computation, before projection — `--strict` and `error`
    // promotion judge what the engine found, never what rendering shows, so a
    // projected report can never pass a gate a full one would fail.
    let severity = registry.grammar_severity().clone();
    let full_counts = [
        ("unbacked-row", report.unbacked_rows.len()),
        ("status-lie", report.status_lies.len()),
        ("untracked-symbol", report.untracked_symbols.len()),
        ("undeclared-status", report.undeclared_statuses.len()),
    ];
    let report = project_by_severity(ctx, report, &severity);

    let format = match args.format {
        Some(f) => f,
        None if args.json => OutputFormat::Json,
        None => OutputFormat::Human,
    };
    match format {
        // Compact by default, indented under the global `--pretty`
        // (FR-008-AC-1, FR-017-AC-15): `to_json()` is unconditionally pretty
        // and the flag was silently ignored (#53). Whitespace-only — the
        // payload parses identically, and `--pretty` restores the old shape.
        //
        // `engine::attach` appends the provenance block, leaving every key the
        // engine emitted in the engine's own order (#68) — it flattens rather
        // than round-tripping through `serde_json::Value`, which would sort
        // every key at every depth alphabetically. A saved payload is the
        // artifact a later reader reasons from, and without provenance it
        // carries no way to learn which build produced it — the defect that let
        // four battletest passes cite figures from a binary that could not emit
        // `binding_census`.
        OutputFormat::Json => println!(
            "{}",
            io::encode_json(&quire_cli::engine::attach(&report), ctx.pretty)?
        ),
        OutputFormat::Tsv => print!("{}", render_tsv(&report)),
        OutputFormat::Human => emit_human(ctx, &report),
    }

    // An `error`-promoted kind fails the run even without --strict — the
    // per-check gate `--strict` cannot express, mirroring `validate`
    // (FR-048-AC-6). Judged on the full computation above.
    let promoted: Vec<String> = full_counts
        .iter()
        .filter(|(check, n)| *n > 0 && pack_level(&severity, check) == GrammarSeverityLevel::Error)
        .map(|(check, n)| format!("{n} coverage:{check} finding(s)"))
        .collect();
    if !promoted.is_empty() {
        bail!("{} at severity `error` (--severity)", promoted.join(", "));
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
        let (unbacked, lies) = (full_counts[0].1, full_counts[1].1);
        if unbacked > 0 || lies > 0 {
            bail!("{unbacked} unbacked row(s) and {lies} contradicted status(es) (--strict)");
        }
    }
    Ok(())
}

/// The four checks the `coverage` severity pack owns (FR-017-AC-13). The
/// gateable engine kinds, and only those — `no_symbol_rows` is an exemption
/// note, deliberately outside the pack (FR-017 CR note, #51).
const PACK_CHECKS: [&str; 4] = [
    "unbacked-row",
    "status-lie",
    "untracked-symbol",
    "undeclared-status",
];

/// Reject a `coverage:<check>` entry naming a check outside [`PACK_CHECKS`]
/// (#57). FR-048's parser validates only the key's *shape* — deliberate for
/// `validate`, where grammars are module-declared and open — so a typo'd
/// check (`coverage:unbaked-row=off`) would merge, match nothing, and the run
/// would proceed as if the flag were absent: a projection the operator asked
/// for silently not happening. Entries for other grammars are left to the
/// FR-048 open-vocabulary posture unchanged.
fn reject_unknown_pack_checks(entries: &[String]) -> anyhow::Result<()> {
    for entry in entries {
        let Some(rest) = entry.strip_prefix("coverage:") else {
            continue;
        };
        let check = rest.split('=').next().unwrap_or(rest);
        if !PACK_CHECKS.contains(&check) {
            bail!(
                "--severity entry '{entry}' names no coverage check: the coverage \
                 pack's checks are {}",
                PACK_CHECKS.join(", ")
            );
        }
    }
    Ok(())
}

/// The configured level of one `coverage` pack check — `coverage:<check>` in
/// the merged severity map, defaulting to `warning` like every FR-048 check.
fn pack_level(severity: &GrammarSeverityMap, check: &str) -> GrammarSeverityLevel {
    quire_rs::grammar::severity_level(severity, "coverage", check)
}

/// Drop every `off` kind's records from the report (FR-017-AC-13, #53) —
/// human, JSON and TSV all render the same projected struct — announcing each
/// non-empty suppression on stderr so a projected report can never be
/// mistaken for a clean one. `totals` and `groups` are untouched: they
/// describe the full reconciliation, captured before this call.
fn project_by_severity(
    ctx: &Ctx,
    mut report: quire_rs::CoverageReport,
    severity: &GrammarSeverityMap,
) -> quire_rs::CoverageReport {
    let note = |check: &str, n: usize| {
        if n > 0 {
            io::emit_diagnostic(
                ctx.diagnostics,
                "CoverageProjection",
                &format!("{n} coverage:{check} finding(s) suppressed by severity `off`"),
            );
        }
    };
    if pack_level(severity, "unbacked-row") == GrammarSeverityLevel::Off {
        note("unbacked-row", report.unbacked_rows.len());
        report.unbacked_rows.clear();
    }
    if pack_level(severity, "status-lie") == GrammarSeverityLevel::Off {
        note("status-lie", report.status_lies.len());
        report.status_lies.clear();
    }
    if pack_level(severity, "untracked-symbol") == GrammarSeverityLevel::Off {
        note("untracked-symbol", report.untracked_symbols.len());
        report.untracked_symbols.clear();
    }
    if pack_level(severity, "undeclared-status") == GrammarSeverityLevel::Off {
        note("undeclared-status", report.undeclared_statuses.len());
        report.undeclared_statuses.clear();
    }
    report
}

/// A TSV cell: the two structural characters (tab, newline) become spaces.
/// Measured on a real corpus, 0 of 1,107 statements contain either — the
/// replacement is the guard, not the common case (#53).
fn tsv_cell(s: &str) -> String {
    s.chars()
        .map(|c| {
            if matches!(c, '\t' | '\n' | '\r') {
                ' '
            } else {
                c
            }
        })
        .collect()
}

/// One TSV record: the kind, then the eight fixed data cells.
fn tsv_line(kind: &str, cells: [&str; 8]) -> String {
    let mut line = String::from(kind);
    for c in cells {
        line.push('\t');
        line.push_str(&tsv_cell(c));
    }
    line.push('\n');
    line
}

/// The tab-separated projection (FR-017-AC-14, #53): full records, one per
/// line on stdout — ~36% of the JSON payload on a real corpus, and the first
/// human-form rendering `diagnostics`, `obligations` and `implements` have
/// ever had.
///
/// Nine fixed columns — `kind id document reference status method line
/// targets text` — with the empty string where a kind carries no value. The
/// `line` column carries the engine's 1-based line where the record has one
/// (quire-rs v0.42.0, #210 — the arrival #53 reserved the column for), empty
/// where it does not; `targets` flattens id lists with `,`; obligation
/// `parameters` are deliberately omitted (a map does not flatten into one
/// column without an escaping contract). Ordering mirrors the JSON arrays, so
/// output is byte-identical across runs over identical input (FR-050-AC-7).
fn render_tsv(report: &quire_rs::CoverageReport) -> String {
    let line_cell = |line: Option<usize>| line.map(|l| l.to_string()).unwrap_or_default();
    let mut out =
        String::from("kind\tid\tdocument\treference\tstatus\tmethod\tline\ttargets\ttext\n");
    for r in &report.unbacked_rows {
        let targets = r.target_ids.join(",");
        out.push_str(&tsv_line(
            "unbacked-row",
            [
                r.row_id.as_deref().unwrap_or(""),
                &r.document,
                &r.reference,
                "",
                "",
                &line_cell(r.line),
                &targets,
                "",
            ],
        ));
    }
    for l in &report.status_lies {
        let targets = l.target_ids.join(",");
        out.push_str(&tsv_line(
            "status-lie",
            [
                l.row_id.as_deref().unwrap_or(""),
                &l.document,
                &l.reference,
                &l.status,
                "",
                &line_cell(l.line),
                &targets,
                "",
            ],
        ));
    }
    for n in &report.no_symbol_rows {
        let targets = n.target_ids.join(",");
        out.push_str(&tsv_line(
            "no-symbol-row",
            [
                n.row_id.as_deref().unwrap_or(""),
                &n.document,
                &n.reference,
                "",
                &n.test_type,
                &line_cell(n.line),
                &targets,
                "",
            ],
        ));
    }
    for s in &report.undeclared_statuses {
        out.push_str(&tsv_line(
            "undeclared-status",
            [
                s.row_id.as_deref().unwrap_or(""),
                &s.document,
                &s.reference,
                &s.status,
                "",
                &line_cell(s.line),
                "",
                "",
            ],
        ));
    }
    for u in &report.untracked_symbols {
        out.push_str(&tsv_line(
            "untracked-symbol",
            [
                "",
                &u.path,
                "",
                "",
                "",
                &line_cell(u.line),
                &u.trace_id,
                &u.symbol,
            ],
        ));
    }
    for d in &report.diagnostics {
        out.push_str(&tsv_line(
            "diagnostic",
            [
                "",
                d.path.as_deref().unwrap_or(""),
                &d.declaration,
                &d.reason,
                "",
                "",
                "",
                &d.message,
            ],
        ));
    }
    for o in &report.obligations {
        let targets = o.target_ids.join(",");
        out.push_str(&tsv_line(
            "obligation",
            [
                &o.id,
                &o.document,
                &o.source,
                o.criticality.as_deref().unwrap_or(""),
                o.method.as_deref().unwrap_or(""),
                "",
                &targets,
                &o.statement,
            ],
        ));
    }
    for i in &report.implements {
        out.push_str(&tsv_line(
            "implements",
            ["", &i.path, &i.form, "", "", "", &i.trace_id, &i.symbol],
        ));
    }
    out
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

/// The lead and trailer of a human finding line (#51, FR-017-AC-12/AC-16).
///
/// With a row id — the declaration names a `row_id_column` — the id leads
/// (`TC-123 (doc.md:7) …`) and the reference kind moves to a bracketed
/// trailer (`… [traces-to]`) so it stays visible. Without one, the reference
/// kind leads exactly as before and there is no trailer: the kind is already
/// the only identity the record has.
///
/// The parenthesized locus is `document:line` when the record carries the
/// engine's 1-based line (quire-rs v0.42.0, FR-050-AC-26) — the clickable
/// `path:line` form `validate` established — and the bare document when it
/// does not, exactly as before.
fn finding_identity(
    row_id: Option<&str>,
    line: Option<usize>,
    reference: &str,
    document: &str,
) -> (String, String) {
    let locus = match line {
        Some(l) => format!("{document}:{l}"),
        None => document.to_string(),
    };
    match row_id {
        Some(id) => (format!("{id} ({locus})"), format!(" [{reference}]")),
        None => (format!("{reference} ({locus})"), String::new()),
    }
}

/// The binding census, one line per language — context, not a finding (#66).
///
/// A pure function so it can be asserted. `emit_human` writes to two streams
/// through a global, and a rendering nothing can read back is a rendering
/// nothing can test — which is how `report.diagnostics` came to render in the
/// TSV path and nowhere else.
fn census_lines(report: &quire_rs::CoverageReport) -> Vec<String> {
    report
        .binding_census
        .iter()
        .map(|c| {
            let forms = if c.forms.is_empty() {
                "none declared".to_string()
            } else {
                c.forms.join(", ")
            };
            let unmatched = c.unmatched_example.as_ref().map_or_else(String::new, |e| {
                format!(", unread tag `{}` at {}:{}", e.symbol, e.path, e.line)
            });
            format!(
                "{}: {}/{}/{} bound/tagged/candidates ({} read; {} authored; forms: {forms}{unmatched})",
                c.language,
                c.bound,
                c.tagged,
                c.candidates,
                percent_label(c.bound, c.candidates),
                percent_label(c.tagged, c.candidates)
            )
        })
        .collect()
}

/// The FR-063 envelope of each measured RATIO metric (#66).
///
/// Ratios only: a count's value and its `matched` are the same fact, so
/// enveloping one says the same number twice. A `not_computed` metric already
/// carries its own `because`, and printing "examined 0" for it would read as a
/// measurement that ran and found nothing.
fn metric_lines(report: &quire_rs::CoverageReport) -> Vec<String> {
    report
        .metrics
        .iter()
        .filter_map(|m| match m.measurement {
            quire_rs::metric::Measurement::Measured {
                population,
                examined,
                matched,
                ..
            } if m.shape == quire_rs::metric::MetricShape::Ratio => Some(format!(
                "{}: over {} {}(s), examined {}, matched {}",
                m.name, population, m.unit, examined, matched
            )),
            _ => None,
        })
        .collect()
}

fn emit_human(ctx: &Ctx, report: &quire_rs::CoverageReport) {
    // CR-012: the census goes to **stdout**. It is what a caller redirecting
    // with `>` came for, and it is not a diagnostic — `1238/2390 rows backed
    // (51%)` rendered in error red was the whole of defect 2 in #59.
    let t = &report.totals;
    io::emit_result(&format!(
        "Coverage: {}/{} rows backed ({})",
        t.backed,
        t.total,
        percent_label(t.backed, t.total)
    ));
    for line in census_lines(report) {
        io::emit_result(&line);
    }
    for g in &report.groups {
        io::emit_result(&format!(
            "{}: {}/{} ({})",
            g.document,
            g.backed,
            g.total,
            percent_label(g.backed, g.total)
        ));
    }
    // FR-017-AC-18 (#51, quire-rs #215): what `source_exclude` subtracted is
    // part of the census. An over-broad glob otherwise reads exactly like
    // tests that were never written. Zero — the state every conformant repo
    // without the declaration is in — prints nothing.
    if report.excluded_source_files > 0 {
        io::emit_result(&format!(
            "{} source file(s) excluded by source_exclude",
            report.excluded_source_files
        ));
    }
    for line in metric_lines(report) {
        io::emit_result(&line);
    }
    // Alerts nobody saw. `report.diagnostics` rendered in the TSV path and
    // nowhere else, so 11 `uncatalogued-verification-method` findings on
    // `quire-cli` and every `no-symbol-bound` were invisible to anyone who ran
    // the command the normal way. Same field selection as the TSV row, so the
    // two surfaces cannot disagree about what a diagnostic says.
    for d in &report.diagnostics {
        let locus = match &d.path {
            Some(path) => format!(" ({path})"),
            None => String::new(),
        };
        io::emit_diagnostic(
            ctx.diagnostics,
            "CoverageDiagnostic",
            &format!("[{}] {}{locus}", d.reason, d.message),
        );
    }
    // Suspicions keep their advisory framing (FR-064): a suspicion is a thing
    // that LOOKS wrong with the measurement that made it look wrong, and the
    // evidence is not decoration — one a reader cannot check in a glance is one
    // they learn to scroll past.
    for s in &report.suspicions {
        io::emit_diagnostic(
            ctx.diagnostics,
            "Suspicion",
            &format!(
                "[{}] {} in {}:{} — {} ({})",
                s.kind, s.symbol, s.path, s.line, s.message, s.evidence
            ),
        );
    }
    // #51: each finding line leads with the row's own id when the record
    // carries one, so a reader can act on the line without going to `--json`
    // — `traces-to (spec/tests.md)` four times over names nothing. Every
    // row_id-carrying record kind renders here (AC-12/AC-17), and
    // `UntrackedSymbol` carries no `row_id` by construction — it is a symbol
    // matching no declared row, and its `trace_id` already prints.
    for r in &report.unbacked_rows {
        let (lead, kind) = finding_identity(r.row_id.as_deref(), r.line, &r.reference, &r.document);
        io::emit_diagnostic(
            ctx.diagnostics,
            "UnbackedRow",
            &format!("{lead} has no backing symbol{kind}"),
        );
    }
    for l in &report.status_lies {
        let (lead, kind) = finding_identity(l.row_id.as_deref(), l.line, &l.reference, &l.document);
        io::emit_diagnostic(
            ctx.diagnostics,
            "StatusLie",
            &format!("{lead} claims `{}` but is not backed{kind}", l.status),
        );
    }
    // FR-017-AC-17 (#51): rendered like every other row-id-carrying kind. It
    // was JSON/TSV-only — but the record is the *explanation* for an unbacked
    // row the census does print, and an explanation only the machine surface
    // carries is one nobody reads (the CR-083 argument, unchanged).
    for n in &report.no_symbol_rows {
        let (lead, kind) = finding_identity(n.row_id.as_deref(), n.line, &n.reference, &n.document);
        io::emit_diagnostic(
            ctx.diagnostics,
            "NoSymbolRow",
            &format!(
                "{lead} is verified by `{}`, which mints no source symbol{kind}",
                n.test_type
            ),
        );
    }
    // CR-083. Rendered rather than left to `--json`, because a finding only the
    // machine surface carries is a finding nobody reads: the whole defect this
    // class reports is a value the engine had an opinion about and never said.
    for s in &report.undeclared_statuses {
        let (lead, kind) = finding_identity(s.row_id.as_deref(), s.line, &s.reference, &s.document);
        io::emit_diagnostic(
            ctx.diagnostics,
            "UndeclaredStatus",
            &format!(
                "{lead} has status `{}`, which the declared vocabulary classes as nothing{kind}",
                s.status
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
    load_registry_for(ctx, &args.module, scope)
}

/// The module-resolution `coverage` performs, taking the flag rather than the
/// whole `Args` so a sibling command resolves modules identically.
///
/// Shared rather than restated: `quire symbols` reports over the same walk and
/// the same declaration, and two resolution orders would let the two commands
/// disagree about which module is in scope for the same invocation — which is
/// the class of drift this repository keeps finding one list at a time.
pub(super) fn load_registry_for(
    ctx: &Ctx,
    module: &Option<String>,
    scope: &Path,
) -> anyhow::Result<Registry> {
    if let Some(raw) = module {
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

#[cfg(test)]
mod tests {
    use super::*;

    // TC-813, FR-017-AC-18 (#66): the census and the metric envelopes reach
    // the human surface, and the census example appears only where there is
    // something unread to look at.
    #[test]
    fn tc813_the_census_and_metric_envelopes_render_for_a_human() {
        use quire_rs::metric::{Measurement, Metric, MetricShape};
        use quire_rs::symbols::trace::{BindingCensus, UnboundSymbol};

        let census = vec![
            BindingCensus {
                language: "rust".to_string(),
                candidates: 1513,
                tagged: 1400,
                bound: 1344,
                forms: vec!["rust-trace-attribute".to_string()],
                unbound_example: Some(UnboundSymbol {
                    path: "crates/a/src/lib.rs".to_string(),
                    line: 732,
                    symbol: "tests::covers".to_string(),
                }),
                unmatched_example: Some(UnboundSymbol {
                    path: "crates/a/src/lib.rs".to_string(),
                    line: 700,
                    symbol: "tests::misspelled".to_string(),
                }),
            },
            BindingCensus {
                language: "typescript".to_string(),
                candidates: 12,
                tagged: 12,
                bound: 12,
                forms: vec!["ts-trace-helper".to_string()],
                // Fully bound, so nothing to look at even if an example
                // survived from an earlier run.
                unbound_example: Some(UnboundSymbol {
                    path: "ui/x.ts".to_string(),
                    line: 4,
                    symbol: "stale".to_string(),
                }),
                unmatched_example: None,
            },
        ];
        let mut report = quire_rs::CoverageReport {
            binding_census: census,
            ..Default::default()
        };

        let lines = census_lines(&report);
        assert_eq!(lines.len(), 2, "{lines:?}");
        // The number AND the premise it rests on. `1354/2540 (53%)` travelling
        // without `1344/1513 bound` is the whole defect this renders for.
        assert_eq!(
            lines[0],
            "rust: 1344/1400/1513 bound/tagged/candidates (88% read; 92% authored; \
             forms: rust-trace-attribute, unread tag `tests::misspelled` at \
             crates/a/src/lib.rs:700)"
        );
        // A fully-bound language names no example: there is nothing unread, and
        // pointing at a symbol anyway reads as a finding where there is none.
        assert_eq!(
            lines[1],
            "typescript: 12/12/12 bound/tagged/candidates (100% read; 100% authored; \
             forms: ts-trace-helper)"
        );

        report.metrics = vec![
            Metric::measured("coverage.backed", "matrix row", "m", 1354, 2540, 2037, 1363),
            Metric::not_computed(
                "coverage.no_symbol_rows",
                "matrix row",
                "m",
                MetricShape::Count,
                "the module declares no vocabulary",
            ),
            // A MEASURED count. The first draft of this test had only the
            // not-computed one, so removing the ratio filter changed nothing
            // and the mutation survived — a fixture that cannot distinguish
            // the two shapes cannot test a rule about them.
            Metric::counted("coverage.dead_tags", "trace tag", "m", 7, 200, 200),
        ];
        let metrics = metric_lines(&report);
        // The ratio is enveloped; the not-computed count is not — it carries
        // its own `because`, and "examined 0" would read as a measurement that
        // ran and found nothing.
        assert_eq!(
            metrics,
            vec!["coverage.backed: over 2540 matrix row(s), examined 2037, matched 1363"]
        );
        assert!(matches!(
            report.metrics[1].measurement,
            Measurement::NotComputed { .. }
        ));
    }

    // TC-812, FR-017-AC-14 (#57): the TSV escaping guard, pinned. The #53
    // measurement found 0/1,107 statements carrying a structural character —
    // which means no corpus fixture can exercise the replacement, and mutating
    // `tsv_cell` to the identity left the whole suite green. A cell carrying
    // all three structural characters must still yield exactly one nine-column
    // record.
    #[test]
    fn tc812_tsv_cells_escape_structural_characters() {
        assert_eq!(tsv_cell("a\tb\nc\rd"), "a b c d");
        assert_eq!(tsv_cell("plain"), "plain");

        let line = tsv_line("kind", ["a\tb", "c\nd", "", "", "", "e\rf", "x,y", "text"]);
        let body = line.strip_suffix('\n').expect("one trailing newline");
        assert_eq!(
            body.split('\t').count(),
            9,
            "hostile cells must not add or remove columns: {body:?}"
        );
        assert!(
            !body.contains('\n') && !body.contains('\r'),
            "hostile cells must not break the one-record-per-line contract: {body:?}"
        );
    }
}
