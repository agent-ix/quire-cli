//! `quire` binary entry point.
//!
//! Dispatches to one of six subcommands: `parse`, `extract`, `lookup`,
//! `edit`, `validate`, `schema`. Every command is a thin wrapper over `quire-rs` —
//! no markdown parsing or structural-validation logic lives in this crate
//! (StR-004).

use clap::{Parser, Subcommand};

use quire_cli::io::{self, exit, DiagnosticsFormat};

mod commands;

#[derive(Parser, Debug)]
#[command(
    name = "quire",
    version,
    about = "Thin CLI over quire-rs (parse, extract, lookup, edit, validate, schema)"
)]
struct Cli {
    /// Diagnostic stream format on stderr.
    #[arg(long, value_name = "FORMAT", default_value = "human", global = true)]
    diagnostics_format: DiagnosticsFormat,

    /// Emit JSON output with pretty-printing where applicable.
    #[arg(long, global = true)]
    pretty: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
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
    /// Emit an archetype's input contract (frontmatter schema + asserts) as JSON.
    Schema(commands::schema::Args),
}

fn main() {
    let cli = Cli::parse();
    let ctx = commands::Ctx {
        diagnostics: cli.diagnostics_format,
        pretty: cli.pretty,
    };
    let result = match cli.command {
        Command::Parse(a) => commands::parse::run(&ctx, a),
        Command::Extract(a) => commands::extract::run(&ctx, a),
        Command::Lookup(a) => commands::lookup::run(&ctx, a),
        Command::Edit(a) => commands::edit::run(&ctx, a),
        Command::Validate(a) => commands::validate::run(&ctx, a),
        Command::Schema(a) => commands::schema::run(&ctx, a),
    };
    match result {
        Ok(()) => std::process::exit(exit::OK),
        Err(e) => {
            // Emit the chain as a single human-readable line (or JSON
            // line) — every command translates upstream errors into
            // anyhow chains, and the leaf message carries the load-
            // bearing identifier.
            let msg = format!("{e:#}");
            io::emit_diagnostic(ctx.diagnostics, "QuireError", &msg);
            std::process::exit(exit::USER_ERROR);
        }
    }
}
