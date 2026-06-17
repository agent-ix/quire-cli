//! `quire fix [<DIR>] [--scope <DIR>] [--write]`
//!
//! Surfaces and (with `--write`) applies quire-rs unlinked-reference
//! suggestions (upstream FR-039, ADR 0007): bare artifact-id tokens in a
//! bundle's prose that should be internal relative-path links. All
//! detection, classification, and suggested-link construction live in
//! quire-rs; this command resolves the bundle root, applies path-safety
//! (FR-005), surfaces findings, and — under `--write` — splices the
//! engine-provided suggestions over their byte spans (StR-004 thin
//! boundary: no markdown parsing or detection logic here).

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Context};
use clap::Parser;

use quire_cli::io;
use quire_cli::safety;
use quire_rs::{unlinked_references, Spec, UnlinkedFix, UnlinkedReason, UnlinkedReference};

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Bundle root directory to scan. Defaults to `--scope` when omitted.
    #[arg(value_name = "DIR")]
    pub directory: Option<String>,

    /// Directory used as the bundle root when no positional DIR is given.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub scope: String,

    /// Apply the auto-fixable suggestions in place. Default is a dry-run
    /// that only reports what it would do.
    #[arg(long)]
    pub write: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let root_raw = args.directory.clone().unwrap_or_else(|| args.scope.clone());
    let root = safety::validate_dir_path("directory", &root_raw)
        .with_context(|| format!("validating bundle root '{root_raw}'"))?;

    let spec = Spec::from_path(&root);
    let findings = unlinked_references(&spec);

    // Warn-only findings are advisory: surfaced in both dry-run and
    // --write, never applied, never affect the exit code.
    for f in &findings {
        if let UnlinkedFix::WarnOnly { reason } = &f.fix {
            let why = match reason {
                UnlinkedReason::Unresolved => {
                    "unresolved — if cross-repo, add an ix:// reference manually"
                }
                UnlinkedReason::Ambiguous => "ambiguous — resolves to more than one artifact",
            };
            io::emit_warning(
                ctx.diagnostics,
                &format!("{}: {} ({why})", f.path.display(), f.token),
            );
        }
    }

    let auto: Vec<&UnlinkedReference> = findings
        .iter()
        .filter(|f| matches!(f.fix, UnlinkedFix::AutoFix { .. }))
        .collect();

    if !args.write {
        for f in &auto {
            if let UnlinkedFix::AutoFix { suggested_link } = &f.fix {
                io::emit_diagnostic(
                    ctx.diagnostics,
                    "UnlinkedReference",
                    &format!(
                        "would-fix: {}: {} -> {suggested_link}",
                        f.path.display(),
                        f.token
                    ),
                );
            }
        }
        if !auto.is_empty() {
            bail!(
                "{} unlinked reference(s) can be auto-fixed; re-run with --write",
                auto.len()
            );
        }
        return Ok(());
    }

    // --write: group fixes by file and splice each file's spans in
    // descending start order so earlier offsets stay valid.
    let mut by_file: BTreeMap<PathBuf, Vec<&UnlinkedReference>> = BTreeMap::new();
    for f in &auto {
        by_file.entry(f.path.clone()).or_default().push(f);
    }
    for (path, mut fixes) in by_file {
        let mut text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading '{}'", path.display()))?;
        fixes.sort_by_key(|f| std::cmp::Reverse(f.byte_span.start));
        for f in &fixes {
            if let UnlinkedFix::AutoFix { suggested_link } = &f.fix {
                text.replace_range(f.byte_span.clone(), suggested_link);
            }
        }
        std::fs::write(&path, &text).with_context(|| format!("writing '{}'", path.display()))?;
        io::emit_diagnostic(
            ctx.diagnostics,
            "Fixed",
            &format!("fixed {} reference(s) in {}", fixes.len(), path.display()),
        );
    }
    Ok(())
}
