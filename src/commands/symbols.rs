//! `quire symbols` — the extracted symbol table, as the engine built it
//! (quire-rs FR-051, `agent-ix/quire-rs#309`).
//!
//! # Why this command exists
//!
//! No surface reported which source symbols the scanner found. `extract` reads
//! *documents*; `coverage` reports what symbols *bound*, which is several
//! transformations downstream. So the only way to answer "how many declarations
//! does the scanner lose?" was to reimplement `src/symbols/python.rs` and diff
//! it against `ast.parse`.
//!
//! That was done three times while sizing `agent-ix/quire-rs#274`, and gave
//! three answers: 386, 490 and 5,263 lost declarations over the same tree. The
//! ports disagree precisely where the original is wrong — **a defect in the
//! scanner cannot be sized by a reimplementation of the scanner.**
//!
//! # The module is optional, and the difference matters
//!
//! Without `--module` this reports what the WALK found: every symbol, its kind,
//! and whether that kind could ever bind a trace id. That is #309's actual ask
//! and it needs no declaration.
//!
//! With one, each record also carries the ids it bound — and that half is a
//! statement about a declaration, since which trace forms exist is module data.
//! Reporting an empty `trace_ids` on every symbol because no module was loaded
//! would read exactly like a repository nobody tagged, which is the conflation
//! this whole programme exists to end. So `bound: false` is stated in the
//! payload rather than left to be inferred from zeroes.

use std::path::Path;

use anyhow::Context;
use clap::Parser;
use quire_cli::io;
use quire_cli::safety;

use crate::commands::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    /// Repository root to walk. Source is read from `<scope>` excluding
    /// `spec/`, the same two-root split `coverage` uses (quire-rs CR-045), so
    /// the two commands report the same paths for the same tree.
    #[arg(long, default_value = ".")]
    pub scope: String,

    /// Module directory supplying the `traceability:` model. Without it the
    /// report says what was EXTRACTED and no record carries a bound id; with
    /// it, each record also carries what it bound.
    #[arg(long)]
    pub module: Option<String>,

    /// Emit JSON on stdout instead of the human summary. The JSON is the
    /// stable interface; the human form may change.
    #[arg(long, conflicts_with = "format")]
    pub json: bool,

    /// Output form: `human` census on stderr (default), `json` payload on
    /// stdout, or `tsv` — one tab-separated record per line, the agent-sized
    /// projection of the same records.
    #[arg(long, value_enum, value_name = "FORMAT")]
    pub format: Option<OutputFormat>,

    /// Report only symbols in this language (`rust`, `python`, `typescript`).
    #[arg(long, value_name = "LANG")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Tsv,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let scope = safety::validate_dir_path("--scope", &args.scope)
        .with_context(|| format!("validating --scope '{}'", args.scope))?;

    // A module is OPTIONAL here, unlike `coverage`, which bails without one.
    // The distinction is deliberate: `coverage` reconciles against a declared
    // model and has nothing to say without it, while "what did the scanner
    // find" is a question about the walk alone.
    let registry = match &args.module {
        Some(_) => Some(super::coverage::load_registry_for(
            ctx,
            &args.module,
            &scope,
        )?),
        None => None,
    };
    let model = registry.as_ref().and_then(|r| r.traceability());

    // The same two-root split and the same declared `source_exclude` globs
    // `coverage` applies, so a symbol missing from one report is missing from
    // both for the same reason. Without a module there are no globs to apply —
    // and that widens the walk, which the payload records rather than leaving
    // a reader to wonder why two runs disagree.
    let empty: Vec<String> = Vec::new();
    let extraction = quire_rs::symbols::extract_tree_scoped(
        &scope,
        &[Path::new(super::DOCUMENT_ROOT_DIR)],
        model.map(|m| &m.source_exclude).unwrap_or(&empty),
    );
    // Diagnostics on stderr in every format. A file the extractor could not
    // read is exactly the thing a symbol-table consumer must not silently miss:
    // it looks identical to a file with no declarations in it.
    for d in &extraction.diagnostics {
        io::emit_diagnostic(
            ctx.diagnostics,
            "SymbolExtraction",
            &format!("{}: {}", d.path, d.reason),
        );
    }

    let graph = model.map(|m| quire_rs::symbols::trace::bind(&extraction, m));
    let mut report = quire_rs::build_symbol_table(&extraction, graph.as_ref());

    if let Some(language) = &args.language {
        report.symbols.retain(|s| &s.language == language);
        report.by_language.retain(|l| &l.language == language);
    }

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
        OutputFormat::Human => emit_human(ctx, &report, model.is_some()),
    }
    Ok(())
}

/// One record per line: path, line, symbol, kind, language, binds, ids.
fn render_tsv(report: &quire_rs::SymbolTableReport) -> String {
    let mut out = String::new();
    for s in &report.symbols {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            s.path,
            s.line,
            s.symbol,
            s.kind,
            s.language,
            s.binds_trace_ids,
            s.trace_ids.join(",")
        ));
    }
    out
}

fn emit_human(ctx: &Ctx, report: &quire_rs::SymbolTableReport, bound: bool) {
    let mut lines = vec![format!(
        "{} symbol(s) over {} file(s)",
        report.symbols.len(),
        report.files
    )];
    for l in &report.by_language {
        // BOTH denominators, always. `binding_kinds` is the population a
        // binding rate is drawn from and `symbols` is not — a repository of
        // containers scores 0% against the wrong one and reads as untagged.
        lines.push(format!(
            "  {:<11} {:>6} symbols  {:>5} files  {:>5} can bind{}",
            l.language,
            l.symbols,
            l.files,
            l.binding_kinds,
            if bound {
                format!("  {:>5} bound", l.bound)
            } else {
                String::new()
            }
        ));
    }
    if !bound {
        // Said out loud, not inferred from a column of zeroes. An unbound run
        // and a repository nobody tagged produce the same empty `trace_ids`,
        // and telling them apart is the whole point of the surface.
        lines.push(
            "  no --module: trace ids were not bound, so an empty `trace_ids` \
             here means NOT ASKED rather than not tagged"
                .to_string(),
        );
    }
    if report.excluded_source_files > 0 {
        lines.push(format!(
            "  {} source file(s) removed by a declared `source_exclude` glob",
            report.excluded_source_files
        ));
    }
    io::emit_diagnostic(ctx.diagnostics, "SymbolTable", &lines.join("\n"));
}
