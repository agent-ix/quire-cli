---
id: FR-044
title: "quire relations flow mode"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-096"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-057"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-084"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-030"
    type: "extends"
---

# FR-044: quire relations flow mode

## Description

`quire relations` ([FR-030](./FR-030-relations-subcommand.md)) SHALL accept `--flow`, which renders
the flow report of quire-rs [FR-096](ix://agent-ix/quire-rs/spec/functional/FR-096): the loaded modules' `flow_rules`,
the facts they were evaluated over, keyed by the archetype names those rules declare, and one denial
for each subject a rule denies. The rules, their
evaluation, the facts, the denials, their order and the `quire.flow-report/v1` payload belong to
quire-rs. The CLI is a pass-through: it resolves roots, applies path safety, calls the engine, and
renders what the engine returns ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)).
It holds no domain rule, and no archetype, edge verb, field, category, trust level, sink kind or
remediation name: which rules exist comes from the loaded modules' `traceability.flow_rules`.

```
quire relations (--scope <DIR> | --repo <ORG/REPO>=<DIR>...) [--module <PATH>]... --flow [--json] [--strict]
```

`--flow` combines with `--scope`, `--repo`, `--module` and `--json` exactly as FR-030 defines them.
`--flow` replaces the relation selection, so it conflicts with `--relation` and with `--design`
([FR-041](./FR-041-relations-design-mode.md)). Unlike `--design`, it accepts `--strict`, with the
meaning given below.

Without `--flow`, `quire relations` renders the engine's relations report as FR-030 defines it. When
the engine's report carries the flow section of quire-rs
[FR-084](ix://agent-ix/quire-rs/spec/functional/FR-084) (the `quire.flow-report/v1` document, present
when the loaded modules declare `flow_rules`), the CLI renders that section unaltered beside the
relations rows, in both output forms. Its denials do not change the exit status of a default run: a
default run exits as FR-030 defines. `--flow` renders that same document as the whole report, and it
alone applies the exit rules below. `--flow` does not require `required_relations`: the FR-030
invalid request for a module set with no `required_relations` does not apply under `--flow`, whose
own precondition is at least one `flow_rules` declaration.

A denial is a rule outcome, not a CLI failure: the CLI never alters,
suppresses, reorders, re-spells or adds to an engine value. The `--severity` lowering of a
`trace:<check>` finding (quire-rs [FR-057](ix://agent-ix/quire-rs/spec/functional/FR-057)) is a `quire validate`
behaviour ([FR-004](./FR-004-validate-subcommand.md)); the flow report lists every denial the engine
produced.

## Behavior

- Human output goes to stdout: one row per denial, in the engine's order, carrying the rule's
  `check` token, the subject id, the resolved severity, the rule's rationale, the matched facts (the
  ids of the matched documents, such as classifications, boundaries and sinks) and the remediation
  names. A run with no denial prints no
  denial row.
- `--json` writes the quire-rs document `quire.flow-report/v1` on stdout and nothing else, and
  honours `--pretty` ([FR-008](./FR-008-json-output-encoding.md)). The document is the complete engine
  report, unaltered and key for key: `schema`, `rules`, `vocabularies`, `facts` (keyed by the
  archetype names declared in the loaded rules), `denials` (each with `rule`, `check`, `subject`,
  `matched`, `remediations`, `rationale` and `severity`) and the sibling `remediations`. Two runs over
  identical inputs are byte-identical, the denial order being the engine's: `check`, then subject
  (`repo`, then `id`, an absent `repo` first), then rule `name`.
- The command always emits the complete report first, then sets the exit status. With `--strict` it
  exits 1 when the report holds any denial, whatever its severity (warning included), and exits 0 only
  when it holds none. Without `--strict` the exit follows the severity of the denials as
  `quire validate` does (quire-rs FR-096): 1 when the report holds a denial of `error` severity, 0
  when it holds none or only `warning` denials. A denial dropped by `--severity ... off` is not in the
  report and so does not count.
- Each of these is an invalid request: exit 3 ([FR-007](./FR-007-exit-codes.md)) naming the cause and
  writing no report on stdout. A module set declaring no `flow_rules`, naming the missing
  declaration; a module that fails the quire-rs load-time validation of `flow_rules` or
  `remediations` (an `any_of` value outside the field's enum, an undeclared remediation, more than two
  path steps, a duplicate rule name, a remediation whose change matches none of the rule's
  `unless_any`), reproducing the engine's diagnostic that names the rule; `--flow` with `--relation`
  or with `--design`, naming both flags.
- A corpus with rules but no connection carrying a classification yields an empty `denials` list and
  exit 0, with or without `--strict`; an absent data-flow model is not an error.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-044-AC-1 | `quire relations --scope <DIR> --module $M --flow --json` over a fixture where `CONN-1` carries a classification with `regulated_categories` including `health` and crosses a boundary with `trust_level: untrusted`, with no control protecting it, emits one denial whose `check` is `health-across-untrusted-boundary` and whose `subject`, `matched` and `remediations` name `CONN-1`, the classification, the boundary and `encrypt-in-transit`, and whose `rationale` and `severity` equal the engine's. | Test |
| FR-044-AC-2 | The human output of the same run has exactly one denial row, carrying the `check`, `CONN-1`, the resolved severity, the rule's rationale, the classification and boundary ids, and the remediation name `encrypt-in-transit`. | Test |
| FR-044-AC-3 | The same fixture plus a control with `safeguards: [encrypt-in-transit]` that `protects` `CONN-1` yields an empty `denials` list in `--json` output and no denial row in human output. | Test |
| FR-044-AC-4 | A fixture with a sink of `kind: llm_prompt` and `processor_agreement: not_covered`, fed by a connection carrying `health`, yields one denial under the module's LLM-sink rule; the same sink with `processor_agreement: covered` yields none, and with `unknown` yields one. | Test |
| FR-044-AC-5 | `--json` output is byte-equal to the quire-rs `quire.flow-report/v1` document for the same inputs, holding `schema`, `rules`, `vocabularies`, `facts`, `denials` (each with `rule`, `check`, `subject`, `matched`, `remediations`, `rationale` and `severity`) and `remediations`, with `facts` keyed by the archetype names declared in the loaded rules; stdout holds only that document. | Test |
| FR-044-AC-6 | Two `quire relations --scope <DIR> --module $M --flow --json` runs over a corpus with at least two denials are byte-identical, and the denials appear in engine order: `check`, then subject `repo` and `id`, then rule `name`. A `--repo` fixture with denials on `o/a` subject `Z` and `o/b` subject `A` lists `o/a:Z` first, and two rules sharing a `check` on one subject appear in rule-name order. | Test |
| FR-044-AC-7 | Over a corpus with one or more denials, `--strict` emits the complete report and exits 1, including when every denial is of warning severity; over a corpus with none, `--strict` exits 0. Without `--strict`, a corpus with a denial of `error` severity exits 1, one whose denials are all `warning` exits 0, and one with none exits 0; the complete report is on stdout in every case. | Test |
| FR-044-AC-8 | Over a fixture module `$M` declaring starter-shaped rules (a connection-across-boundary rule and an LLM-sink rule, with fixture-local names), a corpus with no connection carrying a classification yields no denial, in both output forms, and exits 0 with and without `--strict`. | Test |
| FR-044-AC-9 | A module whose rule names `medical` in `any_of` for `regulated_categories` makes `--flow` exit 3 with a diagnostic naming the rule and the value `medical`, and nothing on stdout. | Test |
| FR-044-AC-10 | Each of an undeclared remediation, a rule with three path steps, a duplicate rule name, and a remediation whose change matches none of the rule's `unless_any` makes `--flow` exit 3 with the engine's diagnostic naming the rule, and nothing on stdout. | Test |
| FR-044-AC-11 | A module set declaring no `flow_rules` exits 3 naming the missing declaration, and `--flow` with `--relation` or with `--design` exits 3 naming both flags, each with nothing on stdout. | Test |
| FR-044-AC-12 | `--repo o/a=<A> --repo o/b=<B> --flow` over a fixture whose connection in `o/a` crosses a boundary in `o/b` reports the denial using the `o/b` boundary; with the `o/b` flag omitted the `crosses` edge has no target and contributes nothing, so the report holds no denial and both `--flow` and `--flow --strict` exit 0. | Test |
| FR-044-AC-13 | With `--severity trace:health-across-untrusted-boundary=warning`, `quire validate` over the AC-1 fixture reports the denial as a warning and exits 0, while `quire relations --flow --json` over it still lists the denial. | Test |
| FR-044-AC-14 | Over a fixture whose archetype, edge verb, field, category, trust-level and remediation names are unrelated to data flow, `--flow` renders the complete engine report in both output forms. | Test |
| FR-044-AC-15 | The CLI source delegates evaluation to quire-rs: it holds no rule, edge-matching, condition, denial or sorting logic and no archetype, verb, field, category or remediation name. | Inspection |
| FR-044-AC-16 | A module set declaring `flow_rules` and no `required_relations` exits 0 under `--flow` and renders the report; the same module set without `--flow` follows FR-030. Default `quire relations --json` over a module set declaring `flow_rules` carries the engine's flow section unaltered, and its denials do not change the default run's exit status. | Test |

## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); [FR-030](./FR-030-relations-subcommand.md); [FR-007](./FR-007-exit-codes.md); [FR-008](./FR-008-json-output-encoding.md); quire-rs [FR-096](ix://agent-ix/quire-rs/spec/functional/FR-096), [FR-084](ix://agent-ix/quire-rs/spec/functional/FR-084), [FR-057](ix://agent-ix/quire-rs/spec/functional/FR-057). FR-096 and FR-084 are open on quire-rs PR 691; this requirement merges with or after it
- **Downstream**: the QSL lowering of the flow facts (PLAT-1541), which reads `quire.flow-report/v1`
