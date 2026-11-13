---
id: "SR-20261113080108"
title: "PLAT-1417 quire-cli PR187 integrity analysis"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli PR187 head 8134836; spec/functional/FR-041-relations-design-mode.md, spec/functional/index.md"
review_set: subset
---

## Summary

Ticket: PLAT-1417. Completeness, consistency and atomicity of FR-041's flag surface, behavior and seven ACs against FR-030, FR-007 and quire-rs FR-086. Four findings: `--design --strict` is undefined, the human row layout contradicts 'is FR-030's' on verification and summary, an engine error path from FR-086 has no CLI outcome, and AC-2 is compound.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `--design` with `--strict` is undefined: the synopsis omits `--strict`, the text says the strict gate over the design column is a separate requirement yet `--strict` 'keeps FR-030's meaning', and no AC covers the combination, so it could be rejected (exit 3), gate the design rows (exit 1 on uncovered) or gate nothing. | spec/functional/FR-041-relations-design-mode.md:28,33-34 |
| FND-002 | medium | Human output 'is FR-030's' but then gives a row layout (id, state, elements, criteria) that drops FR-030's `verification` and per-element verification and never mentions the per-block `summary`; whether design mode prints verification and the summary is unstated, and no AC checks summary parity with `--json` as FR-030-AC-5 does. | spec/functional/FR-041-relations-design-mode.md:38-41 |
| FND-003 | medium | FR-086 makes the join error when a `design_relations` entry names no declared relation (FR-086-AC-10), but FR-041 gives that engine error no CLI outcome and no AC; per FR-007 it should exit 3 naming the entry with no report on stdout, and the spec should say so. | spec/functional/FR-041-relations-design-mode.md:44-45,55 |
| FND-004 | low | FR-041-AC-2 combines determinism with design-relation selection in one criterion, and 'In the same run two executions' is self-contradictory wording; split into two ACs. | spec/functional/FR-041-relations-design-mode.md:52 |

## Examined scope

```yaml
scope:
- id: FR-041
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '`quire relations` ([FR-030](./FR-030-relations-subcommand.md)) SHALL accept `--design`, which
    renders the design-element join of quire-rs [FR-086](ix://agent-ix/quire-rs/FR-086): for each requirement,
    the design elements that realize it, and an explicit gap when there are none. The join, its rows,
    states and `criteria` belong to quire-rs; the CLI resolves roots, applies path safety and renders
    ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)). It holds no requirement or design
    archetype name and no realizing verb: which relations are design relations is the loaded modules''
    `tr'
- id: FR-041#synopsis
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '`--design` combines with `--scope`, `--repo`, `--module` and `--json` exactly as FR-030 defines
    them, so design elements held in a sibling repository join through `--repo` ([FR-085](ix://agent-ix/quire-rs/FR-085)).
    `--design` replaces the relation selection, so it conflicts with `--relation`. The matrix design column
    and the strict gate over it are separate requirements; `--strict` keeps FR-030''s meaning.'
- id: FR-041#behavior-human
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '- Human output is FR-030''s, one block per design relation, one line per requirement: the
    requirement id, its `state`, then each design element as `id (kind)` with its repo when present, and
    the criterion ids it realizes. An `uncovered` requirement is marked `no design element` and names
    the relation''s `check` token. Dangling and kind-mismatch entries print beneath the row as in FR-030.'
- id: FR-041#behavior-json
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '- `--json` writes the quire-rs join on stdout and nothing else, and honours `--pretty` ([FR-008](./FR-008-json-output-encoding.md)).'
- id: FR-041#behavior-errors
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '- A module set declaring no `design_relations` is an invalid request: exit 3 naming the missing
    declaration, computing nothing. `--design` together with `--relation` exits 3 naming both flags.'
- id: FR-041-AC-1
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '`quire relations --scope <DIR> --module $M --design --json` over a fixture with one requirement
    realized by a design element and one with none emits only the join: the first row `covered` with the
    element''s `id` and `archetype` in `linked`, the second `uncovered` with empty `linked`.'
- id: FR-041-AC-2
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: In the same run two executions are byte-identical, and a relation declared in the module but
    absent from `design_relations` produces no block.
- id: FR-041-AC-3
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: Default human output prints each element as `id (kind)`, prints realized criterion ids, and
    marks an `uncovered` requirement `no design element` with the relation's `check` token.
- id: FR-041-AC-4
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '`--repo o/a=<A> --repo o/b=<B> --design` over a fixture whose requirement in `o/a` is realized
    by an element in `o/b` shows that element with repo `o/b`; without the `o/b` flag the requirement
    is a gap and the reference prints under the row''s dangling entries as authored.'
- id: FR-041-AC-5
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: A module set with no `design_relations` exits 3 naming the missing declaration, with no report
    on stdout.
- id: FR-041-AC-6
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '`--design` with `--relation` exits 3 naming both flags.'
- id: FR-041-AC-7
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: The CLI source holds no requirement or design archetype name and no realizing verb, and delegates
    the join to quire-rs.
- id: FR-041#relationships
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: "relationships:\n  - target: \"ix://agent-ix/quire-cli/spec/stakeholder/StR-004\"\n    type:\
    \ \"implements\"\n    cardinality: \"1:1\"\n  - target: \"ix://agent-ix/quire-rs/spec/functional/FR-086\"\
    \n    type: \"references\"\n  - target: \"ix://agent-ix/quire-cli/spec/functional/FR-030\"\n    type:\
    \ \"extends\""
- id: FR-041#dependencies
  path: spec/functional/FR-041-relations-design-mode.md
  role: examined
  excerpt: '- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); [FR-030](./FR-030-relations-subcommand.md);
    quire-rs [FR-086](ix://agent-ix/quire-rs/FR-086), [FR-085](ix://agent-ix/quire-rs/FR-085) - **Downstream**:
    the matrix design column (PLAT-1529)'
- id: spec/functional/index.md
  path: spec/functional/index.md
  role: examined
  excerpt: '* [FR-041: quire relations design mode](./FR-041-relations-design-mode.md)'
- id: FR-030
  path: spec/functional/FR-030-relations-subcommand.md
  role: context_only
  excerpt: The CLI SHALL provide a `relations` subcommand that renders the two-way relation coverage report
    of quire-rs FR-084 over one repository or over several repositories loaded as one corpus (FR-085).
- id: FR-007
  path: spec/functional/FR-007-exit-codes.md
  role: context_only
  excerpt: '3 | Invalid request: invalid argv, missing or incorrectly typed input paths, malformed input,
    unknown archetypes, unresolved selection, module load or structural validation failures ...'
- id: StR-004
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: context_only
  excerpt: For artifact operations, the CLI SHALL be a thin process boundary over quire-rs. Every observable
    artifact behavior ... SHALL trace to a quire-rs FR.
- id: quire-rs FR-086 (agent-ix/quire-rs#668, unmerged)
  path: agent-ix/quire-rs:spec/functional/FR-086-design-element-join.md
  role: context_only
  excerpt: 'Where the authored target carries a #fragment equal to an acceptance-criterion id of that
    requirement, the resolved linked entry carries "criteria": ["<id>", ...] ... A corpus whose loaded
    modules declare no design_relations yields {"relations": []}.'
- id: quire-rs FR-084
  path: agent-ix/quire-rs:spec/functional/FR-084-two-way-relation-coverage-report.md
  role: context_only
  excerpt: 'Document (the obliged document and a resolved linked entry): {"id", "path", "archetype", "repo",
    "verification", "criteria"} ... criteria is {"backed": n, "unbacked": n, "not_measured": n} and is
    always present'
```

## Verdict

Pass with findings. FND-001 to FND-003 are real gaps an implementer would trip on; none contradicts FR-007 or StR-004. No compatibility layer, fallback or pin is introduced.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (head 3bcab5d): `--design --strict` is an invalid request (exit 3 naming both flags), stated in prose and AC-9; consistent with FR-007 class 3. |
| FND-002 | fixed | Round 1 (head 3bcab5d): Human layout keeps FR-030's state and verification, ends each block with the summary, and AC-4/AC-5 test both. |
| FND-003 | fixed | Round 1 (head 3bcab5d): Undeclared design_relations entry exits 3 naming the entry with no stdout (Behavior + AC-8), matching FR-086-AC-10 at ad0986d (join fails, no report). |
| FND-004 | fixed | Round 1 (head 3bcab5d): Split into AC-2 (stdout-only, determinism) and AC-3 (relation selection). |
