---
id: SR-20261113142746
title: "PLAT-1587 PLAT-1589 quire-cli PR195 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR195 head 6f5d180; src/commands/matrix.rs, src/commands/matrix_design.rs, src/commands/mod.rs, tests/cli_matrix_design.rs, Cargo.lock, CHANGELOG.md, spec/functional/FR-043-matrix-design-column.md"
review_set: subset
---

## Summary

Tickets: PLAT-1587 (CLI half) and PLAT-1589. This is a code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`. The PR moves `quire matrix` onto quire-rs `design_matrix` (FR-092). It adds a Design column to Markdown, human, JSON and TSV output, a `--repo <ORG/REPO>=<DIR>` corpus input, and the `--strict` design gate (FR-043).

The rendering follows FR-043's Behavior section closely: cell forms, marker order, gap-line order, TSV columns, zero population, the gate and exit codes. The CLI holds no archetype or verb name, and there are no compatibility layers, digests, pins or SHAs.

Gates run: `cargo fmt --check` passed, and `cargo test --test cli_matrix_design --test cli_matrix` passed (15 + 23). I also ran 13 hand mutations on a scratch copy; 11 were killed (see gap-analysis SR-20261113142747). Clippy and the aggregate gate were not run, by brief.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `parse_repo` counts identity segments after dropping empty ones, so it accepts `o//a`, `/o/a` and `o/a/`. Measured: `quire matrix --repo 'agent-ix//quire-cli=.'` exits 0 and prints `agent-ix//quire-cli` as the TSV `repo` value. An identity like that matches no `ix://` reference, so a typo shows up as false design gaps instead of an input error. No test covers a malformed `--repo` value; the mutant `count() >= 2` survived. | src/commands/matrix.rs:104-111 |
| FND-002 | medium | `--repo` gets its own parser and member loader in matrix.rs. Open PR #184 (PLAT-1554) adds `src/commands/corpus_input.rs`, documented as "the `--repo <ORG/REPO>=<DIR>` corpus input shared by" commands, with `parse_repo` (a clap value_parser that rejects empty segments) and `Corpus::from_repos`. FR-043 says `--repo` "resolves as in FR-041", which in turn follows FR-030. The two parsers already disagree on FND-001's inputs and on when the error is raised (clap parse time versus run time). Whichever PR lands second should use `corpus_input` and delete the local copy. | src/commands/matrix.rs:97-138 |
| FND-003 | low | Each member is walked twice: `compute_report` loads `Spec::from_path(<root>/spec)`, then `compute_matrix` calls `load_repo(&spec_root)` again for `Spec::from_members`. This costs a second full document walk per member. A file edited between the two walks would also give criteria from one snapshot and design blocks from another. Fixing it needs a `compute_report` variant that takes a loaded spec; #184 has the same shape, so the fix belongs with the shared corpus input. | src/commands/matrix.rs:189-198 |
| FND-004 | low | The README synopsis still reads `quire matrix [--scope <DIR>] [--module <PATH>]... [--format markdown\|json\|tsv] [--strict]`. It lacks `--repo` and `--format human`, so it disagrees with FR-043's synopsis and the new `--help`. | README.md:45 |
| FND-005 | low | Spec, FR-043-AC-2: the human `design:` line is described as "(elements, or `out of scope`)". The Behavior section and AC-25 define the out-of-scope token as `n/a`, and the code prints `n/a`. AC-2 also lists the `design:` line before the `Design gap:` lines, while Behavior puts the gap lines first. The spec means `n/a` and gap-lines-first; AC-2 needs the one-word fix (`out of scope` -> `n/a`) and its order corrected. Reported only; not edited. | spec/functional/FR-043-matrix-design-column.md:114 |
| FND-006 | low | Spec, FR-043 Behavior "Design cell": an element "with its repo when present" defers to FR-041, and FR-041 never states the separator. Only the TSV clause and AC-24 state `[kind:]<repo>/<id>`. The code prints `o/b/LAN-001 (Lantern)` in Markdown and human, which is consistent with AC-24. But the gate line uses `<repo>:<document>`, and #184's relations output uses `repo:id`, so the Markdown/human form should be written down (`<repo>/<id> (kind)`). | spec/functional/FR-043-matrix-design-column.md:58-60, src/commands/matrix_design.rs:21-35 |

## Examined scope

```yaml
scope:
- id: src/commands/matrix.rs
  path: src/commands/matrix.rs
  role: examined
  excerpt: "#[arg(long, value_name = \"ORG/REPO=DIR\", conflicts_with = \"scope\")] pub repo: Vec<String>"
- id: src/commands/matrix_design.rs
  path: src/commands/matrix_design.rs
  role: examined
  excerpt: "Nothing here knows what a requirement or a design element is."
- id: tests/cli_matrix_design.rs
  path: tests/cli_matrix_design.rs
  role: examined
  excerpt: "Every kind, verb, relation, check and criterion id below is invented"
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: "quire-rs ... branch=main#b502d6c (PLAT-1587 design_matrix op)"
- id: CHANGELOG.md
  path: CHANGELOG.md
  role: examined
  excerpt: "quire matrix design column (PLAT-1587, PLAT-1589; FR-043)."
- id: FR-043
  path: spec/functional/FR-043-matrix-design-column.md
  role: examined
  excerpt: "quire matrix SHALL render the design column of quire-rs FR-092"
- id: FR-092
  path: "quire-rs spec/functional/FR-092-matrix-design-column.md"
  role: context_only
  excerpt: "Key order is document, repo, id, criticality, carried, criteria, design."
```

## Verdict

CONDITIONAL. Two medium findings: the `--repo` parser accepts empty identity segments, and it duplicates the shared parser in PR #184. There are four low findings, two of them spec wording.

The coder's stated deviations:

- (a) `--repo` should be shared. #184 already holds the shared home, `corpus_input.rs` (FND-002).
- (b) Absent design columns and unchanged output without `design_relations` match FR-043 Behavior and AC-11.
- (c) The spec means `n/a`; AC-2 needs a one-word fix (FND-005).
- (d) `<repo>/<id>` is stated for TSV and AC-24, but not for Markdown/human (FND-006).
- (e) AC-9 being covered only by a unit test is in the gap analysis.
- (f) The double load is FND-003.

What works: every Design cell form, marker order, `repo:` then `Design gap:` order, the TSV leading `repo` column and criteria-less record, one stderr line per uncovered block of a live requirement, exit 1/3, and the `--scope`/`--repo` conflict all match FR-043. The JSON payload is the engine's `DesignMatrixEntry` verbatim, so FR-092 key order is the engine's. No archetype names, verbs, compatibility layers or digests appear.

## Dispositions
