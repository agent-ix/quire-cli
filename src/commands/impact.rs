//! `quire impact` presents the source-change analysis owned by quire-rs FR-099.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use ix_trace_rs::implements;
use quire_rs::impact::{self, FileChange, ImpactInput, LineRange, SourceSnapshot};
#[cfg(test)]
use quire_rs::Registry;
use quire_rs::Spec;

use crate::commands::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, value_name = "REVISION", required = true)]
    base: String,
    #[arg(long, value_name = "REVISION", required = true)]
    head: String,
    #[arg(long, value_name = "PATH", default_value = ".")]
    repo: PathBuf,
    #[arg(long, value_enum, default_value = "human")]
    format: Format,
    #[arg(long, value_name = "N")]
    caller_depth: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Format {
    Human,
    Json,
}

#[implements("FR-049-AC-1")]
#[implements("FR-049-AC-2")]
#[implements("FR-049-AC-3")]
#[implements("FR-049-AC-4")]
#[implements("FR-049-AC-7")]
#[implements("FR-049-AC-8")]
#[implements("FR-049-CON-1")]
#[implements("FR-049-CON-2")]
#[implements("FR-049-CON-3")]
#[implements("FR-049-CON-4")]
pub fn run(ctx: &Ctx, args: Args) -> Result<()> {
    let repo = std::fs::canonicalize(&args.repo)
        .with_context(|| format!("opening repository {}", args.repo.display()))?;
    let base_id = git(
        &repo,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{}^{{commit}}", args.base),
        ],
    )?;
    let head_id = git(
        &repo,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{}^{{commit}}", args.head),
        ],
    )?;
    if base_id == head_id {
        return Err(anyhow!("base and head resolve to the same commit"));
    }
    let base_tree = tempfile::tempdir()?;
    let head_tree = tempfile::tempdir()?;
    materialize(&repo, &base_id, base_tree.path())?;
    materialize(&repo, &head_id, head_tree.path())?;
    let base = snapshot(base_tree.path(), &base_id)?;
    let head = snapshot(head_tree.path(), &head_id)?;
    let changed = changed_paths(&repo, &base_id, &head_id)?;
    let changes: Vec<FileChange> = changed
        .into_iter()
        .filter(|p| source_file(p))
        .map(|path| {
            let before = base.files.get(&path);
            let after = head.files.get(&path);
            let (base_range, head_range) = match (before, after) {
                (Some(before), Some(after)) => line_range(before, after).unwrap_or((None, None)),
                (Some(before), None) => (Some(full_range(before)), None),
                (None, Some(after)) => (None, Some(full_range(after))),
                (None, None) => (None, None),
            };
            FileChange {
                path,
                base: base_range,
                head: head_range,
            }
        })
        .collect();
    let registry = super::coverage::load_registry_for(ctx, &[], head_tree.path())
        .context("loading head revision traceability model")?;
    let model = super::require_traceability_model(&registry)?;
    let spec = Spec::from_path(&head_tree.path().join("spec"));
    let available: BTreeSet<String> = spec.documents().iter().map(|d| d.id.clone()).collect();
    let unavailable = BTreeSet::new();
    let report = impact::analyze(ImpactInput {
        base: Some(&base),
        head: Some(&head),
        changes: &changes,
        traceability: model,
        available_spec_ids: &available,
        unavailable_spec_ids: &unavailable,
        caller_search_depth: args.caller_depth,
    })?;
    match args.format {
        Format::Json => {
            let mut value = serde_json::to_value(&report)?;
            if let Some(object) = value.as_object_mut() {
                if let Some(v) = object.remove("caller_searches") {
                    object.insert("caller_search".into(), v);
                }
            }
            let output = if ctx.pretty {
                serde_json::to_string_pretty(&value)?
            } else {
                serde_json::to_string(&value)?
            };
            primary!("{output}\n");
        }
        Format::Human => {
            primary_line!(
                "Impact {} -> {}\n",
                report.base_revision,
                report.head_revision
            );
            primary_line!("Selection complete: {}\n", report.selection_complete);
            for symbol in &report.changed_symbols {
                primary_line!(
                    "Changed {:?} {}#{} lines {}-{}\n",
                    symbol.side,
                    symbol.path,
                    symbol.name,
                    symbol.lines.start,
                    symbol.lines.end
                );
            }
            for test in &report.selected_tests {
                primary_line!(
                    "Selected test {}#{} ({}) reasons={:?}\n",
                    test.path,
                    test.name,
                    test.filter,
                    test.reasons
                );
            }
            for related in &report.related_implementations {
                primary_line!(
                    "Related implementation {}#{} via {} reason={:?}\n",
                    related.path,
                    related.name,
                    related.trace_id,
                    related.reason
                );
            }
            for finding in &report.findings {
                primary_line!("Finding: {:?}\n", finding);
            }
            for unmapped in &report.unmapped_changes {
                primary_line!(
                    "Unmapped {} {:?} lines {}-{}\n",
                    unmapped.path,
                    unmapped.side,
                    unmapped.lines.start,
                    unmapped.lines.end
                );
            }
            for caller in &report.caller_searches {
                primary_line!("Caller search: {:?}\n", caller);
            }
        }
    }
    Ok(())
}

#[implements("FR-049-AC-8")]
fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .context("running local git command")?;
    if !output.status.success() {
        return Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

#[implements("FR-049-AC-1")]
#[implements("FR-049-AC-4")]
#[implements("FR-049-CON-4")]
fn snapshot(root: &Path, revision: &str) -> Result<SourceSnapshot> {
    let mut files = BTreeMap::new();
    read_source_files(root, root, &mut files)?;
    Ok(SourceSnapshot {
        revision: revision.into(),
        files,
    })
}

#[implements("FR-049-AC-8")]
#[implements("FR-049-CON-4")]
fn read_source_files(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            read_source_files(root, &path, files)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            if source_file(&relative) {
                if let Ok(text) = std::fs::read_to_string(path) {
                    files.insert(relative, text);
                }
            }
        }
    }
    Ok(())
}

#[implements("FR-049-AC-1")]
fn changed_paths(repo: &Path, base: &str, head: &str) -> Result<Vec<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["diff", "--name-only", "-z", base, head])
        .output()?;
    if !out.status.success() {
        return Err(anyhow!("could not diff requested revisions"));
    }
    out.stdout
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8(p.to_vec()).map_err(Into::into))
        .collect()
}

#[implements("FR-049-AC-4")]
fn source_file(path: &str) -> bool {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    matches!(
        ext,
        "rs" | "c"
            | "h"
            | "cc"
            | "cpp"
            | "hpp"
            | "cs"
            | "go"
            | "java"
            | "js"
            | "jsx"
            | "mjs"
            | "ts"
            | "tsx"
            | "py"
            | "rb"
            | "swift"
            | "kt"
            | "scala"
    )
}

#[implements("FR-049-AC-1")]
fn full_range(text: &str) -> LineRange {
    let n = text.lines().count().max(1);
    LineRange { start: 1, end: n }
}

#[implements("FR-049-AC-1")]
fn line_range(before: &str, after: &str) -> Option<(Option<LineRange>, Option<LineRange>)> {
    use gix_imara_diff::{Algorithm, Diff, InternedInput};
    let input = InternedInput::new(before, after);
    let diff = Diff::compute(Algorithm::Histogram, &input);
    let hunks: Vec<_> = diff.hunks().collect();
    let first = hunks.first()?;
    let last = hunks.last().unwrap_or(first);
    let range = |r: std::ops::Range<usize>| {
        (r.start < r.end).then_some(LineRange {
            start: r.start + 1,
            end: r.end,
        })
    };
    Some((
        range(first.before.start as usize..last.before.end as usize),
        range(first.after.start as usize..last.after.end as usize),
    ))
}

#[implements("FR-049-CON-4")]
fn materialize(repo: &Path, revision: &str, root: &Path) -> Result<()> {
    let mut archive = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["archive", revision])
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let mut child = Command::new("tar")
        .arg("-x")
        .arg("-C")
        .arg(root)
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    let mut archive_stdout = archive.stdout.take().unwrap();
    let mut child_stdin = child.stdin.take().unwrap();
    std::io::copy(&mut archive_stdout, &mut child_stdin)?;
    drop(child_stdin);
    if !archive.wait()?.success() || !child.wait()?.success() {
        return Err(anyhow!("could not materialize temporary head snapshot"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-049-AC-1")]
    fn source_extensions_are_classified_without_treating_docs_as_source() {
        assert!(source_file("src/lib.rs"));
        assert!(source_file("app/main.ts"));
        assert!(!source_file("spec/functional/FR-049.md"));
        assert!(!source_file("Cargo.toml"));
    }

    #[test]
    #[trace("FR-049-AC-1")]
    fn changed_line_range_spans_first_through_last_hunk() {
        let (base, head) = line_range("a\nb\nc\nd\n", "a\nB\nc\nD\n").unwrap();
        assert_eq!(base, Some(LineRange { start: 2, end: 4 }));
        assert_eq!(head, Some(LineRange { start: 2, end: 4 }));
    }

    #[test]
    #[trace("FR-049-AC-1")]
    fn deletion_only_hunk_has_no_range_on_the_unchanged_side() {
        let (base, head) = line_range("a\nb\nc\n", "a\nc\n").unwrap();
        assert_eq!(base, Some(LineRange { start: 2, end: 2 }));
        assert_eq!(head, None);
    }

    #[test]
    #[trace("FR-049-AC-3")]
    fn missing_traceability_model_is_not_replaced_with_an_empty_model() {
        let root = tempfile::tempdir().unwrap();
        let registry = Registry::load_module(root.path()).unwrap();
        let error = super::super::require_traceability_model(&registry).unwrap_err();
        assert!(error.to_string().contains("no module in scope declares"));
    }
}
