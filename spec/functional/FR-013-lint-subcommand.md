---
id: FR-013
title: "quire lint subcommand"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-036"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-005"
    type: "requires"
    cardinality: "1:1"
---

## Description

The CLI SHALL expose a `lint` subcommand that surfaces quire-rs declarative lint
rules (advisory authoring-convention findings, distinct from structural
`validate`) against a markdown document, emitting findings on stderr and
delegating all rule evaluation to `quire-rs`. The behavioral surface is specified
below.

## Behavior

The CLI SHALL expose a `lint` subcommand:

```
quire lint <DOC.md|-> --module <PATH> [--archetype <NAME>]
```

It surfaces quire-rs declarative lint rules (upstream FR-036): the module's
`manifest.yaml` `lint_rules:` are evaluated against the parsed document. Lint
is **advisory** — a posture distinct from `validate` ([FR-004](./FR-004-validate-subcommand.md)): findings flag
authoring-convention drift (e.g. an AC `Verification` cell outside the
ISO 29148 vocabulary, a `Configuration` `Scope` cell outside
`creation`/`runtime`/`session`), never structural invalidity, and lint never
gates extraction or sync.

1. Path-safety ([FR-005](./FR-005-path-safety.md)) on the document path and `--module`; `-` reads stdin.
2. Load the module via the shared eager-failure loader helper ([FR-004](./FR-004-validate-subcommand.md) CR
   note): a missing `manifest.yaml` exits 1 with the real reason.
3. Resolve the archetype **tolerantly** for rule scoping only: `--archetype`
   override, else frontmatter `type`, else unresolved — an
   unresolved archetype runs unfiltered rules and is NOT an error
   (upstream FR-036-AC-3).
4. Evaluate `lint_document(registry.lint_rules(), archetype, doc)`.
5. Emit each finding on **stderr** as `<severity>: <rule-id>: <message>`
   (kind `LintWarning` / `LintError` in `--diagnostics-format=json`).
   NEVER writes stdout.
6. Exit 0 when there are no findings or only `warning`-severity findings;
   exit 1 when any `error`-severity finding fires.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-013-AC-1 | `quire lint clean.md --module $M` against a conforming document exits 0 with empty stdout and empty stderr | Test |
| FR-013-AC-2 | Against a document with a `warning`-severity finding, the command exits 0 and stderr carries `warning: <rule-id>:` plus the offending value; stdout stays empty | Test |
| FR-013-AC-3 | Against a document with an `error`-severity finding, the command exits 1 and stderr carries `error: <rule-id>:` | Test |
| FR-013-AC-4 | `--archetype <NAME>` overrides scoping: a rule filtered to `archetypes: [FR]` emits nothing when the document is linted as `NFR` | Test |
| FR-013-AC-5 | A `--module` path without `manifest.yaml` exits 1 naming the missing manifest (shared eager-failure loader), not `UnknownArchetype` | Test |

## Dependencies

- **Upstream**: quire-rs FR-036 (declarative lint rules); [FR-005](./FR-005-path-safety.md) path-safety.
- **Downstream**: authoring agents that consume advisory lint findings.
