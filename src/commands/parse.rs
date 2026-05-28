//! `quire parse <doc|->`
//!
//! Surfaces `quire_rs::parse_document` (upstream FR-005/006/008).

use anyhow::Context;
use clap::Parser;

use quire_cli::io;

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Document path (or `-` for stdin).
    pub doc: String,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let text = io::read_text(&args.doc).with_context(|| format!("reading '{}'", args.doc))?;
    let doc = quire_rs::parse_document(&text);
    let payload = io::encode_json(&doc, ctx.pretty).context("encoding QuireDocument as JSON")?;
    io::write_primary_stdout(payload.as_bytes()).context("writing parse output")?;
    // Trailing newline keeps line-oriented consumers happy without
    // changing the JSON document.
    io::write_primary_stdout(b"\n").ok();
    Ok(())
}
