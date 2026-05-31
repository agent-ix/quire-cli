//! `quire lookup <doc|->`
//!
//! Surfaces `quire_rs::parse_document` plus the `quire-rs` query
//! surface for heading lookup. ID and block-id lookup are direct walks
//! over the parsed `QuireSection` tree.

use anyhow::{anyhow, bail, Context};
use clap::Parser;

use quire_cli::io;
use quire_rs::{QuireDocument, QuireSection};

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Document path (or `-` for stdin).
    pub doc: String,

    /// Heading text to locate.
    #[arg(long, value_name = "TEXT", conflicts_with_all = ["id", "block_id"])]
    pub heading: Option<String>,

    /// Parser-derived section id, for example `behavior-L4`.
    #[arg(long, value_name = "ID", conflicts_with_all = ["heading", "block_id"])]
    pub id: Option<String>,

    /// Stable Pandoc block id from a heading attribute, without `{#...}`.
    #[arg(long, value_name = "BLOCK_ID", conflicts_with_all = ["heading", "id"])]
    pub block_id: Option<String>,

    /// Constrain heading lookup to one ATX heading level.
    #[arg(long, value_name = "1..6", requires = "heading", value_parser = clap::value_parser!(u8).range(1..=6))]
    pub level: Option<u8>,

    /// Emit only the selected section content.
    #[arg(long)]
    pub content: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let text = io::read_text(&args.doc).with_context(|| format!("reading '{}'", args.doc))?;
    let doc = quire_rs::parse_document(&text);
    let section = select_section(&doc, &args)?;

    if args.content {
        io::write_primary_stdout(section.content.as_bytes()).context("writing lookup content")?;
    } else {
        let payload = io::encode_json(section, ctx.pretty).context("encoding lookup section")?;
        io::write_primary_stdout(payload.as_bytes()).context("writing lookup output")?;
        io::write_primary_stdout(b"\n").ok();
    }
    Ok(())
}

fn select_section<'d>(doc: &'d QuireDocument, args: &Args) -> anyhow::Result<&'d QuireSection> {
    let selector_count = [
        args.heading.is_some(),
        args.id.is_some(),
        args.block_id.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if selector_count != 1 {
        bail!("lookup requires exactly one of --heading, --id, or --block-id");
    }

    if let Some(heading) = args.heading.as_deref() {
        let found = if let Some(level) = args.level {
            find_by_heading_at_level(doc, heading, level)
        } else {
            quire_rs::section(doc, heading)
        };
        return found.ok_or_else(|| anyhow!("section not found for --heading '{}'", heading));
    }

    if let Some(id) = args.id.as_deref() {
        return find_by_id(&doc.sections, id)
            .ok_or_else(|| anyhow!("section not found for --id '{}'", id));
    }

    let block_id = args
        .block_id
        .as_deref()
        .expect("selector count already proved block_id is present");
    find_by_block_id(&doc.sections, block_id)
        .ok_or_else(|| anyhow!("section not found for --block-id '{}'", block_id))
}

fn find_by_heading_at_level<'d>(
    doc: &'d QuireDocument,
    heading: &str,
    level: u8,
) -> Option<&'d QuireSection> {
    quire_rs::sections(doc, Some(level))
        .into_iter()
        .find(|candidate| heading_matches(candidate, heading))
}

fn heading_matches(section: &QuireSection, query: &str) -> bool {
    normalize_heading(&section.heading).eq_ignore_ascii_case(&normalize_heading(query))
}

fn normalize_heading(heading: &str) -> String {
    let trimmed = heading.trim();
    let mut chars = trimmed.char_indices().peekable();
    let mut saw_digit = false;
    let mut prefix_end = 0;

    while let Some((idx, c)) = chars.peek().copied() {
        if !c.is_ascii_digit() {
            break;
        }
        saw_digit = true;
        prefix_end = idx + c.len_utf8();
        chars.next();
    }

    while let Some((_, '.')) = chars.peek().copied() {
        let dot_idx = chars.next().expect("peeked dot").0;
        let mut saw_segment_digit = false;
        let mut segment_end = dot_idx + 1;

        while let Some((idx, c)) = chars.peek().copied() {
            if !c.is_ascii_digit() {
                break;
            }
            saw_segment_digit = true;
            segment_end = idx + c.len_utf8();
            chars.next();
        }

        if saw_segment_digit {
            prefix_end = segment_end;
            continue;
        }

        if matches!(chars.peek(), Some((_, c)) if c.is_whitespace()) {
            prefix_end = dot_idx + 1;
            break;
        }

        return trimmed.to_string();
    }

    if !saw_digit {
        return trimmed.to_string();
    }

    match trimmed[prefix_end..].chars().next() {
        Some(c) if c.is_whitespace() => trimmed[prefix_end..].trim_start().to_string(),
        None => trimmed.to_string(),
        _ => trimmed.to_string(),
    }
}

fn find_by_id<'d>(sections: &'d [QuireSection], id: &str) -> Option<&'d QuireSection> {
    for section in sections {
        if section.id == id {
            return Some(section);
        }
        if let Some(found) = find_by_id(&section.children, id) {
            return Some(found);
        }
    }
    None
}

fn find_by_block_id<'d>(sections: &'d [QuireSection], block_id: &str) -> Option<&'d QuireSection> {
    for section in sections {
        if section.block_id.as_deref() == Some(block_id) {
            return Some(section);
        }
        if let Some(found) = find_by_block_id(&section.children, block_id) {
            return Some(found);
        }
    }
    None
}
