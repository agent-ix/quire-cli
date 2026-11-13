---
id: SR-20261112234812
title: PLAT-1520 quire-cli PR 180 spec-review/base
type: SpecReview
analysis: base
scope: agent-ix/quire-cli; PR 180; spec/functional/FR-030-relations-subcommand.md, spec/functional/index.md
review_set: subset
---

## Summary

Ticket: PLAT-1520. Base checklist review of quire-cli PR 180 (FR-030 `quire relations`, index entry), scoped to `git diff origin/main...HEAD`. FR-030 correctly references quire-rs FR-084/FR-085 (agent-ix/quire-rs#664) for the report computation, row states, verification statuses and cross-repository resolution instead of restating them; it carries no paywalled standard text, no tracking table or pin, and no compatibility layer. `quire validate` passes. No applicable AssuranceProfile exists in this repository. Quoin 0.30.1.

## Verdict

CHANGES REQUESTED (3 medium, 1 low). The thin-boundary split is sound and nothing is duplicated from quire-rs. The CLI-owned policy (strict exit and refusals) needs tightening: `--strict` passes an empty or unmeasured report, refusal ACs say `non-zero` where FR-007 fixes exit 3, and the `partial` strict branch has no AC. Merge ordering: quire-rs#664 is still open, so FR-030's FR-084/FR-085 targets do not exist on quire-rs main yet; land #664 first or together.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | medium | `--strict` fails only on `uncovered`, `unverified` or `partial` rows, so a relation selecting zero documents (quire-rs FR-084-AC-10) and a report whose rows are all `not-tracked` (no `obligations:` source, FR-084-AC-8) both exit 0 under `--strict`. That is a strict gate passing on a report that measured nothing, contrary to this repo's own nonempty-population premise (FR-017-AC-8, FR-026-AC-10). State whether zero rows and `not-tracked` rows fail `--strict`, and add an AC for each. | spec/functional/FR-030-relations-subcommand.md:49-50 |
| FND-002 | medium | AC-7 (and AC-5's unknown `--relation`) assert only `non-zero`, so an implementation exiting 1 (FR-007: partial result, complete report emitted) or 2 (refusal) passes the AC while violating FR-007, which classes invalid argv, missing input and unresolved selection as 3. Pin exit 3 per FR-007 and the `MissingDocumentRoot` diagnostic kind FR-017-AC-5 uses for a scope with no `spec/`. | spec/functional/FR-030-relations-subcommand.md:64 |
| FND-003 | medium | Behavior lists `partial` as a `--strict` failure, but AC-6 exercises only `uncovered` and `unverified`; an implementation that lets `partial` rows pass `--strict` satisfies every AC. Add a `partial` row case. | spec/functional/FR-030-relations-subcommand.md:63 |
| FND-004 | low | "exits 0 on the same corpus without it" can be read as without `--strict` or without the failing row; the second clause suggests the flag, but say so explicitly (FR-017-AC-7 has the same wording with an unambiguous antecedent). | spec/functional/FR-030-relations-subcommand.md:63 |

## Examined units

```yaml
scope:
- id: FR-030
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: The CLI SHALL provide a `relations` subcommand that renders the two-way relation coverage report
    of quire-rs [FR-084](ix://agent-ix/quire-rs/FR-084) over one repository or over several repositories
    loaded as one corpus ([FR-085](ix://agent-ix/quire-rs/FR-085)).
- id: FR-030-AC-1
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: '`quire relations --scope <DIR> --module $M --json` over a fixture corpus with an unmitigated
    risk, a mitigated-but-untested risk and a mitigated-and-tested risk emits rows whose `state` and `verification`
    are respectively `uncovered`, `covered`/`unverified`, `covered`/`verified`, each row carrying the
    document id, linked ids and per-linked-document verification, and stdout holds only the JSON.'
- id: FR-030-AC-2
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: The default human output lists each row with its state and verification, marks an `uncovered`
    row as a gap with the relation's `check` token, and lists a classified requirement with no linked
    risk document as a gap.
- id: FR-030-AC-3
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: '`--repo o/a=<A> --repo o/b=<B>` over a fixture where a requirement in `o/a` mitigates a hazard
    in `o/b` shows the hazard `covered`; the same command without `--repo o/b` shows the reference under
    the row''s dangling entries, naming it as authored.'
- id: FR-030-AC-4
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: A fixture edge to a wrong-kind target prints beneath its row as a kind mismatch naming the
    target and its actual archetype, in human and `--json` output.
- id: FR-030-AC-5
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: '`--relation <NAME>` restricts the report to that declaration; an unknown name exits non-zero
    naming it with no report.'
- id: FR-030-AC-6
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: '`--strict` exits 1 over a corpus with an `uncovered` or `unverified` row and exits 0 on the
    same corpus without it; a corpus whose rows are all `verified` exits 0 under `--strict`.'
- id: FR-030-AC-7
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: '`--scope` together with `--repo`, a `--scope` holding no `spec/`, and a module set declaring
    no `required_relations` each exit non-zero with a diagnostic naming the cause.'
- id: FR-030-AC-8
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: Two runs over identical inputs produce byte-identical `--json` output, compact by default and
    indented under `--pretty` with the identical parsed value.
- id: FR-030-AC-9
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: The CLI source holds no hazard, threat or safety-classification name, and delegates the report
    to quire-rs.
- id: spec/functional/index.md
  path: spec/functional/index.md
  role: examined
  excerpt: '* [FR-030: quire relations subcommand](./FR-030-relations-subcommand.md)'
- id: quire-rs FR-084
  path: agent-ix/quire-rs#664 spec/functional/FR-084-two-way-relation-coverage-report.md
  role: context_only
  excerpt: 'This FR adds a **report** over the same declarations: one row per obliged document, listing
    what is linked to it and how well that is verified.'
- id: quire-rs FR-085
  path: agent-ix/quire-rs#664 spec/functional/FR-085-cross-repository-edge-resolution.md
  role: context_only
  excerpt: This FR lets one corpus hold **several repositories** and resolves edges between them.
- id: FR-007
  path: spec/functional/FR-007-exit-codes.md
  role: context_only
  excerpt: '| 1 | Partial result: a complete requested report was emitted, but strict coverage/matrix
    or property resolution found incomplete or failing records. | ... | 3 | Invalid request: invalid argv,
    missing or incorrectly typed input paths, malformed input, unknown archetypes, unresolved selection,
    module load or structural validation failures. |'
- id: FR-017-AC-8
  path: spec/functional/FR-017-coverage-subcommand.md
  role: context_only
  excerpt: A model matching zero live and zero retired rows fails `--strict` with a diagnostic distinct
    from the unbacked-rows one, rather than reporting full coverage.
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-005 | medium | AC-20 asserts on an "unjustified-scope finding", but no check by that name exists: the sibling manifests' tokens are `unjustified-safety-nfr` and `unjustified-security-nfr`, and FR-030 itself defines none. A test cannot target an unnamed finding, and naming a chain token would break AC-22. Phrase it over the fixture module's own relation, e.g. "no finding carrying the fixture relation's `check` token". | spec/functional/FR-030-relations-subcommand.md:101 |
| FND-006 | low | Behavior calls a module set without `required_relations` and an unknown `--relation` "refusals", which is FR-007's word for exit 2, while the Description and AC-9/AC-15 say exit 3. Call them invalid requests, or drop the word. | spec/functional/FR-030-relations-subcommand.md:75-76 |

## Dispositions

| FND | outcome | reason |
|---|---|---|
| FND-001 | fixed | verified in commit 3746178 |
| FND-002 | fixed | verified in commit 3746178 |
| FND-003 | fixed | verified in commit 3746178 |
| FND-004 | fixed | verified in commit 3746178 |
| FND-005 | fixed | verified in commit e495fd9 |
| FND-006 | fixed | verified in commit e495fd9 |
