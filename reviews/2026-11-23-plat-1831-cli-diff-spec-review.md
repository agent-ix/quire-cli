---
id: SR-20261116045015
title: "PLAT-1831 quire-cli PR229 spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR229 head 83a7028; spec/functional/FR-021-clauses-subcommand.md"
review_set: subset
date: 2026-11-23
---

## Summary

Ticket: PLAT-1831. Spec review of the FR-021 edit: the amended AC-4 and the new AC-9. EARS, integrity and evidence checks were applied inline to these two rows. `quire validate --scope . spec/functional/FR-021-clauses-subcommand.md` passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-9 is compound: three-view rendering, schema conformance, and load-time rejection across two commands, each with its own failure mode. Its schema clause repeats AC-6, and the AC-4 amendment repeats its rendering clause. | spec/functional/FR-021-clauses-subcommand.md:48, spec/functional/FR-021-clauses-subcommand.md:43 |
| FND-002 | low | "a padded kind" is ambiguous, because each catalog entry also has its own `kind` field; the AC means a producer kind with leading or trailing whitespace. The AC also omits the exit code (the test asserts 3 and empty stdout) and duplicate kinds, which the engine also rejects. | spec/functional/FR-021-clauses-subcommand.md:48 |
| FND-003 | low | "show each ... changed entry" does not say what a TSV changed row must show. Shipped TSV shows only producers, so a kind- or description-only change is indistinguishable (code-review FND-003), and the AC cannot judge that either way. | spec/functional/FR-021-clauses-subcommand.md:48 |

## Finding details

- FND-001 fix: split AC-9 into a rendering AC and a rejection AC, drop the schema clause (AC-6 covers it), and either drop the AC-4 amendment or keep it and tag it142 to AC-4.
- FND-002 fix: write "a producers list that is empty, or that holds an empty, whitespace-padded or duplicate kind, exits 3 with empty stdout and a diagnostic naming the output".
- FND-003 fix: state which fields a TSV changed row carries (producers only, or kind/description too), and make the code match.

## Scope examined

```yaml
scope:
- id: FR-021-AC-9
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: "Human, TSV, and JSON `diff` views show each added, removed, and changed `outputCatalog` entry; an entry's producers are shown when present. JSON conforms to the engine's published diff schema. `diff` and `evaluate` reject a clause set whose catalog producers are invalid (empty list or a padded kind) with a diagnostic naming the output."
- id: FR-021-AC-4
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: "`diff` delegates to the engine and reports added, removed, and changed clauses between two exact versions of the same authority and set id, and the output-catalog entries added, removed, and changed between them."
- id: FR-021-AC-6
  path: spec/functional/FR-021-clauses-subcommand.md
  role: context_only
  excerpt: "JSON carries tool provenance including the `clause_sets` capability and conforms to the pinned engine's published binding or diff schema."
```

## Verdict

Acceptable with low findings. The spec was updated before the code, as required, and the rows are testable.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | AC-9 opens with all three views, then says "A changed entry ... names which of `kind`, `description`, and `producers` changed". Only the TSV `changed_fields` cell names them: the human view prints before/after descriptions with no field list, and JSON is the engine's before/after. As written, the AC claims the human and JSON views name the changed fields. | spec/functional/FR-021-clauses-subcommand.md:48 |

Fix for FND-004: scope that sentence to TSV ("In TSV, a changed entry names which of ..."), or add the field list to the human `output changed` line.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 7e50a89: rejection split into FR-021-AC-10, the schema clause dropped from AC-9 (AC-6 owns it), AC-4 kept and bound by it142 |
| FND-002 | fixed | 7e50a89: AC-10 reads "a producers list that is empty, or that holds an empty, whitespace-padded, or duplicate producer kind, both `evaluate` and `diff` exit 3 with empty stdout and a diagnostic that names the output" |
| FND-003 | fixed | 7e50a89: AC-9 defines the TSV `changed_fields` cell (kind, description, producers in that order, comma-separated) and the JSON-array producers cell |
