//! `quire` binary entry point.
//!
//! Dispatches to the `quire <verb>` subcommands: `parse`, `extract`, `lookup`,
//! `edit`, `validate`, `schema`, `lint`, `fix`. Each is a thin wrapper over
//! `quire-rs` — no markdown parsing or structural-validation logic lives in
//! this crate (StR-004). `update` is the one exception: it wraps the
//! package-agnostic `self_update` engine instead of `quire-rs`.

use clap::{Parser, Subcommand};

use quire_cli::io::{self, exit, ColorChoice, DiagnosticsFormat};

mod commands;

#[derive(Parser, Debug)]
#[command(
    name = "quire",
    // #68: both versions, from the ONE place the string is assembled. It was
    // built here and again in `engine`, with nothing binding the two.
    version = quire_cli::engine::VERSION_LINE,
    about = "Thin CLI over quire-rs (documents, contracts, and clause sets)"
)]
struct Cli {
    /// Diagnostic stream format on stderr.
    #[arg(long, value_name = "FORMAT", default_value = "human", global = true)]
    diagnostics_format: DiagnosticsFormat,

    /// Emit JSON output with pretty-printing where applicable.
    #[arg(long, global = true)]
    pretty: bool,

    /// Colorize human diagnostics on stderr: auto (TTY only, honours
    /// NO_COLOR), always, or never.
    #[arg(long, value_name = "WHEN", default_value = "auto", global = true)]
    color: ColorChoice,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Evaluate or compare rights-aware module clause sets.
    Clauses(commands::clauses::Args),
    /// Parse a markdown document to JSON.
    Parse(commands::parse::Args),
    /// Extract structured records + edges from a document.
    Extract(commands::extract::Args),
    /// Look up one parsed section by heading, id, or block id.
    Lookup(commands::lookup::Args),
    /// Edit one section/block of a document in place via byte-exact writeback.
    Edit(commands::edit::Args),
    /// Validate a markdown document against its archetype structure.
    Validate(commands::validate::Args),
    /// AC→TC→code coverage rollup (FR-050). Reports; does not judge.
    Coverage(commands::coverage::Args),
    /// Report the extracted symbol table, as the engine built it (#309).
    Symbols(commands::symbols::Args),
    /// Per-criterion property-shape classification (FR-052). Reports; never a finding.
    Properties(commands::properties::Args),
    /// Report exact CLI/engine source identity and capabilities as JSON.
    Provenance(commands::provenance::Args),
    /// Emit an archetype's input contract (frontmatter schema + asserts) as JSON.
    Schema(commands::schema::Args),
    /// Evaluate the module's advisory lint rules against a document.
    Lint(commands::lint::Args),
    /// Surface (and with --write, apply) internal relative-path link fixes.
    Fix(commands::fix::Args),
    /// Check for and install the latest quire (auto-detects npm vs cargo).
    Update(commands::update::Args),
}

fn main() {
    let cli = Cli::parse();
    let ctx = commands::Ctx {
        diagnostics: io::Diagnostics::new(cli.diagnostics_format, cli.color.resolve()),
        pretty: cli.pretty,
    };
    let result = match cli.command {
        Command::Clauses(a) => commands::clauses::run(&ctx, a),
        Command::Parse(a) => commands::parse::run(&ctx, a),
        Command::Extract(a) => commands::extract::run(&ctx, a),
        Command::Lookup(a) => commands::lookup::run(&ctx, a),
        Command::Edit(a) => commands::edit::run(&ctx, a),
        Command::Coverage(a) => commands::coverage::run(&ctx, a),
        Command::Symbols(a) => commands::symbols::run(&ctx, a),
        Command::Validate(a) => commands::validate::run(&ctx, a),
        Command::Properties(a) => commands::properties::run(&ctx, a),
        Command::Provenance(a) => commands::provenance::run(&ctx, a),
        Command::Schema(a) => commands::schema::run(&ctx, a),
        Command::Lint(a) => commands::lint::run(&ctx, a),
        Command::Fix(a) => commands::fix::run(&ctx, a),
        Command::Update(a) => commands::update::run(&ctx, a),
    };
    match result {
        Ok(()) => std::process::exit(exit::OK),
        Err(e) => {
            // Emit the chain as a single human-readable line (or JSON
            // line) — every command translates upstream errors into
            // anyhow chains, and the leaf message carries the load-
            // bearing identifier.
            let msg = format!("{e:#}");
            // A typed error keeps its own `kind` in the JSON shape rather
            // than collapsing into the generic one. `MissingDocumentRoot`
            // is the first: it is the failure a caller most needs to
            // branch on, and it was a formatted `bail!` string, which is
            // why the tests covering it could only assert
            // `contains("spec")` (agent-ix/quire-rs#113).
            let kind = e
                .downcast_ref::<commands::DocumentRootError>()
                .map(|d| d.kind())
                .unwrap_or("QuireError");
            io::emit_diagnostic(ctx.diagnostics, kind, &msg);
            std::process::exit(exit::USER_ERROR);
        }
    }
}
