---
id: FR-020
title: "quire assurance subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-006"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-067"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-068"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL provide an `assurance` subcommand that exposes quire-rs's
implemented `assurance-v1` export through a thin offline process boundary.

```text
quire assurance --scope <DIR> --module <PATH>
                --repository <IDENTITY> --revision <FULL_SHA>
                --expect-module <NAME@VERSION>
                [--expect-schema <MODULE/ARCHETYPE@SHA256>]...
```

The exact module and expected premises are mandatory. An explicit empty
`--expect-schema` set means that the expected module has no active archetype;
it does not disable schema checking.

## Inputs

- `--scope` is the repository root. Documents are loaded only from
  `<scope>/spec`; source symbols are extracted from `<scope>` with `spec/` and
  the module-declared `source_exclude` globs excluded.
- `--module` names one exact module directory containing `manifest.yaml`; the
  command performs no environment/default module discovery and no lazy
  installation.
- `--repository` is the caller-selected non-empty repository identity copied
  into the upstream envelope.
- `--revision` is the caller-selected 40-character lowercase Git object id.
  The command records it but SHALL NOT invoke Git to discover or verify it.
- `--expect-module` names the only accepted module and semantic version.
- Each `--expect-schema` names one accepted active-archetype SHA-256 digest.
  The supplied set is compared exactly with the set emitted by quire-rs.

## Behavior

1. The command applies the existing path-safety guards, loads the exact module,
   and derives the existing two roots from `--scope`.
2. It constructs `Spec`, `SymbolExtraction`, and `SymbolGraph` only through the
   authoritative quire-rs loaders and binder. It then calls
   `build_assurance_export`; no CLI-owned artifact, obligation, symbol,
   relation, availability, or freshness logic is permitted. A module without a
   `traceability:` model is valid: static artifacts and symbols are exported,
   while obligations and `verifies`/`implements` relations are empty.
3. It validates the completed bytes with `read_assurance_export` and the
   caller's accepted module/schema premises, and separately requires the
   accepted set to equal the emitted set so an unused extra premise cannot pass.
4. Only after construction, upstream schema validation, and exact-premise
   comparison succeed does it write JSON to stdout. Compact output is the exact
   `AssuranceExport::to_json_bytes()` value plus one newline. Global `--pretty`
   parses and re-indents those validated compact bytes, changes whitespace only,
   and remains deterministic.
5. Registry, corpus, and symbol-extraction diagnostics use the established
   stderr channel. An error exits non-zero with empty stdout; a successful
   export may legitimately contain empty record arrays and still exits zero
   with a complete envelope.

> **CR note (engine pin advances, 2026-09-06, agent-ix/quire-rs#405):**
> CON-1 named merge `e3352a0644abcfd5f0ebad348bc7aca235925ecc` —
> the commit that implemented the upstream assurance FRs. The pin advances to
> `a874fb641cb70da83c8c8b23f9fea0a44255b88a`, still quire-rs 0.46.0 and a
> descendant of that merge, which carries `Registry::load_module_set` for the
> repeatable `--module` in [FR-017](./FR-017-coverage-subcommand.md)-AC-20. The
> assurance contract is unchanged: the same owned `assurance-v1` schema, the
> same closed payload, nothing vendored. CON-1 names the pin the CLI ACTUALLY
> carries, not the commit that introduced the feature — a constraint that
> lagged the manifest would make IT-145, whose whole job is to catch
> disagreement between the two, agree with a number nobody runs. The
> Dependencies section still names the implementing merge, which stays true.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-020-CON-1 | The CLI SHALL pin quire-rs 0.46.0 at revision `616a7e97c0e8c84aedda71dc198e94e3de3d9da6` and use its owned `assurance-v1` schema without vendoring or generating another schema. | Compatibility | Inspection |
| FR-020-CON-2 | The command SHALL execute no test, proof, solver, consumer, package-manager, Git, or network command. It performs parsing and static source extraction only. | Responsibility | Test |
| FR-020-CON-3 | The CLI SHALL add no verdict, execution result, evidence freshness claim, generic evidence envelope, or tool-provenance field to the closed upstream payload. | Responsibility | Inspection |
| FR-020-CON-4 | Unknown or malformed module versions, schema premises, source revisions, and incomplete module loads SHALL fail closed before any stdout byte. | Integrity | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-020-AC-1 | A pinned fixture containing artifacts, obligations, symbols, resolved and dangling corpus relations, `verifies` and `implements` bindings, locators, relation-kind capabilities, and `available`, `missing`, `not_applicable`, and `unknown` observations emits a complete `quire-assurance` v1 document that validates against `quire_rs::assurance::ASSURANCE_V1_SCHEMA`. | Test (IT-136, IT-137) |
| FR-020-AC-2 | Two compact runs over identical fixture bytes and arguments produce byte-identical stdout; `--pretty` changes only whitespace and is independently byte-identical across runs. | Test (IT-138) |
| FR-020-AC-3 | A mismatched module name/version, missing or extra schema premise, wrong schema digest, malformed premise syntax, unnamed/unversioned module, or unsupported source revision exits non-zero with empty stdout and a diagnostic naming the refused premise. | Test (IT-139, IT-140) |
| FR-020-AC-4 | A valid corpus with zero artifacts, obligations, symbols, or relations still emits the complete successful envelope and exits zero. A module without a `traceability:` model exports its static artifacts and symbols with empty obligations and no `verifies`/`implements` relations. A document the corpus walker cannot read remains a successful export with an `unknown` relation observation and non-empty reason as quire-rs FR-068 requires; a missing root, invalid module/source premise, or export-wide upstream error exits non-zero with empty stdout. | Test (IT-141) |
| FR-020-AC-5 | Module-loader and symbol-extraction diagnostics are emitted on stderr in human or JSON diagnostic form and never enter the assurance payload. | Test (IT-142) |
| FR-020-AC-6 | The command delegates construction to `build_assurance_export`, validation to `read_assurance_export`, corpus loading to `Spec`, extraction to `extract_tree_scoped`, and binding to `trace::bind`; a static boundary audit rejects a second graph, schema, or direct parser in the CLI. | Inspection (TC-814) |
| FR-020-AC-7 | The command opens no network socket and spawns no child process on success or refusal paths. | Test (IT-143) |
| FR-020-AC-8 | A checked-in golden JSON fixture validates against the upstream schema in Rust, is consumed from the exact same bytes by required Node/TypeScript and Python compatibility probes without normalization, and pins every v1 field and state token. A missing probe runtime fails the gate. | Test (IT-144) |
| FR-020-AC-9 | `--help`, README, changelog, linked-engine capability reporting, and the exact Cargo revision pin consistently describe the assurance command and quire-rs compatibility boundary. | Test (IT-145) |

## Dependencies

- **Upstream**: quire-rs [FR-067](ix://agent-ix/quire-rs/FR-067) and
  [FR-068](ix://agent-ix/quire-rs/FR-068), implemented by
  `agent-ix/quire-rs#389` at merge
  `e3352a0644abcfd5f0ebad348bc7aca235925ecc`; current compatible engine pin
  `616a7e97c0e8c84aedda71dc198e94e3de3d9da6` (crate version 0.46.0).
- **Ownership gate**: `agent-ix/engineering-assurance#5`, accepted before this
  command was specified.
- **Downstream**: `agent-ix/quoin#322` and the common compatibility fixtures in
  `agent-ix/engineering-assurance#9` consume this static export; they own
  retention, audit, attestations, and verdicts.
