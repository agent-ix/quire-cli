---
id: FR-025
title: "Rust-owned CLI qualification tooling"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/StR-004"
    type: "implements"
---

# FR-025: Rust-owned CLI qualification tooling

## Description

The repository SHALL expose one private Rust qualification package for its
assurance-traceability, benchmark, thin-boundary, and unsafe-comment gates.

## Inputs

- A Quire coverage JSON report and the repository-owned target set.
- A hyperfine JSON report and a finite non-negative millisecond threshold.
- A repository root containing Rust source and the unsafe-comment baseline.
- An explicit request to regenerate the unsafe-comment baseline, when desired.

## Outputs

- A zero status and a concise observation when the selected gate passes.
- A non-zero status with the rejected field, target, source locus, or measured
  threshold when the selected gate fails.
- An updated repository-relative unsafe-comment baseline only after an explicit
  update request.

## Behavior

The qualification package SHALL deserialize coverage and benchmark reports into
typed Rust input models before applying assertions.

When assurance traceability is checked, the package SHALL require every
repository-owned FR-020-AC-1 through FR-020-AC-9, IT-136 through IT-145,
TC-814, StR-004-VC-2, and StR-004-VC-3 target. When a required target is
unbacked, the package SHALL reject the report and name that target. When a
US-006 example row is promoted to binding acceptance, the package SHALL reject
the report and name that row. When relevant unmatched tags, status lies, or
untracked symbols exist, the package SHALL reject the report and name them.

When benchmark evidence is checked, the package SHALL compute nearest-rank p95
from the first result's non-empty times array and convert seconds to
milliseconds. When p95 exceeds the finite non-negative threshold, the package SHALL
fail and report both values.

When Rust source boundaries are checked, the package SHALL parse Rust syntax
with a maintained Rust parser. The package SHALL reject forbidden engine calls,
CLI-owned assurance models, and process execution outside their admitted
dispatch loci, including imports expressed through aliases or nested groups.

When unsafe comments are checked, the package SHALL associate every parsed
unsafe block with its source locus. When an unsafe block has no `SAFETY:` comment
within the three preceding source lines, the package SHALL reject it unless that
exact locus appears in the reviewed baseline.

When the unsafe baseline contains a locus that no longer lacks a qualifying
comment, the package SHALL reject the stale exemption. The package SHALL replace
the baseline only through an explicit update operation.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-025-CON-1 | The Rust-source gates SHALL NOT classify syntax with grep, regular expressions alone, or a hand-written lexer | Maintainability | Inspection + mutation (TC-822) |
| FR-025-CON-2 | Make targets SHALL contain orchestration only | Boundary | Static test (TC-825) |
| FR-025-CON-3 | Make targets SHALL NOT interpret coverage, benchmark, or Rust-source content | Boundary | Static test (TC-825) |
| FR-025-CON-4 | `npm/quire-cli/bin/quire.js` SHALL remain the admitted npm distribution launcher | Boundary | Static test (TC-826) |
| FR-025-CON-5 | `npm/quire-cli/bin/quire.js` SHALL NOT acquire qualification semantics | Boundary | Static test (TC-826) |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-025-AC-1 | A complete backed assurance report passes, while each missing, unbacked, wrongly promoted, unmatched, lying, or untracked case fails naming the condition | Test (TC-815, TC-816, TC-817) |
| FR-025-AC-2 | Missing fields, wrong field types, malformed JSON, empty benchmark results, and non-finite or negative thresholds fail before a success observation is emitted | Test (TC-818, TC-820) |
| FR-025-AC-3 | Nearest-rank p95 is computed deterministically and values on either side of the configured threshold produce opposite verdicts | Test (TC-819, TC-821) |
| FR-025-AC-4 | The thin-boundary gate accepts admitted dispatch and rejects forbidden direct, aliased, and nested-group references with a file and syntax locus | Test (TC-822, TC-823) |
| FR-025-AC-5 | The unsafe gate accepts a documented block, rejects an undocumented block, honours only exact baseline loci, rejects stale exemptions, and changes the baseline only through the explicit update operation | Test (TC-824) |
| FR-025-AC-6 | Repository Make targets invoke Rust-owned gates without inline report parsing or source classification | Test (TC-825) |
| FR-025-AC-7 | No executable Python or shell qualification file remains, while the npm launcher and non-executable fixture/data formats remain | Test (TC-826) |

## Dependencies

- **Upstream:** [FR-020](./FR-020-assurance-export-subcommand.md) defines the
  assurance surface whose repository traceability is checked.
- **Upstream:** [NFR-003](../non-functional/NFR-003-zero-unsafe.md) defines the
  unsafe documentation policy.
- **Downstream:** [NFR-009](../non-functional/NFR-009-native-qualification-boundary.md)
  constrains the implementation and local evidence.
