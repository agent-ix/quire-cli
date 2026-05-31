//! `quire edit <doc|-> (--heading TEXT | --block-id ID) --content <file|-> [--out PATH]`
//!
//! Surfaces `quire_rs::update_section` / `quire_rs::update_block` — byte-exact
//! section/block writeback (FR-012 / upstream FR-022). Frontmatter and every
//! untouched section stay byte-identical; only the addressed region changes.

use std::path::PathBuf;

use anyhow::{bail, Context};
use clap::Parser;

use quire_cli::io;
use quire_cli::safety;

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Document path (or `-` for stdin).
    pub doc: String,

    /// Heading of the section to replace (case-insensitive, number-normalized).
    /// The new content is the section BODY — everything after the heading line,
    /// up to the next heading.
    #[arg(long, value_name = "TEXT", conflicts_with = "block_id")]
    pub heading: Option<String>,

    /// Stable Pandoc block id (`{#blk-id}`) of the block to replace. The new
    /// content is the FULL block rendering — the heading line (with its
    /// `{#blk-id}` attribute) followed by the body.
    #[arg(long, value_name = "BLOCK_ID", conflicts_with = "heading")]
    pub block_id: Option<String>,

    /// New content source: a file path, or `-` for stdin.
    #[arg(long, value_name = "FILE")]
    pub content: String,

    /// Optional output file; default stdout. Pass the input path to edit it in place.
    #[arg(long, value_name = "PATH")]
    pub out: Option<String>,
}

pub fn run(_ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    if args.doc == "-" && args.content == "-" {
        bail!("edit cannot read both <doc> and --content from stdin");
    }

    let out_path: Option<PathBuf> = match args.out.as_deref() {
        Some(o) => {
            Some(safety::validate_out_path(o).with_context(|| format!("validating --out '{o}'"))?)
        }
        None => None,
    };

    let text = io::read_text(&args.doc).with_context(|| format!("reading '{}'", args.doc))?;
    let new_content = io::read_text(&args.content)
        .with_context(|| format!("reading --content '{}'", args.content))?;
    let doc = quire_rs::parse_document(&text);

    let updated = match (args.heading.as_deref(), args.block_id.as_deref()) {
        (Some(heading), None) => quire_rs::update_section(&doc, heading, &new_content)
            .with_context(|| format!("updating section --heading '{heading}'"))?,
        (None, Some(block_id)) => quire_rs::update_block(&doc, block_id, &new_content)
            .with_context(|| format!("updating block --block-id '{block_id}'"))?,
        _ => bail!("edit requires exactly one of --heading or --block-id"),
    };

    io::write_primary(out_path.as_deref(), updated.as_bytes()).context("writing edit output")?;
    Ok(())
}
