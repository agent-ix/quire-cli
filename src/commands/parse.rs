//! `quire parse <doc|->`
//!
//! Surfaces `quire_rs::parse_document` (upstream FR-005/006/008).

use ix_trace_rs::implements;
use anyhow::Context;
use clap::Parser;

use quire_cli::io;

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Document path (or `-` for stdin).
    pub doc: String,
}

#[implements("FR-002-AC-1")]
#[implements("FR-002-AC-2")]
#[implements("FR-002-AC-3")]
#[implements("FR-002-AC-4")]
#[implements("FR-002-AC-5")]
#[implements("FR-006-AC-4")]
#[implements("FR-007-AC-6")]
#[implements("FR-008-AC-1")]
#[implements("FR-008-AC-3")]
#[implements("FR-008-AC-4")]
pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let text = io::read_text(&args.doc)
        .map_err(super::failure::input)
        .with_context(|| format!("reading '{}'", args.doc))?;
    let doc = quire_rs::parse_document(&text);
    let payload =
        ix_cli_kit::json::encode(&doc, ctx.pretty).context("encoding QuireDocument as JSON")?;
    ix_cli_kit::streams::write_primary_stdout(payload.as_bytes())
        .context("writing parse output")?;
    // Trailing newline keeps line-oriented consumers happy without
    // changing the JSON document.
    ix_cli_kit::streams::write_primary_stdout(b"\n")?;
    Ok(())
}
