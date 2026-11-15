---
id: SR-20261113192643
title: "PLAT-1441 quire-cli PR204 spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR204 head 64ff318; spec/functional/FR-030-relations-subcommand.md (amendment); agent-ix/quire-rs FR-084 on main and on open PR #688, agent-ix/spec-objects-security FR-023 (context)"
review_set: subset
---

## Summary

Ticket: PLAT-1441. A base spec review of the FR-030 amendment, compared with what it claims to pass through. That is the "Carried record values and the reverse view" section added to quire-rs FR-084 by the open quire-rs PR #688, which is not merged; quire-rs `main` FR-084 has no `record`. The key `regulatedCategories` and its authored form `regulated_categories` are declared by `agent-ix/spec-objects-security` FR-023. The sub-analyses I chose are EARS (SR-20261113192644), integrity (SR-20261113192645) and dependency (SR-20261113192646). `quire validate` on FR-030 could not be run, because the harness denied the command in this session. FR-030 has not been validated by me or, per the PR body, by its author.

## Verdict

**CONDITIONAL**: two medium and two low findings. The intent is right. The CLI passes the engine report through unchanged, holds no category logic and is deterministic. The code already does this (SR-20261113192641). But the bullet misdescribes where FR-084 puts `record`, and it states `record` as always present. AC-26 does not pin the absent case, which is what guards against the CLI defaulting or exposing a key the engine did not emit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The bullet says FR-084 attaches `record` "to the linked target". quire-rs PR #688 puts `record` on every document entry: the obliged `document` and each resolved `linked` entry. In the classification-to-connection (incoming) view the categories sit on the row's `document.record`, which the bullet and AC-26 omit. | spec/functional/FR-030-relations-subcommand.md:69-74 |
| FND-002 | medium | "Each `--json` row carries ... `record`" states `record` as always present. FR-084 omits `record` when a document has no resolved `data_schema` or an empty record, and a key that was not authored is absent, never defaulted. AC-26 has no negative case asserting that the CLI emits no `record` and no `regulatedCategories` (not `null`, not `[]`) when the engine omits them, unlike quire-rs FR-084-AC-20. | spec/functional/FR-030-relations-subcommand.md:69-71, spec/functional/FR-030-relations-subcommand.md:112 |
| FND-003 | low | "under the engine's own camelCase key spelling; the CLI does not re-spell the authored snake_case form" attributes the key to the engine. Under quire-rs FR-069 (PR #688) the engine knows no key name. The record key `regulatedCategories` and its source `regulated_categories` are declared by the spec-objects-security module (FR-023). The CLI spec should say "the record key as the module declares it". | spec/functional/FR-030-relations-subcommand.md:71-72 |
| FND-004 | low | "Reverse classification-to-connection rows of FR-084" reads as a distinct row kind. In FR-084 they are ordinary rows of a second, `direction: incoming` declaration over the same verb. AC-26 does not say the fixture module declares both relations, so "the reverse ... rows" has no defined source in the fixture. | spec/functional/FR-030-relations-subcommand.md:72-73, spec/functional/FR-030-relations-subcommand.md:112 |

## Examined scope

```yaml
scope:
  - {id: FR-030, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "Each `--json` row carries, unchanged, the generic semantic `record` that quire-rs FR-084 attaches to the linked target, including its `regulatedCategories` when the engine produced them, under the engine's own camelCase key spelling; the CLI does not re-spell the authored snake_case form. Reverse classification-to-connection rows of FR-084 are preserved as the engine produced them. The CLI neither interprets, filters nor adds to the record, and holds no category-specific logic."}
  - {id: FR-030-AC-26, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "`quire relations --scope <DIR> --module $M --json` over a fixture whose linked target carries a semantic record with `regulatedCategories` emits, on the row, the same `record` (including `regulatedCategories`, camelCase) as the quire-rs FR-084 report, and emits the reverse classification-to-connection rows unchanged; a second run over identical inputs is byte-identical to the first."}
  - {id: quire-rs/FR-084, path: "agent-ix/quire-rs PR #688 spec/functional/FR-084-two-way-relation-coverage-report.md", role: context_only, excerpt: "Every document entry of the report (the obliged `document` and each resolved `linked` entry) carries `record`: the document's composed semantic record, exactly as extracted ... `record` is omitted when the document has no resolved `data_schema` or its record is empty; a key the document did not state is absent, never defaulted. The engine selects no key"}
  - {id: spec-objects-security/FR-023, path: "agent-ix/spec-objects-security spec/functional/FR-023-regulated-data-categories-on-data-classification.md", role: context_only, excerpt: "The `data_classification` manifest entry SHALL extract the frontmatter list `regulated_categories` into `regulatedCategories` with a `frontmatter_field` locator that has `required: false`."}
```

## Dispositions

Round 1, fix commit 7a89d89. Compared with quire-rs PR #688 head 7b43672 (OPEN, not merged).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The bullet now says "Each document entry of a `--json` row (the row's `document` and each resolved `linked` entry) carries ... the generic semantic `record`", which matches #688 FR-084. AC-26 asserts "the `document` and `linked` records" in both directions. |
| FND-002 | fixed | The bullet says the `record` is present "only when the engine produced one", and that the entry otherwise has "no `record` key, never `null` or an empty value". AC-26 adds "emits no `record` and no `regulatedCategories` key on an entry for which FR-084 omits it". |
| FND-003 | fixed | The bullet now says "the key `regulatedCategories` is declared by the spec-objects-security module (`agent-ix/spec-objects-security` FR-023), not known to the engine or the CLI". |
| FND-004 | fixed | The bullet now calls the incoming view "an ordinary row set", not a distinct row kind. AC-26 now says "a fixture module declaring both an outgoing relation and its incoming relation". The new wording has its own defect, recorded as FND-005. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | "The reverse view is an ordinary row set from a declaration with `direction: incoming` over the same relation" gets FR-084's terms wrong. In #688 FR-084 ("One report, both directions"), the **reverse view** is `direction: outgoing` with a `where` selector, and an incoming declaration gives the forward view. The two views are also two declarations, so two relations, over the same **verb**, not "the same relation". Say "the classification-to-connection view is an ordinary row set from a second declaration, with `direction: incoming`, over the same verb". | spec/functional/FR-030-relations-subcommand.md:76-78 |
| FND-006 | medium | The parenthetical "(no resolved `data_schema`, or an empty record)" restates FR-084's omission conditions in the CLI spec. It matches #688 head 7b43672. But #688's own base review (SR-20261115183659 FND-001, high) asks FR-084 to carry only the record keys the declaration selects, which would change when `record` is omitted. A pass-through spec only needs "when FR-084 omits `record`, the entry has no `record` key". Drop the parenthetical so FR-030 cannot drift from the engine's rule. | spec/functional/FR-030-relations-subcommand.md:71-73 |
