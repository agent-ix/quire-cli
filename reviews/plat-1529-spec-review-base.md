---
id: SR-20261113083701
title: "PLAT-1529 quire-cli PR188 spec-review base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR188; spec/functional/FR-042-matrix-design-column.md, spec/functional/FR-026-matrix-subcommand.md (CR note), spec/functional/index.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/FR-042
    type: references
---

## Summary

Ticket: PLAT-1529. The PR adds FR-042 (`quire matrix` design column, `--format human`, `--repo`, a design gap under `--strict`), a CR note on FR-026 and an index entry. Context read: FR-026, FR-028, FR-030, FR-041, FR-007, FR-017 (scope default), and the companion quire-rs FR-092 from quire-rs PR 670. The review set is base plus the integrity, dependency and EARS lenses. No AssuranceProfile document exists in the repository, so no review policy applies. `quire validate` passes on the three changed files. The only warning is on FR-026-AC-16, which this diff does not touch.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | "An empty cell means out of scope" is false. Under a covered entry, a criterion whose `design_elements` is empty also has an empty cell (FR-092: not a gap; AC-5 requires it), so in Markdown and human output an empty cell has two meanings | spec/functional/FR-042-matrix-design-column.md:49-53 |
| FND-002 | medium | FR-026 §C/§D truncation is applied to the Design cell. The 80-character cut can remove the `no design element (<check>)` marker the cell exists to show, because the marker is appended after the elements | spec/functional/FR-042-matrix-design-column.md:56-58 |
| FND-003 | medium | No AC covers the `--scope` with `--repo` exit-3 error that the Description and Errors sections state | spec/functional/FR-042-matrix-design-column.md:40-41,90-92 |
| FND-004 | medium | `--repo` without `design_relations` is undefined. The rule "output is FR-026's unchanged" and AC-11 assume one scope, and FR-026 has no repo labels for multi-member entries | spec/functional/FR-042-matrix-design-column.md:45-47,108 |
| FND-005 | low | The synopsis `(--scope <DIR> \| --repo ...)` reads as mandatory and drops FR-026's `[--scope]` default of `.` | spec/functional/FR-042-matrix-design-column.md:33-34 |
| FND-006 | low | The human format does not say whether the `Design gap:` and `repo:` lines are printed, or their order under the heading. AC-19 does not name its formats | spec/functional/FR-042-matrix-design-column.md:53-55,59-62,115 |
| FND-007 | low | Under `--repo`, the strict stderr line names only `document`. Two members can share a document path (FR-092-AC-15), so the line does not identify the failing requirement | spec/functional/FR-042-matrix-design-column.md:79-82 |
| FND-008 | low | `kind_mismatch` is never rendered, unlike FR-041. A block left uncovered by a kind mismatch shows only `no design element` | spec/functional/FR-042-matrix-design-column.md:49-52 |
| FND-009 | low | TSV joins a repo with two separators: `<repo>:` before `document` but `kind:repo/id` in `design`. AC-4 covers neither repo form | spec/functional/FR-042-matrix-design-column.md:39-40,66-68 |

## Examined scope

```yaml
scope:
- {id: FR-042, path: spec/functional/FR-042-matrix-design-column.md, role: examined, excerpt: "quire matrix (FR-026) SHALL render the design column of quire-rs FR-092: for each requirement, the design elements that realize it, then its criteria, their tests and results."}
- {id: FR-042-AC-1..20, path: spec/functional/FR-042-matrix-design-column.md, role: examined, excerpt: "all 20 acceptance criteria"}
- {id: FR-026, path: spec/functional/FR-026-matrix-subcommand.md, role: examined, excerpt: "CR note (design column and design gate, PLAT-1529)"}
- {id: FR-026-AC-4, path: spec/functional/FR-026-matrix-subcommand.md, role: examined, excerpt: "Each requirement renders as the exact heading ## <document> ..."}
- {id: FR-007, path: spec/functional/FR-007-exit-codes.md, role: context_only, excerpt: "1 Partial result ... 3 Invalid request"}
- {id: FR-030-AC-16, path: spec/functional/FR-030-relations-subcommand.md, role: context_only, excerpt: "--scope together with --repo exits 3 naming both flags."}
- {id: FR-041, path: spec/functional/FR-041-relations-design-mode.md, role: context_only, excerpt: "--design ... conflicts with --strict: the matrix design column and the strict gate over it are a separate requirement (PLAT-1529)"}
- {id: FR-092, path: "agent-ix/quire-rs spec/functional/FR-092-matrix-design-column.md (PR 670)", role: context_only, excerpt: "a criterion with an empty design_elements under a covered entry is not a gap."}
```

## Verdict

Changes requested: 4 medium and 5 low findings, with no high finding. The following were checked and are clean:

- The FR-026-AC-4 heading `## <document>` is kept and stated twice.
- Exit codes follow FR-007: 1 for a strict failure with the full rendering on stdout, 3 for an unknown design relation and for `--scope` with `--repo`.
- `--strict` follows FR-030: the complete report is printed, then the run exits 1. It is consistent with FR-041's deferral of the strict gate to this ticket.
- "Live requirement" is defined precisely. A requirement is live unless it has criteria and every one is `retired`. AC-9 and AC-19 test this.
- `--format human` is justified by the ticket: "in human, Markdown, JSON and TSV output".
- The TSV columns keep FR-028's `requirement_criticality` last.
- Gate scope is module data through the FR-058 `where`, `from` and `exclude` selectors (AC-8). No name is held in the CLI (AC-14, AC-15).
- The no-`design_relations` path is the engine's own byte-identical output, not a compatibility layer.
- There is no hash, digest or pin antipattern.
