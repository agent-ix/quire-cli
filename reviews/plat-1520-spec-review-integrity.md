---
id: SR-20261112234813
title: PLAT-1520 quire-cli PR 180 spec-review/integrity
type: SpecReview
analysis: integrity
scope: agent-ix/quire-cli; PR 180; spec/functional/FR-030-relations-subcommand.md, spec/functional/index.md
review_set: subset
---

## Summary

Ticket: PLAT-1520. Integrity analysis (completeness, consistency, atomicity) of FR-030 and its index entry in quire-cli PR 180. The index entry is correct; FR-030's links resolve by the same `ix://agent-ix/quire-rs/<ID>` convention as FR-017/FR-029.

## Verdict

CHANGES REQUESTED (1 medium, 3 low). Main gap: the multi-repository `--repo` input path has no root, path-safety or duplicate-identity criteria.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | medium | The `--repo <ORG/REPO>=<DIR>` input is specified only for its happy path (AC-3). Nothing says whether a `--repo` root without `spec/` is an error (Behavior and AC-7 name only `--scope`), no AC applies FR-005 path safety to `--repo` roots, and no AC covers two `--repo` flags with one identity (quire-rs FR-085-AC-9 makes it an engine error; the CLI exit and diagnostic are unstated) or a malformed `ORG/REPO=DIR` value. | spec/functional/FR-030-relations-subcommand.md:35-38 |
| FND-002 | low | Dependencies lists quire-rs FR-058 as upstream, but the frontmatter `relationships` carries only StR-004, FR-084 and FR-085, so the FR-058 edge is invisible to `quire extract`/trace. Add a `references` relationship or drop it from Dependencies. | spec/functional/FR-030-relations-subcommand.md:70 |
| FND-003 | low | AC-2 joins the forward-view gap rendering and the reverse-view gap (classified requirement with no linked risk document) in one criterion, and the fixture does not say a reverse-direction relation is declared; split it, or name the two declarations the fixture carries. | spec/functional/FR-030-relations-subcommand.md:59 |
| FND-004 | low | quire-rs FR-084 gives each relation a `summary` (row count by `state` and `verification`); FR-030's human output says nothing about rendering it, so a human reader gets no per-relation totals. State whether the human block prints the summary. | spec/functional/FR-030-relations-subcommand.md:42-45 |

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
| FND-005 | medium | `validate --okf --repo` has no engine contract. FR-014 validates one bundle via `validate_bundle(spec, registry, posture, document_root, reference_root)`, with one document root and one reference root; quire-rs FR-085 (quire-rs#664 head) builds a multi-member corpus and resolves edges, but specifies no bundle validation over one, no per-member bundle and reference roots, no `--strict` posture across members, and no source-member identity on FR-058 findings. FR-030 nonetheless states that every member is validated against the one corpus and that each finding names the member identity. Under StR-004 the upstream FR comes first: add the multi-member validate entry point (roots, posture, finding identity) to quire-rs#664, or state here which existing quire-rs API the CLI calls per member. | spec/functional/FR-030-relations-subcommand.md:42-50 |
| FND-006 | low | The section makes `--repo` with `--scope` or a positional argument an error on `validate`, but AC-16 covers only `relations` and AC-21 only a missing `--okf`; no AC covers `validate --okf --repo` with `--scope` or a positional bundle root. The text also says "positional globs", while FR-014's `--okf` positional is a bundle `<DIR>`. | spec/functional/FR-030-relations-subcommand.md:49-50 |

## Dispositions

| FND | outcome | reason |
|---|---|---|
| FND-001 | fixed | verified in commit 3746178 |
| FND-002 | fixed | verified in commit 3746178 |
| FND-003 | fixed | verified in commit 3746178 |
| FND-004 | fixed | verified in commit 3746178 |
| FND-005 | still-open | Commit e495fd9 names the engine entry point (quire-rs FR-085 `validate_corpus`, AC-16..20), the posture (FR-014) and the finding identity (source-member `repo`, FR-085-AC-19), which resolves those parts. The per-member roots this finding named are still missing: FR-030 passes "one root per member" and FR-085 takes `roots`, one per member, but FR-014 calls the engine with two roots per bundle, a document root (`<scope>/spec` or the positional `<DIR>`) and a reference root (the scope), so path-bound module declarations resolve against the repository (quire-rs FR-049-AC-9). For a `--repo o/a=<DIR>` member, nothing says whether `<DIR>` or `<DIR>/spec` is the bundle root, or what the member's reference root is. Fix: state document root `<DIR>/spec` and reference root `<DIR>` per member here, and give `validate_corpus` a root pair per member in quire-rs#664. |
| FND-006 | fixed | verified in commit e495fd9 |
| FND-005 | fixed | verified in commit 2778ef1 against quire-rs#664 FR-085 validate_corpus root pairs and FR-085-AC-21 |
