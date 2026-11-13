//! The design column of `quire matrix` (FR-043): how one `design_matrix` entry
//! reads as a cell, a gap line, a TSV field and a `--strict` failure.
//!
//! Nothing here knows what a requirement or a design element is. A block, its
//! `check` token, an element's `kind` and the per-criterion `design_elements`
//! all come from quire-rs; this file only prints them.

use quire_rs::{DesignBlock, DesignElement, DesignMatrixCriterion, DesignMatrixEntry, RowState};

/// The Design cell of an entry with no `design` key (out of scope).
pub(super) const NOT_APPLICABLE: &str = "n/a";
/// The Design cell with no element and nothing to append.
pub(super) const NONE: &str = "(none)";

/// `design_state` of an entry with an `uncovered` block.
const UNCOVERED: &str = "uncovered";
/// `design_state` of an entry whose blocks are all covered.
const COVERED: &str = "covered";

/// `<repo>/<id>` when the document has a repo, `<id>` otherwise.
fn qualified(repo: Option<&str>, id: &str) -> String {
    match repo {
        Some(repo) => format!("{repo}/{id}"),
        None => id.to_string(),
    }
}

/// An element as FR-041 prints it: `id (kind)`, `id` alone when untyped.
fn element_label(element: &DesignElement) -> String {
    let id = qualified(element.repo.as_deref(), &element.id);
    match &element.kind {
        Some(kind) => format!("{id} ({kind})"),
        None => id,
    }
}

/// An element as a TSV field: `[kind:]<id>`, `[kind:]<repo>/<id>` with a repo.
fn element_field(element: &DesignElement) -> String {
    let id = qualified(element.repo.as_deref(), &element.id);
    match &element.kind {
        Some(kind) => format!("{kind}:{id}"),
        None => id,
    }
}

/// The entry's elements across all blocks, in block then element order, once
/// each by (repo, id).
fn entry_elements(blocks: &[DesignBlock]) -> Vec<&DesignElement> {
    let mut seen: Vec<(&Option<String>, &str)> = Vec::new();
    let mut out = Vec::new();
    for element in blocks.iter().flat_map(|block| &block.elements) {
        let key = (&element.repo, element.id.as_str());
        if !seen.contains(&key) {
            seen.push(key);
            out.push(element);
        }
    }
    out
}

/// The entry elements one criterion names in its `design_elements`, in the
/// order quire-rs gives them.
fn criterion_elements<'a>(
    blocks: &'a [DesignBlock],
    criterion: &DesignMatrixCriterion,
) -> Vec<&'a DesignElement> {
    let all = entry_elements(blocks);
    criterion
        .design_elements
        .iter()
        .flatten()
        .filter_map(|reference| {
            all.iter()
                .copied()
                .find(|e| e.id == reference.id && e.repo == reference.repo)
        })
        .collect()
}

/// What a block says beyond its elements, in cell order: a gap marker per
/// `uncovered` block, a dangling reference per `covered` block with no
/// elements, then each kind mismatch.
fn markers(blocks: &[DesignBlock]) -> Vec<String> {
    let gaps = blocks
        .iter()
        .filter(|b| b.state == RowState::Uncovered)
        .map(|b| format!("no design element ({})", b.check));
    let dangling = blocks
        .iter()
        .filter(|b| b.state == RowState::Covered && b.elements.is_empty())
        .flat_map(|b| &b.dangling)
        .map(|d| format!("-> {} (dangling)", d.reference));
    let mismatches = blocks.iter().flat_map(|b| &b.kind_mismatch).map(|m| {
        let id = qualified(m.repo.as_deref(), &m.id);
        match &m.archetype {
            Some(archetype) => format!("kind mismatch: {id} ({archetype})"),
            None => format!("kind mismatch: {id}"),
        }
    });
    gaps.chain(dangling).chain(mismatches).collect()
}

/// Elements then markers as one cell; `(none)` when there is neither.
fn join_cell(elements: Vec<String>, markers: Vec<String>) -> String {
    let parts: Vec<String> = elements.into_iter().chain(markers).collect();
    if parts.is_empty() {
        NONE.to_string()
    } else {
        parts.join(", ")
    }
}

/// The Design cell of one criterion row.
pub(super) fn criterion_cell(
    entry: &DesignMatrixEntry,
    criterion: &DesignMatrixCriterion,
) -> String {
    let Some(blocks) = &entry.design else {
        return NOT_APPLICABLE.to_string();
    };
    let elements = criterion_elements(blocks, criterion)
        .into_iter()
        .map(element_label)
        .collect();
    join_cell(elements, markers(blocks))
}

/// The `design:` value of an entry: the union of its elements, then markers.
pub(super) fn entry_cell(entry: &DesignMatrixEntry) -> String {
    let Some(blocks) = &entry.design else {
        return NOT_APPLICABLE.to_string();
    };
    let elements = entry_elements(blocks)
        .into_iter()
        .map(element_label)
        .collect();
    join_cell(elements, markers(blocks))
}

/// One `Design gap:` line per `uncovered` block.
pub(super) fn gap_lines(entry: &DesignMatrixEntry) -> Vec<String> {
    entry
        .design
        .iter()
        .flatten()
        .filter(|b| b.state == RowState::Uncovered)
        .map(|b| format!("Design gap: no design element ({})", b.check))
        .collect()
}

/// The lines under an entry's heading: `repo:` when the entry has one, then
/// its gap lines.
pub(super) fn header_lines(entry: &DesignMatrixEntry) -> Vec<String> {
    entry
        .repo
        .iter()
        .map(|repo| format!("repo: {repo}"))
        .chain(gap_lines(entry))
        .collect()
}

/// The TSV `design` field of a criterion, comma-separated.
pub(super) fn criterion_field(
    entry: &DesignMatrixEntry,
    criterion: &DesignMatrixCriterion,
) -> String {
    let Some(blocks) = &entry.design else {
        return String::new();
    };
    criterion_elements(blocks, criterion)
        .into_iter()
        .map(element_field)
        .collect::<Vec<_>>()
        .join(",")
}

/// The TSV `design` field of an entry with no criteria: the union of its elements.
pub(super) fn entry_field(entry: &DesignMatrixEntry) -> String {
    entry
        .design
        .as_deref()
        .map(entry_elements)
        .unwrap_or_default()
        .into_iter()
        .map(element_field)
        .collect::<Vec<_>>()
        .join(",")
}

/// The TSV `design_state`: empty for an entry with no `design` key.
pub(super) fn state(entry: &DesignMatrixEntry) -> &'static str {
    match &entry.design {
        None => "",
        Some(blocks) if blocks.iter().any(|b| b.state == RowState::Uncovered) => UNCOVERED,
        Some(_) => COVERED,
    }
}

/// A requirement is live unless it has criteria and every one is `retired`.
fn is_live(entry: &DesignMatrixEntry) -> bool {
    entry.criteria.is_empty()
        || entry
            .criteria
            .iter()
            .any(|c| c.criterion.status != quire_rs::coverage::CoverageMatrixStatus::Retired)
}

/// The `--strict` stderr lines: `design gap: [<repo>:]<document> <check>`, one
/// per live requirement and `uncovered` block, in entry then block order.
pub(super) fn gate_failures(entries: &[DesignMatrixEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| is_live(entry))
        .flat_map(|entry| {
            let place = match &entry.repo {
                Some(repo) => format!("{repo}:{}", entry.document),
                None => entry.document.clone(),
            };
            entry
                .design
                .iter()
                .flatten()
                .filter(|b| b.state == RowState::Uncovered)
                .map(move |b| format!("design gap: {place} {}", b.check))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    use quire_rs::coverage::{CoverageMatrixCriterion, CoverageMatrixStatus};

    fn criterion(id: &str, status: CoverageMatrixStatus) -> DesignMatrixCriterion {
        DesignMatrixCriterion {
            criterion: CoverageMatrixCriterion {
                id: id.to_string(),
                statement: "s".to_string(),
                method: None,
                binders: Vec::new(),
                record_binders: None,
                identity: None,
                status,
            },
            design_elements: Some(Vec::new()),
        }
    }

    fn dark_entry(statuses: &[CoverageMatrixStatus]) -> DesignMatrixEntry {
        DesignMatrixEntry {
            document: "spec/G-1.md".to_string(),
            repo: None,
            id: Some("G-1".to_string()),
            criticality: None,
            carried: None,
            criteria: statuses
                .iter()
                .enumerate()
                .map(|(n, s)| criterion(&format!("c{n}"), *s))
                .collect(),
            design: Some(vec![DesignBlock {
                relation: "r".to_string(),
                check: "g-dark".to_string(),
                state: RowState::Uncovered,
                elements: Vec::new(),
                dangling: Vec::new(),
                kind_mismatch: Vec::new(),
            }]),
        }
    }

    #[trace("FR-043-AC-9")]
    #[test]
    fn a_requirement_whose_criteria_are_all_retired_does_not_fail_the_gate() {
        let retired = dark_entry(&[CoverageMatrixStatus::Retired, CoverageMatrixStatus::Retired]);
        assert!(gate_failures(std::slice::from_ref(&retired)).is_empty());
        assert_eq!(
            gap_lines(&retired),
            ["Design gap: no design element (g-dark)"]
        );
        let live = dark_entry(&[CoverageMatrixStatus::Retired, CoverageMatrixStatus::Tagged]);
        assert_eq!(gate_failures(&[live]), ["design gap: spec/G-1.md g-dark"]);
        // No criteria at all is live.
        assert_eq!(gate_failures(&[dark_entry(&[])]).len(), 1);
    }

    #[trace("FR-043-AC-22")]
    #[test]
    fn the_three_empty_forms_are_distinct() {
        let mut covered = dark_entry(&[CoverageMatrixStatus::Tagged]);
        covered.design = Some(vec![]);
        assert_eq!(criterion_cell(&covered, &covered.criteria[0]), NONE);
        let mut out_of_scope = covered.clone();
        out_of_scope.design = None;
        assert_eq!(
            criterion_cell(&out_of_scope, &out_of_scope.criteria[0]),
            NOT_APPLICABLE
        );
        let uncovered = dark_entry(&[CoverageMatrixStatus::Tagged]);
        assert_eq!(
            criterion_cell(&uncovered, &uncovered.criteria[0]),
            "no design element (g-dark)"
        );
    }
}
