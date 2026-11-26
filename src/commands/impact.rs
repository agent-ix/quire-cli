//! `quire impact` presents the source-change analysis owned by quire-rs FR-099.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use ix_trace_rs::implements;
use quire_rs::impact::{
    self, DependencyMoveInput, FileChange, ImpactFinding, ImpactInput, LineRange, SourceSnapshot,
};
#[cfg(test)]
use quire_rs::Registry;
use quire_rs::{Diagnostic, Spec};
use serde_json::Value;

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
    #[arg(long)]
    check_tags: bool,
    #[arg(long, requires = "check_tags")]
    fail_on_tag_findings: bool,
    #[arg(
        long = "dependency-move",
        num_args = 3,
        value_names = ["REPOSITORY_PATH", "OLD_REVISION", "NEW_REVISION"],
        action = clap::ArgAction::Append
    )]
    dependency_move: Vec<String>,
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
#[implements("FR-049-AC-5")]
#[implements("FR-049-AC-6")]
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
    let available = available_spec_ids(&spec);
    let unavailable = unavailable_spec_ids(&spec);
    let mut report = impact::analyze(ImpactInput {
        base: Some(&base),
        head: Some(&head),
        changes: &changes,
        traceability: model,
        available_spec_ids: &available,
        unavailable_spec_ids: &unavailable,
        caller_search_depth: args.caller_depth,
    })?;
    let tag_findings = report.findings.iter().any(is_tag_finding);
    if !args.check_tags {
        report.findings.retain(|finding| !is_tag_finding(finding));
    }
    let dependency_moves = dependency_moves(&args.dependency_move, &base, &head, model)?;
    match args.format {
        Format::Json => {
            let mut value = serde_json::to_value(&report)?;
            if let Some(object) = value.as_object_mut() {
                if let Some(v) = object.remove("caller_searches") {
                    object.insert("caller_search".into(), v);
                }
                if !dependency_moves.is_empty() {
                    object.insert(
                        "dependency_moves".into(),
                        Value::Array(dependency_moves.clone()),
                    );
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
            for movement in &dependency_moves {
                primary_line!(
                    "Dependency move {} {} -> {}\n",
                    movement["repository_path"].as_str().unwrap_or("<unknown>"),
                    movement["old_input_revision"]
                        .as_str()
                        .unwrap_or("<unknown>"),
                    movement["new_input_revision"]
                        .as_str()
                        .unwrap_or("<unknown>")
                );
                for unavailable in movement["unavailable_upstream"]
                    .as_array()
                    .into_iter()
                    .flatten()
                {
                    primary_line!("  Unavailable upstream: {}\n", unavailable);
                }
                for change in movement["upstream_changes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                {
                    primary_line!(
                        "  Upstream change {} items={:?}\n",
                        change["path"].as_str().unwrap_or("<unknown>"),
                        change["items"]
                    );
                }
                for reference in movement["local_references"]
                    .as_array()
                    .into_iter()
                    .flatten()
                {
                    primary_line!(
                        "  Local reference {}#{} upstream_path={} upstream_item={}\n",
                        reference["local_path"].as_str().unwrap_or("<unknown>"),
                        reference["local_symbol"].as_str().unwrap_or("<unknown>"),
                        reference["upstream_path"].as_str().unwrap_or("<unknown>"),
                        reference["upstream_item"].as_str().unwrap_or("<unknown>")
                    );
                }
                for selected in movement["selected_tests"].as_array().into_iter().flatten() {
                    primary_line!(
                        "  Selected test {}#{} via {}#{} upstream_path={} upstream_item={} call_path={:?}\n",
                        selected["test_path"].as_str().unwrap_or("<unknown>"),
                        selected["test_symbol"].as_str().unwrap_or("<unknown>"),
                        selected["local_path"].as_str().unwrap_or("<unknown>"),
                        selected["local_symbol"].as_str().unwrap_or("<unknown>"),
                        selected["upstream_path"].as_str().unwrap_or("<unknown>"),
                        selected["upstream_item"].as_str().unwrap_or("<unknown>"),
                        selected["call_path"]
                    );
                }
                for uncovered in movement["uncovered"].as_array().into_iter().flatten() {
                    primary_line!(
                        "  Uncovered upstream change {} items={:?}\n",
                        uncovered["path"].as_str().unwrap_or("<unknown>"),
                        uncovered["items"]
                    );
                }
            }
        }
    }
    if args.fail_on_tag_findings && tag_findings {
        return Err(anyhow!("requirement link findings were reported"));
    }
    Ok(())
}

#[implements("FR-049-AC-5")]
fn is_tag_finding(finding: &ImpactFinding) -> bool {
    matches!(
        finding,
        ImpactFinding::UnlinkedSymbol { .. }
            | ImpactFinding::UnresolvedId { .. }
            | ImpactFinding::UnavailableSpecification { .. }
            | ImpactFinding::NonBindingMarker { .. }
    )
}

#[implements("FR-049-AC-5")]
fn unavailable_spec_ids(spec: &Spec) -> BTreeSet<String> {
    spec.diagnostics()
        .iter()
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::DocumentUnreadable { path, .. }
            | Diagnostic::DocumentWithoutFrontmatter { path, .. } => {
                path.file_stem()?.to_str().map(str::to_owned)
            }
            _ => None,
        })
        .collect()
}

#[implements("FR-049-AC-5")]
fn available_spec_ids(spec: &Spec) -> BTreeSet<String> {
    fn section_ids(
        document_id: &str,
        section: &quire_rs::ast::QuireSection,
        ids: &mut BTreeSet<String>,
    ) {
        let prefix = format!("{document_id}-AC-");
        for table in quire_rs::query::parse_tables(&section.content) {
            for row in table.rows {
                if let Some(id) = row.first().and_then(|id| id.strip_prefix(&prefix)) {
                    if !id.is_empty() && id.chars().all(|digit| digit.is_ascii_digit()) {
                        ids.insert(format!("{prefix}{id}"));
                    }
                }
            }
        }
        for child in &section.children {
            section_ids(document_id, child, ids);
        }
    }

    let mut ids = BTreeSet::new();
    for document in spec.documents() {
        if document.id.is_empty() {
            continue;
        }
        ids.insert(document.id.clone());
        for section in &document.body().sections {
            section_ids(&document.id, section, &mut ids);
        }
    }
    ids
}

#[implements("FR-049-AC-6")]
fn dependency_moves(
    arguments: &[String],
    base: &SourceSnapshot,
    head: &SourceSnapshot,
    traceability: &quire_rs::traceability::TraceabilityModel,
) -> Result<Vec<Value>> {
    if arguments.len() % 3 != 0 {
        return Err(anyhow!(
            "each --dependency-move needs a path and two revisions"
        ));
    }
    let mut output = Vec::new();
    for group in arguments.chunks_exact(3) {
        let repository_path = PathBuf::from(&group[0]);
        let old_input_revision = &group[1];
        let new_input_revision = &group[2];
        let old = upstream_snapshot(&repository_path, old_input_revision);
        let new = upstream_snapshot(&repository_path, new_input_revision);
        let dependency = old
            .as_ref()
            .and_then(|(_, package)| package.as_deref())
            .or_else(|| new.as_ref().and_then(|(_, package)| package.as_deref()))
            .map(str::to_owned)
            .or_else(|| {
                repository_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            })
            .ok_or_else(|| {
                anyhow!(
                    "cannot determine the dependency name for {}",
                    repository_path.display()
                )
            })?;
        let report = impact::analyze_dependency_move(DependencyMoveInput {
            dependency: &dependency,
            base,
            head,
            upstream_base: old.as_ref().map(|(snapshot, _)| snapshot),
            upstream_head: new.as_ref().map(|(snapshot, _)| snapshot),
            traceability,
        });
        let mut value = serde_json::to_value(report)?;
        if let Some(object) = value.as_object_mut() {
            object.insert(
                "repository_path".into(),
                Value::String(repository_path.to_string_lossy().into_owned()),
            );
            object.insert(
                "old_input_revision".into(),
                Value::String(old_input_revision.clone()),
            );
            object.insert(
                "new_input_revision".into(),
                Value::String(new_input_revision.clone()),
            );
        }
        output.push(value);
    }
    Ok(output)
}

#[implements("FR-049-AC-6")]
fn upstream_snapshot(
    repository: &Path,
    revision: &str,
) -> Option<(SourceSnapshot, Option<String>)> {
    let repository = std::fs::canonicalize(repository).ok()?;
    let revision = git(
        &repository,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ],
    )
    .ok()?;
    let root = tempfile::tempdir().ok()?;
    materialize(&repository, &revision, root.path()).ok()?;
    let manifest = std::fs::read_to_string(root.path().join("Cargo.toml")).ok();
    let package = manifest
        .as_deref()
        .and_then(|text| toml::from_str::<toml::Value>(text).ok())
        .and_then(|manifest| {
            manifest
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
        });
    let snapshot = snapshot_sources(root.path(), &revision, true).ok()?;
    Some((snapshot, package))
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
#[implements("FR-049-AC-6")]
#[implements("FR-049-AC-4")]
#[implements("FR-049-CON-4")]
fn snapshot(root: &Path, revision: &str) -> Result<SourceSnapshot> {
    let mut snapshot = snapshot_sources(root, revision, false)?;
    for name in ["Cargo.lock", "Cargo.toml"] {
        let path = root.join(name);
        if let Ok(text) = std::fs::read_to_string(path) {
            snapshot.files.insert(name.to_owned(), text);
        }
    }
    Ok(snapshot)
}

#[implements("FR-049-AC-6")]
fn snapshot_sources(
    root: &Path,
    revision: &str,
    include_documents: bool,
) -> Result<SourceSnapshot> {
    let mut files = BTreeMap::new();
    read_source_files(root, root, &mut files, include_documents)?;
    Ok(SourceSnapshot {
        revision: revision.into(),
        files,
    })
}

#[implements("FR-049-AC-8")]
#[implements("FR-049-CON-4")]
#[implements("FR-049-AC-6")]
fn read_source_files(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, String>,
    include_documents: bool,
) -> Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            read_source_files(root, &path, files, include_documents)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            if source_file(&relative)
                || (include_documents && (relative.ends_with(".json") || relative.ends_with(".md")))
            {
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
#[implements("FR-049-AC-6")]
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
