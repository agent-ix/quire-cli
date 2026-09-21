//! `quire trace` — structural forward/inverse lookup over the trace-search
//! index (quire-rs FR-077, PLAT-844, `agent-ix/quire-rs#477`).
//!
//! # Why this command exists
//!
//! PLAT-844's own framing: agents hand-roll `grep`/`rg`/`jq` pipelines for
//! "what backs FR-047" and "what does this file verify" — each slow, each
//! subtly different, each able to silently miss a hit or, worse, silently
//! promote a citation to evidence. The engine API alone does not change that;
//! this subcommand is what makes it reachable from the shell.
//!
//! # Claims and citations are never one list
//!
//! **This is the entire value of the tool.** `verifies`/`implements` claims —
//! declared trace-tag forms a module's grammar recognizes — and citations —
//! generic id-shaped text the engine also finds but that binds nothing — stay
//! in separate JSON subtrees (`claims` vs `citations`) and separate,
//! visually distinct human sections, the second headed literally `Citations
//! (N) — NOT verification evidence`. Never a single list with a boolean
//! column: `jq '.claims'` and `jq '.citations'` must never be confusable, and
//! a human skimming stdout must not have to read a flag to tell them apart.
//!
//! # Cross-language honesty, derived not hardcoded
//!
//! Every claim/citation record carries the `language` and `confidence`
//! (`structural`/`line_heuristic`) the engine's own
//! [`quire_rs::symbols::trace_search::language_confidence`] reports for that
//! record's actual extracted language — never a static per-language table in
//! this crate. Which languages are `structural` today is engine data, not CLI
//! policy, and changes out from under this command the moment quire-rs ports
//! another extractor (PLAT-868/PLAT-882 are in flight as this is written).
//! The human-output caveat about `line_heuristic` records is likewise driven
//! by what the *result set* actually contains, never by a compile-time
//! assumption about which languages are ported.
//!
//! # Grouping
//!
//! quire-rs's `reports/2026-09-20-plat844-mentions-census.md` measured 17,399
//! `mentions` across six repos, with `FR-001` alone at 278 hits in each of
//! two repos and the generic id-shaped pattern matching SPDX `AGPL-3.0-...`
//! headers 573 times in one repo. A raw per-citation dump is unusable at that
//! density, so the **human** form groups citations by file once the count
//! exceeds a small inline threshold; `--format json` is never grouped or
//! truncated — it is the stable, complete interface.
//!
//! `--exclude-path` is this command's caller-controlled filter for the
//! measured false-positive class (see `apply_path_exclusion` below for why it
//! is scoped to citations only, never claims).

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{bail, Context};
use clap::Parser;
use serde::Serialize;

use quire_cli::io;
use quire_cli::safety;
use quire_rs::symbols::trace::{ImplementsRelation, Mention, SymbolGraph, VerifiesRelation};
use quire_rs::symbols::trace_search::{self, language_confidence, symbol_language, Query};
use quire_rs::symbols::SymbolExtraction;

use crate::commands::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    /// Repository root to walk. Source is read from `<scope>` excluding
    /// `spec/`, the same two-root split `coverage` uses.
    #[arg(long, default_value = ".")]
    pub scope: String,

    /// Module directory supplying the `traceability:` model. Functionally
    /// required — like `coverage` and unlike `symbols` — because the graph
    /// needs a declared model to compute at all; a missing model fails with
    /// the same error `coverage` produces. Repeatable: the roots are used in
    /// the order given and REPLACE ambient discovery rather than adding to
    /// it. When omitted, resolution falls back to a `manifest.yaml` at the
    /// scope root, then to ambient discovery — the same order `coverage`
    /// uses.
    #[arg(long, value_name = "PATH")]
    pub module: Vec<String>,

    /// Forward query: an exact trace id (`FR-047`, `FR-047-AC-2`, ...).
    /// Exactly one of --id/--symbol/--file is required.
    #[arg(long, value_name = "ID", conflicts_with_all = ["symbol", "file"])]
    pub id: Option<String>,

    /// Inverse query: one symbol as `{path}#{qualified_name}` (for example
    /// `src/foo.rs#Foo::bar`), or a bare unqualified name (no `#`) to match
    /// every symbol sharing it. A bare name matching more than one symbol is
    /// ambiguous: every candidate is listed in `ambiguous_matches`, never
    /// picked for you. Exactly one of --id/--symbol/--file is required.
    #[arg(long, value_name = "REF", conflicts_with_all = ["id", "file"])]
    pub symbol: Option<String>,

    /// Inverse query: every claim/citation recorded against one file.
    /// Exactly one of --id/--symbol/--file is required.
    #[arg(long, value_name = "PATH", conflicts_with_all = ["id", "symbol"])]
    pub file: Option<String>,

    /// With --id, additionally match every id sharing --id's value as an
    /// exact string prefix on a separator boundary: `FR-047` matches
    /// `FR-047-AC-1` but never `FR-0470`. The engine matches ids exactly and
    /// deliberately knows nothing of FR/AC/TC hierarchy (quire-rs
    /// `traceability.rs` states this outright) — this is a caller-requested,
    /// CLI-side string match layered on top of the engine's exact query, not
    /// an engine capability.
    #[arg(long, requires = "id")]
    pub prefix: bool,

    /// Drop citations (never claims) whose path matches this glob. A
    /// caller-controlled escape hatch for the measured false-positive class
    /// (PLAT-844's census: the generic id-shaped pattern matches SPDX
    /// license headers, 573 hits for `AGPL-3` in one repo) — deliberately
    /// NOT filtered in the engine, because the engine has no vocabulary to
    /// tell a license header from a real citation for every repo. The
    /// caller, who knows their own tree's noisy paths (fixtures, generated
    /// files, vendored license headers), can. Repeatable.
    #[arg(long = "exclude-path", value_name = "GLOB")]
    pub exclude_path: Vec<String>,

    /// Emit JSON on stdout instead of the human summary. Equivalent to
    /// `--format json`.
    #[arg(long, conflicts_with = "format")]
    pub json: bool,

    /// Output form: `human` (claims/citations sections on stdout, caveats
    /// and notices on stderr — default), `json` (the complete, ungrouped
    /// `TraceReport` payload on stdout — the stable interface), or `tsv`
    /// (one tab-separated record per line on stdout).
    #[arg(long, value_enum, value_name = "FORMAT")]
    pub format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Tsv,
}

/// What was asked, echoed back so a JSON consumer never has to re-derive it
/// from argv. `matched_ids` is populated only under `--prefix`: it is the set
/// of concrete ids the prefix actually matched, present so a caller can see
/// exactly what got unioned into one result.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum QueryDescription {
    Id {
        id: String,
        prefix: bool,
        matched_ids: Vec<String>,
    },
    Symbol {
        path: String,
        qualified_name: String,
    },
    SymbolName {
        name: String,
    },
    File {
        path: String,
    },
}

/// A `verifies` claim: a test/benchmark/fuzz symbol claiming the queried
/// id(s) as evidence.
#[derive(Debug, Clone, Serialize)]
struct VerifiesRecord {
    trace_id: String,
    symbol: String,
    path: String,
    line: usize,
    form: String,
    provenance: &'static str,
    /// The claiming symbol's actual extracted language, joined back through
    /// `symbol_language` — `None` only if the join itself cannot find the
    /// symbol (which should not happen against the same extraction the graph
    /// was bound from; reported honestly as absent rather than guessed).
    language: Option<String>,
    /// `structural`/`line_heuristic`, from `language_confidence(language)` —
    /// never a static per-language table in this crate. `None` exactly when
    /// `language` is `None`.
    confidence: Option<&'static str>,
}

/// An `implements` claim: production code claiming the queried id(s) as
/// scope. Never evidence (CR-061) — reported for the same reason it is
/// reported anywhere, so a reader can see it without re-deriving it.
#[derive(Debug, Clone, Serialize)]
struct ImplementsRecord {
    trace_id: String,
    symbol: String,
    path: String,
    form: String,
    language: Option<String>,
    confidence: Option<&'static str>,
}

/// Both claim kinds under one JSON key (`claims`), each in its own typed
/// array — mirrors the engine's own `SearchResult` shape rather than
/// collapsing two differently-shaped records into one struct with optional
/// fields.
#[derive(Debug, Clone, Serialize, Default)]
struct ClaimsSection {
    verifies: Vec<VerifiesRecord>,
    implements: Vec<ImplementsRecord>,
}

/// A non-claim citation — near-miss tag, orphaned legacy tag, or plain
/// id-shaped text in a comment/docstring/string literal. **Never** evidence;
/// the second human section is headed to say so in as many words.
#[derive(Debug, Clone, Serialize)]
struct CitationRecord {
    trace_id: String,
    path: String,
    symbol: Option<String>,
    kind: Option<&'static str>,
    bucket: &'static str,
    line: usize,
    excerpt: String,
    language: String,
    confidence: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct TraceReport {
    query: QueryDescription,
    /// `false` iff `claims` and `citations` are both empty and
    /// `ambiguous_matches` is empty too — a query that matched nothing still
    /// returns this full shape (FR-077-AC-3), never an empty `{}`.
    resolved: bool,
    claims: ClaimsSection,
    citations: Vec<CitationRecord>,
    /// `path#qualified_name` for every candidate, populated only when a bare
    /// `--symbol` name matched more than one symbol. Never a silent pick.
    ambiguous_matches: Vec<String>,
    /// How many citations `--exclude-path` removed. Always present (0 when
    /// the flag was not used) so a caller can see filtering happened without
    /// comparing two runs.
    citations_excluded_by_path_filter: usize,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let selector_count = [
        args.id.is_some(),
        args.symbol.is_some(),
        args.file.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if selector_count != 1 {
        bail!("trace requires exactly one of --id, --symbol, or --file");
    }

    let exclude_globs = compile_exclude_globs(&args.exclude_path)?;

    let scope = safety::validate_dir_path("--scope", &args.scope)
        .with_context(|| format!("validating --scope '{}'", args.scope))?;

    // FR-077 needs a declared traceability model to compute at all — unlike
    // `symbols`, which reports the walk with no declaration. Same resolution
    // `coverage` uses, and the identical refusal via the shared
    // `require_traceability_model` helper, so the two commands cannot
    // disagree about which module is in scope, or drift apart on the
    // refusal's wording, for one invocation.
    let registry = super::coverage::load_registry_for(ctx, &args.module, &scope)?;
    let model = super::require_traceability_model(&registry)?;

    let extraction = quire_rs::symbols::extract_tree_scoped(
        &scope,
        &[Path::new(super::DOCUMENT_ROOT_DIR)],
        &model.source_exclude,
    );
    for d in &extraction.diagnostics {
        io::emit_diagnostic(
            ctx.diagnostics,
            "SymbolExtraction",
            &format!("{}: {}", d.path, d.reason),
        );
    }

    let graph = quire_rs::symbols::trace::bind(&extraction, model);

    let (query_desc, mut result) = run_query(&args, &graph, &extraction);

    let before = result.citations.len();
    result
        .citations
        .retain(|m| !path_excluded(&m.path, &exclude_globs));
    let citations_excluded_by_path_filter = before - result.citations.len();

    // `result.resolved` is the engine's PRE-filter verdict; `--exclude-path`
    // runs after it and can empty `citations` entirely, which must flip
    // `resolved` to `false` too — otherwise a fully-filtered result reports
    // itself resolved with nothing in it (review finding #3). Recomputed
    // rather than left as the engine's answer, matching the same rule
    // `trace_search::finish` uses (FR-077-AC-3) over the POST-filter state.
    let resolved = !(result.verifies.is_empty()
        && result.implements.is_empty()
        && result.citations.is_empty()
        && result.ambiguous_matches.is_empty());

    let report = TraceReport {
        query: query_desc,
        resolved,
        claims: ClaimsSection {
            verifies: result
                .verifies
                .iter()
                .map(|v| verifies_record(v, &extraction))
                .collect(),
            implements: result
                .implements
                .iter()
                .map(|i| implements_record(i, &extraction))
                .collect(),
        },
        citations: result.citations.iter().map(citation_record).collect(),
        ambiguous_matches: result.ambiguous_matches,
        citations_excluded_by_path_filter,
    };

    let format = match args.format {
        Some(f) => f,
        None if args.json => OutputFormat::Json,
        None => OutputFormat::Human,
    };
    match format {
        OutputFormat::Json => println!(
            "{}",
            io::encode_json(&quire_cli::engine::attach(&report), ctx.pretty)?
        ),
        OutputFormat::Tsv => print!("{}", render_tsv(&report)),
        OutputFormat::Human => emit_human(ctx, &report),
    }
    Ok(())
}

/// Build the engine [`Query`] the CLI arguments describe, run it, and return
/// both the query as the CLI understood it (for the echoed `query` field —
/// deliberately not read back off `result.query`, which the engine may
/// rewrite internally, e.g. a resolved bare-name query) and the raw
/// [`trace_search::SearchResult`].
fn run_query(
    args: &Args,
    graph: &SymbolGraph,
    extraction: &SymbolExtraction,
) -> (QueryDescription, trace_search::SearchResult) {
    if let Some(id) = &args.id {
        if args.prefix {
            return search_by_prefix(id, graph, extraction);
        }
        let result = trace_search::search(graph, extraction, &Query::Id(id.clone()));
        return (
            QueryDescription::Id {
                id: id.clone(),
                prefix: false,
                matched_ids: Vec::new(),
            },
            result,
        );
    }
    if let Some(symbol) = &args.symbol {
        let query = parse_symbol_query(symbol);
        let result = trace_search::search(graph, extraction, &query);
        return (describe_query(&query), result);
    }
    let file = args
        .file
        .as_deref()
        .expect("selector count already proved --file is present");
    let result = trace_search::search(graph, extraction, &Query::File(file.to_string()));
    (
        QueryDescription::File {
            path: file.to_string(),
        },
        result,
    )
}

/// `{path}#{qualified_name}` splits into an exact [`Query::Symbol`]; anything
/// else (no `#`, or an empty side) is a bare name and becomes
/// [`Query::SymbolName`], which the engine resolves — one match, an
/// unambiguous [`Query::Symbol`]-equivalent result; more than one, every
/// candidate listed in `ambiguous_matches` and nothing picked.
fn parse_symbol_query(raw: &str) -> Query {
    match raw.split_once('#') {
        Some((path, qualified_name)) if !path.is_empty() && !qualified_name.is_empty() => {
            Query::Symbol {
                path: path.to_string(),
                qualified_name: qualified_name.to_string(),
            }
        }
        _ => Query::SymbolName(raw.to_string()),
    }
}

fn describe_query(query: &Query) -> QueryDescription {
    match query {
        Query::Id(id) => QueryDescription::Id {
            id: id.clone(),
            prefix: false,
            matched_ids: Vec::new(),
        },
        Query::Symbol {
            path,
            qualified_name,
        } => QueryDescription::Symbol {
            path: path.clone(),
            qualified_name: qualified_name.clone(),
        },
        Query::SymbolName(name) => QueryDescription::SymbolName { name: name.clone() },
        Query::File(path) => QueryDescription::File { path: path.clone() },
    }
}

/// `--id --prefix`: union every distinct id in the graph whose
/// [`id_segments`] start with `prefix`'s segments, running the engine's own
/// exact `Query::Id` search once per **normalized-equivalence class** of
/// matched id and concatenating.
///
/// Two raw spellings that fold to the same normalized id (e.g. `FR-047` and
/// `FR_047`) must be searched only **once** between them: the engine's own
/// `Query::Id` search matches by normalized id (`normalized_trace_id`), so
/// searching each raw spelling separately would return the identical set of
/// relations twice — a real double-count this function used to have (review
/// finding H2). `matched_ids` in the returned [`QueryDescription`] still
/// lists every distinct RAW spelling that matched, for transparency; only the
/// search itself is deduplicated by normalized form.
fn search_by_prefix(
    prefix: &str,
    graph: &SymbolGraph,
    extraction: &SymbolExtraction,
) -> (QueryDescription, trace_search::SearchResult) {
    let mut ids: BTreeSet<&str> = BTreeSet::new();
    ids.extend(graph.verifies.iter().map(|v| v.trace_id.as_str()));
    ids.extend(graph.implements.iter().map(|i| i.trace_id.as_str()));
    ids.extend(graph.mentions.iter().map(|m| m.trace_id.as_str()));
    let matched_ids: Vec<String> = ids
        .into_iter()
        .filter(|id| prefix_matches(id, prefix))
        .map(String::from)
        .collect();

    let mut verifies = Vec::new();
    let mut implements = Vec::new();
    let mut citations = Vec::new();
    let mut searched_normalized: BTreeSet<String> = BTreeSet::new();
    for id in &matched_ids {
        // The one representative per normalized-equivalence class. A second
        // raw spelling that normalizes the same is skipped here, not
        // searched-then-deduplicated after the fact — the engine's search
        // would hand back the exact same rows either way, so running it
        // twice is pure waste as well as the double-count bug.
        if !searched_normalized.insert(normalized_id(id)) {
            continue;
        }
        let r = trace_search::search(graph, extraction, &Query::Id(id.clone()));
        verifies.extend(r.verifies);
        implements.extend(r.implements);
        citations.extend(r.citations);
    }
    let resolved = !(verifies.is_empty() && implements.is_empty() && citations.is_empty());
    let result = trace_search::SearchResult {
        query: Query::Id(prefix.to_string()),
        verifies,
        implements,
        citations,
        ambiguous_matches: Vec::new(),
        resolved,
    };
    (
        QueryDescription::Id {
            id: prefix.to_string(),
            prefix: true,
            matched_ids,
        },
        result,
    )
}

/// Split `value` on runs of non-alphanumeric characters and uppercase each
/// resulting piece, dropping empty pieces (so a leading/trailing separator —
/// `FR-047-` — contributes nothing, and consecutive separators collapse).
///
/// This is character-for-character the same fold quire-rs's own
/// `normalized_trace_id` applies (`src/symbols/trace.rs:871`: filter
/// `is_ascii_alphanumeric`, uppercase) — that function is `pub(crate)` and
/// unreachable from here, so this duplicates its per-character rule rather
/// than sharing it (asked for during review; requesting the export be made
/// `pub` upstream is the preferred long-term fix and does not, by itself,
/// replace this function even if granted — see below). It is deliberately
/// NOT a drop-in reimplementation of `normalized_trace_id` itself: that
/// function concatenates every alphanumeric character into one string,
/// discarding exactly the segment boundaries `--prefix` needs to tell
/// `FR-047` (must match `FR-047-AC-1`) apart from `FR-0470` (must not). No
/// function in the engine currently preserves those boundaries — boundary-
/// aware prefix matching is CLI-only logic by design (quire-rs
/// `traceability.rs`: the engine knows nothing of FR/AC/TC hierarchy).
///
/// This duplication is a liability by itself: if quire-rs ever changes what
/// `normalized_trace_id` folds (a new character class, different case
/// handling), this copy keeps the OLD rule and `--prefix` silently diverges
/// from exact `--id` again, with every test in this file staying green
/// because they all check this copy against itself. The integration test
/// `prefix_fold_tracks_the_engines_own_normalization_across_spellings` in
/// `tests/cli_trace.rs` is the guard against that: it measures the engine's
/// fold behaviorally (through exact `--id`, since `normalized_trace_id`
/// itself is `pub(crate)` and unreachable) and this crate's fold
/// behaviorally (through `--prefix`) over the same spelling table, so a
/// drift between the two shows up as a named, failing assertion instead of
/// silence. Do not replace that test with a doc comment restating this
/// property — an assertion the code does not enforce is what this note
/// itself would become.
fn id_segments(value: &str) -> Vec<String> {
    value
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_uppercase)
        .collect()
}

/// The engine's own normalized id, reproduced by concatenating
/// [`id_segments`] — used only to tell whether two RAW spellings are the
/// same id as far as `Query::Id`'s own match is concerned (H2's dedup key),
/// never to decide `--prefix` boundaries (which need the segments, not the
/// concatenation — see [`id_segments`]).
fn normalized_id(value: &str) -> String {
    id_segments(value).concat()
}

/// Segment-boundary prefix match, fold-consistent with the engine's own
/// `normalized_trace_id` equality: `prefix` matches `candidate` when
/// `candidate`'s [`id_segments`] start with `prefix`'s.
///
/// `FR-047` (segments `["FR","047"]`) matches `FR-047-AC-1`
/// (`["FR","047","AC","1"]`, a genuine extra segment) and `FR_047_AC_9`
/// (separator-agnostic) and is case-insensitive (`fr-047` folds the same as
/// `FR-047`) — all three were confirmed broken under the previous raw,
/// case-sensitive `strip_prefix` (review finding H1). `FR-047` still does
/// NOT match `FR-0470` (segments `["FR","0470"]`; `"0470" != "047"` as whole
/// segments, not merely as a string-prefix check within one segment) — the
/// property this flag exists to guarantee.
fn prefix_matches(candidate: &str, prefix: &str) -> bool {
    let prefix_segments = id_segments(prefix);
    if prefix_segments.is_empty() {
        return false;
    }
    let candidate_segments = id_segments(candidate);
    candidate_segments.len() >= prefix_segments.len()
        && candidate_segments[..prefix_segments.len()] == prefix_segments[..]
}

fn verifies_record(v: &VerifiesRelation, extraction: &SymbolExtraction) -> VerifiesRecord {
    let language = symbol_language(extraction, &v.symbol_id).map(str::to_string);
    let confidence = language.as_deref().map(|l| language_confidence(l).as_str());
    VerifiesRecord {
        trace_id: v.trace_id.clone(),
        symbol: v.symbol.clone(),
        path: v.path.clone(),
        line: v.line,
        form: v.form.clone(),
        provenance: v.provenance.as_str(),
        language,
        confidence,
    }
}

fn implements_record(i: &ImplementsRelation, extraction: &SymbolExtraction) -> ImplementsRecord {
    let language = symbol_language(extraction, &i.symbol_id).map(str::to_string);
    let confidence = language.as_deref().map(|l| language_confidence(l).as_str());
    ImplementsRecord {
        trace_id: i.trace_id.clone(),
        symbol: i.symbol.clone(),
        path: i.path.clone(),
        form: i.form.clone(),
        language,
        confidence,
    }
}

fn citation_record(m: &Mention) -> CitationRecord {
    CitationRecord {
        trace_id: m.trace_id.clone(),
        path: m.path.clone(),
        symbol: m.symbol.clone(),
        kind: m.kind,
        bucket: m.bucket.as_str(),
        line: m.line,
        excerpt: m.excerpt.clone(),
        confidence: language_confidence(&m.language).as_str(),
        language: m.language.clone(),
    }
}

fn compile_exclude_globs(patterns: &[String]) -> anyhow::Result<Vec<glob::Pattern>> {
    patterns
        .iter()
        .map(|p| {
            glob::Pattern::new(p).with_context(|| format!("parsing --exclude-path glob '{p}'"))
        })
        .collect()
}

fn path_excluded(path: &str, globs: &[glob::Pattern]) -> bool {
    globs.iter().any(|g| g.matches(path))
}

/// A result set contains at least one record the engine could not ground in
/// a parsed AST — read off the actual records returned, never a compile-time
/// list of which languages are "still line-scanner-backed": that list goes
/// stale the moment quire-rs ports another extractor.
fn has_line_heuristic_record(report: &TraceReport) -> bool {
    // The one place this literal lives: the engine's own `Confidence` enum,
    // not a copy of its string re-typed here.
    let line_heuristic = trace_search::Confidence::LineHeuristic.as_str();
    report
        .claims
        .verifies
        .iter()
        .any(|v| v.confidence == Some(line_heuristic))
        || report
            .claims
            .implements
            .iter()
            .any(|i| i.confidence == Some(line_heuristic))
        || report
            .citations
            .iter()
            .any(|c| c.confidence == line_heuristic)
}

const CITATION_INLINE_LIMIT: usize = 20;
const CITATION_GROUP_DISPLAY_LIMIT: usize = 20;

fn confidence_suffix(confidence: Option<&str>) -> String {
    match confidence {
        Some(c) => format!(" ({c})"),
        None => " (language unknown)".to_string(),
    }
}

fn emit_human(ctx: &Ctx, report: &TraceReport) {
    io::emit_result(&format!("resolved: {}", report.resolved));

    // `--prefix` unions however many concrete ids matched — the exact thing
    // H1/H2 got wrong, and the exact thing a `--prefix` user most needs to
    // see to trust the result. It was JSON-only before (review finding #12).
    if let QueryDescription::Id {
        prefix: true,
        matched_ids,
        ..
    } = &report.query
    {
        io::emit_result(&format!(
            "matched ids ({}): {}",
            matched_ids.len(),
            if matched_ids.is_empty() {
                "(none)".to_string()
            } else {
                matched_ids.join(", ")
            }
        ));
    }

    let claim_count = report.claims.verifies.len() + report.claims.implements.len();
    io::emit_result(&format!("Claims ({claim_count})"));
    for v in &report.claims.verifies {
        io::emit_result(&format!(
            "  verifies    {} {}:{} {} [{}]{}",
            v.trace_id,
            v.path,
            v.line,
            v.symbol,
            v.form,
            confidence_suffix(v.confidence)
        ));
    }
    for i in &report.claims.implements {
        io::emit_result(&format!(
            "  implements  {} {} {} [{}]{}",
            i.trace_id,
            i.path,
            i.symbol,
            i.form,
            confidence_suffix(i.confidence)
        ));
    }

    io::emit_result("");
    io::emit_result(&format!(
        "Citations ({}) — NOT verification evidence",
        report.citations.len()
    ));
    emit_citations(&report.citations);

    if !report.ambiguous_matches.is_empty() {
        io::emit_result("");
        io::emit_result(&format!(
            "Ambiguous matches ({}) — every candidate, none picked:",
            report.ambiguous_matches.len()
        ));
        for m in &report.ambiguous_matches {
            io::emit_result(&format!("  {m}"));
        }
    }

    if report.citations_excluded_by_path_filter > 0 {
        io::emit_diagnostic(
            ctx.diagnostics,
            "TraceCitationsExcluded",
            &format!(
                "{} citation(s) dropped by --exclude-path",
                report.citations_excluded_by_path_filter
            ),
        );
    }

    if has_line_heuristic_record(report) {
        io::emit_diagnostic(
            ctx.diagnostics,
            "TraceConfidence",
            "this result set includes line_heuristic record(s): matched by a \
             pre-structural line/indentation scanner, not a parsed syntax tree — \
             treat with the same caution as a grep hit, not as AST-grounded evidence",
        );
    }
}

fn emit_citations(citations: &[CitationRecord]) {
    for line in citation_lines(citations) {
        io::emit_result(&line);
    }
}

/// Full detail inline under [`CITATION_INLINE_LIMIT`]; above it, grouped by
/// file with counts (PLAT-844's census: `FR-001` alone runs 278 hits in a
/// single repo, `AGPL-3` 573 — a raw per-line dump at that density is not
/// usable). `--format json` is never grouped or truncated.
///
/// A pure function so the grouping/truncation behavior can be asserted
/// directly rather than only through a captured-stdout integration test.
fn citation_lines(citations: &[CitationRecord]) -> Vec<String> {
    if citations.is_empty() {
        return vec!["  (none)".to_string()];
    }
    if citations.len() <= CITATION_INLINE_LIMIT {
        return citations
            .iter()
            .map(|c| {
                format!(
                    "  {} {}:{} {} [{}/{}]",
                    c.trace_id,
                    c.path,
                    c.line,
                    c.symbol.as_deref().unwrap_or("(file scope)"),
                    c.bucket,
                    c.confidence,
                )
            })
            .collect();
    }
    let mut by_path: BTreeMap<&str, usize> = BTreeMap::new();
    for c in citations {
        *by_path.entry(c.path.as_str()).or_insert(0) += 1;
    }
    let mut groups: Vec<(&str, usize)> = by_path.into_iter().collect();
    groups.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let mut lines = vec![format!(
        "  grouped by file ({} file(s)) — use --format json for every citation",
        groups.len()
    )];
    lines.extend(
        groups
            .iter()
            .take(CITATION_GROUP_DISPLAY_LIMIT)
            .map(|(path, count)| format!("  {path}: {count} citation(s)")),
    );
    if groups.len() > CITATION_GROUP_DISPLAY_LIMIT {
        lines.push(format!(
            "  ... and {} more file(s)",
            groups.len() - CITATION_GROUP_DISPLAY_LIMIT
        ));
    }
    lines
}

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

fn tsv_line(kind: &str, cells: [&str; 8]) -> String {
    let mut line = String::from(kind);
    for c in cells {
        line.push('\t');
        line.push_str(&tsv_cell(c));
    }
    line.push('\n');
    line
}

/// One record per line: `kind trace_id path line symbol form bucket language
/// confidence`. `kind` is `verifies`/`implements`/`citation`/`ambiguous`.
///
/// `form` and `bucket` are two SEPARATE columns, deliberately — a claim's
/// declared marker-form name (`form`) and a citation's classification
/// (`bucket`, e.g. `mention`/`evidence_near_miss`) are different facts, and
/// overloading one column to carry either depending on `kind` (the first
/// draft's shape) made a citation row's `form` cell silently contain bucket
/// data with no column name saying so (review finding #4). A claim row
/// leaves `bucket` empty; a citation row leaves `form` empty.
///
/// An `ambiguous` row is emitted per `ambiguous_matches` entry (`path` and
/// `symbol` split from its `path#qualified_name` form) so a TSV consumer can
/// tell an ambiguous query apart from a zero-match one — both used to render
/// as the bare header line with nothing to distinguish them (review finding
/// #4).
fn render_tsv(report: &TraceReport) -> String {
    let mut out =
        String::from("kind\ttrace_id\tpath\tline\tsymbol\tform\tbucket\tlanguage\tconfidence\n");
    for v in &report.claims.verifies {
        out.push_str(&tsv_line(
            "verifies",
            [
                &v.trace_id,
                &v.path,
                &v.line.to_string(),
                &v.symbol,
                &v.form,
                "",
                v.language.as_deref().unwrap_or(""),
                v.confidence.unwrap_or(""),
            ],
        ));
    }
    for i in &report.claims.implements {
        out.push_str(&tsv_line(
            "implements",
            [
                &i.trace_id,
                &i.path,
                "",
                &i.symbol,
                &i.form,
                "",
                i.language.as_deref().unwrap_or(""),
                i.confidence.unwrap_or(""),
            ],
        ));
    }
    for c in &report.citations {
        out.push_str(&tsv_line(
            "citation",
            [
                &c.trace_id,
                &c.path,
                &c.line.to_string(),
                c.symbol.as_deref().unwrap_or(""),
                "",
                c.bucket,
                &c.language,
                c.confidence,
            ],
        ));
    }
    for m in &report.ambiguous_matches {
        let (path, symbol) = m.split_once('#').unwrap_or((m.as_str(), ""));
        out.push_str(&tsv_line(
            "ambiguous",
            ["", path, "", symbol, "", "", "", ""],
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tc_prefix_matches_a_separator_boundary_not_a_digit_run() {
        // FR-077's own AC-6 hazard, one level up: --prefix must not let
        // `FR-047` match `FR-0470`, only ids that continue past a separator.
        assert!(prefix_matches("FR-047", "FR-047"));
        assert!(prefix_matches("FR-047-AC-1", "FR-047"));
        assert!(!prefix_matches("FR-0470", "FR-047"));
        assert!(!prefix_matches("FR-04", "FR-047"));
        assert!(!prefix_matches("XR-047", "FR-047"));
    }

    // Review finding H1: the previous raw, case-sensitive `strip_prefix`
    // diverged from the engine's own `normalized_trace_id` fold in three
    // reproduced ways. Each of these failed before the `id_segments` rewrite.
    #[test]
    fn tc_prefix_matches_folds_case_like_the_engine() {
        assert!(prefix_matches("FR-047", "fr-047"));
        assert!(prefix_matches("fr-047-ac-1", "FR-047"));
    }

    #[test]
    fn tc_prefix_matches_folds_separator_choice_like_the_engine() {
        // The engine strips ALL non-alphanumeric characters before comparing
        // ids, so `-` and `_` are interchangeable to it; --prefix must agree.
        assert!(prefix_matches("FR_047_AC_9", "FR-047"));
        assert!(prefix_matches("FR-047-AC-9", "FR_047"));
    }

    #[test]
    fn tc_prefix_matches_ignores_a_trailing_separator_on_the_query() {
        // `--id FR-047- --prefix` must behave exactly like `--id FR-047
        // --prefix` — a trailing separator on the CALLER's query contributes
        // no empty trailing segment.
        assert!(prefix_matches("FR-047-AC-1", "FR-047-"));
        assert!(prefix_matches("FR-047", "FR-047-"));
    }

    #[test]
    fn tc_id_segments_splits_on_non_alphanumeric_runs_and_uppercases() {
        assert_eq!(id_segments("FR-047-AC-1"), vec!["FR", "047", "AC", "1"]);
        assert_eq!(id_segments("fr_047"), vec!["FR", "047"]);
        assert_eq!(id_segments("FR-047-"), vec!["FR", "047"]);
        assert_eq!(id_segments(""), Vec::<String>::new());
    }

    #[test]
    fn tc_normalized_id_matches_across_separator_and_case_spellings() {
        assert_eq!(normalized_id("FR-047-AC-1"), normalized_id("fr_047_ac_1"));
        assert_ne!(normalized_id("FR-047"), normalized_id("FR-0470"));
    }

    fn citation(trace_id: &str, path: &str) -> CitationRecord {
        CitationRecord {
            trace_id: trace_id.to_string(),
            path: path.to_string(),
            symbol: Some("some_fn".to_string()),
            kind: Some("TestFunction"),
            bucket: "mention",
            line: 1,
            excerpt: "// see FR-1".to_string(),
            language: "rust".to_string(),
            confidence: "structural",
        }
    }

    #[test]
    fn tc_citations_below_the_threshold_render_inline_not_grouped() {
        let citations = vec![citation("FR-1", "a.rs"), citation("FR-1", "b.rs")];
        let lines = citation_lines(&citations);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].contains("a.rs"), "{lines:?}");
        assert!(lines[1].contains("b.rs"), "{lines:?}");
    }

    #[test]
    fn tc_citations_above_the_threshold_group_by_file_with_counts() {
        // PLAT-844's census: a common id runs into the hundreds of hits in a
        // single repo. A raw per-line dump at that density is unusable, so
        // above the inline threshold the human form must group by path and
        // report a count instead of one line per citation.
        let mut citations = Vec::new();
        for _ in 0..(CITATION_INLINE_LIMIT + 1) {
            citations.push(citation("FR-1", "hot.rs"));
        }
        citations.push(citation("FR-1", "cold.rs"));
        let lines = citation_lines(&citations);
        // One header line plus one line per distinct path (2 here), never one
        // line per citation (CITATION_INLINE_LIMIT + 2).
        assert_eq!(lines.len(), 3, "{lines:?}");
        assert!(lines[0].contains("grouped by file"), "{lines:?}");
        assert!(
            lines[1].contains("hot.rs")
                && lines[1].contains(&(CITATION_INLINE_LIMIT + 1).to_string()),
            "the denser file must sort first with its real count: {lines:?}"
        );
        assert!(
            lines[2].contains("cold.rs") && lines[2].contains(": 1 citation"),
            "{lines:?}"
        );
    }

    #[test]
    fn tc_path_exclusion_glob_matches_repo_relative_paths() {
        let globs = compile_exclude_globs(&["fuzz/**".to_string()]).expect("valid glob");
        assert!(path_excluded("fuzz/fuzz_targets/a.rs", &globs));
        assert!(!path_excluded("src/lib.rs", &globs));
    }

    #[test]
    fn tc_parse_symbol_query_splits_on_hash_else_bare_name() {
        assert_eq!(
            parse_symbol_query("src/foo.rs#Foo::bar"),
            Query::Symbol {
                path: "src/foo.rs".to_string(),
                qualified_name: "Foo::bar".to_string(),
            }
        );
        assert_eq!(
            parse_symbol_query("bare_name"),
            Query::SymbolName("bare_name".to_string())
        );
        // A `#` with an empty side either way falls back to the bare-name
        // form rather than constructing a Symbol query with an empty path or
        // qualified_name, which could never match anything.
        assert_eq!(
            parse_symbol_query("#bar"),
            Query::SymbolName("#bar".to_string())
        );
    }

    // Review finding #8: `tsv_cell`/`tsv_line` were copy-pasted from
    // `coverage.rs` with no equivalent of ITS `tc812` escaping test in this
    // file. Pinned here the same way.
    #[test]
    fn tc_tsv_cells_escape_structural_characters() {
        assert_eq!(tsv_cell("a\tb\nc\rd"), "a b c d");
        assert_eq!(tsv_cell("plain"), "plain");

        let line = tsv_line("kind", ["a\tb", "c\nd", "", "", "", "", "e\rf", "text"]);
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

    fn minimal_report(claims: ClaimsSection, citations: Vec<CitationRecord>) -> TraceReport {
        TraceReport {
            query: QueryDescription::Id {
                id: "FR-1".to_string(),
                prefix: false,
                matched_ids: Vec::new(),
            },
            resolved: true,
            claims,
            citations,
            ambiguous_matches: Vec::new(),
            citations_excluded_by_path_filter: 0,
        }
    }

    // Review finding #4: no test anywhere exercised `render_tsv`, and the
    // format actually shipped a real defect — a citation row's `bucket`
    // silently occupied the `form` column, undocumented. Pinned as two
    // separate, present columns.
    #[test]
    fn tc_render_tsv_keeps_form_and_bucket_as_separate_columns() {
        let report = minimal_report(
            ClaimsSection {
                verifies: vec![VerifiesRecord {
                    trace_id: "FR-1".to_string(),
                    symbol: "tc_verifies".to_string(),
                    path: "src/lib.rs".to_string(),
                    line: 3,
                    form: "rust-trace-attribute".to_string(),
                    provenance: "canonical",
                    language: Some("rust".to_string()),
                    confidence: Some("structural"),
                }],
                implements: Vec::new(),
            },
            vec![citation("FR-1", "src/other.rs")],
        );
        let tsv = render_tsv(&report);
        let mut lines = tsv.lines();
        assert_eq!(
            lines.next(),
            Some("kind\ttrace_id\tpath\tline\tsymbol\tform\tbucket\tlanguage\tconfidence")
        );
        let verifies_line = lines.next().expect("verifies row");
        let cells: Vec<&str> = verifies_line.split('\t').collect();
        assert_eq!(cells[0], "verifies");
        assert_eq!(cells[5], "rust-trace-attribute", "form column: {cells:?}");
        assert_eq!(cells[6], "", "a claim leaves bucket empty: {cells:?}");

        let citation_line = lines.next().expect("citation row");
        let cells: Vec<&str> = citation_line.split('\t').collect();
        assert_eq!(cells[0], "citation");
        assert_eq!(cells[5], "", "a citation leaves form empty: {cells:?}");
        assert_eq!(cells[6], "mention", "bucket column: {cells:?}");
    }

    // Review finding #4: an ambiguous query and a zero-match query used to
    // render byte-identical TSV — the header line and nothing else — so the
    // one property this command exists to guarantee (never silently pick a
    // candidate) was invisible to a TSV consumer.
    #[test]
    fn tc_render_tsv_distinguishes_ambiguous_from_zero_match() {
        let zero_match = minimal_report(ClaimsSection::default(), Vec::new());
        let mut ambiguous = minimal_report(ClaimsSection::default(), Vec::new());
        ambiguous.resolved = true;
        ambiguous.ambiguous_matches = vec![
            "src/a.rs#tests::outer::helper".to_string(),
            "src/b.rs#tests::outer::helper".to_string(),
        ];

        let zero_tsv = render_tsv(&zero_match);
        let ambiguous_tsv = render_tsv(&ambiguous);
        assert_ne!(
            zero_tsv, ambiguous_tsv,
            "an ambiguous result must not render identically to a zero-match one"
        );
        assert_eq!(zero_tsv.lines().count(), 1, "header only: {zero_tsv:?}");
        assert_eq!(
            ambiguous_tsv.lines().count(),
            3,
            "header plus one `ambiguous` row per candidate: {ambiguous_tsv:?}"
        );
        assert!(ambiguous_tsv.contains("ambiguous\t\tsrc/a.rs\t\ttests::outer::helper"));
    }

    // Review finding #11: the "... and N more file(s)" overflow branch in
    // `citation_lines` needs more distinct paths than
    // `CITATION_GROUP_DISPLAY_LIMIT`, and nothing exercised it.
    #[test]
    fn tc_citation_grouping_overflow_reports_the_remainder() {
        let mut citations = Vec::new();
        // One extra citation beyond the inline threshold to force grouping,
        // spread over more distinct paths than the display limit so the
        // overflow line is reached.
        for i in 0..(CITATION_GROUP_DISPLAY_LIMIT + 5) {
            citations.push(citation("FR-1", &format!("file{i}.rs")));
        }
        citations.push(citation("FR-1", "file0.rs"));
        let lines = citation_lines(&citations);
        let last = lines.last().expect("at least a header line");
        assert!(
            last.contains("more file(s)"),
            "expected an overflow line: {lines:?}"
        );
        assert!(
            last.contains('5'),
            "5 files beyond the display limit: {lines:?}"
        );
    }
}
