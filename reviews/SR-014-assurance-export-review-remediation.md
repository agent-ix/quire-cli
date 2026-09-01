---
id: SR-014
title: "Code review remediation — PR #75"
type: SpecReview
analysis: code-review
scope: "PR #75 external findings FND-022 through FND-029; remediation at issue/74-assurance-export"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/spec/functional/FR-020
    type: reviews
  - target: ix://agent-ix/quire-cli/reviews/SR-012
    type: references
---

# SR-014: Code review remediation — PR #75

## Summary

The independent review at `c5f8c43f` found three traceability/gating defects,
two portability/serialization weaknesses, and three low contract/documentation
gaps. All eight findings are dispositioned in code, tests, specifications, and
the local composite gate. The one suggested repair not adopted verbatim was
promotion of US-006 examples to acceptance criteria: the authoritative module
pair does not mint user-story acceptance, and `spec-artifacts-iso` flags
`Acceptance Criteria` on US artifacts as drift. The pseudo criteria were
therefore removed and normative acceptance remains in FR-020.

## Verdict

**PASS after fixes.** No FND-022..029 finding remains open.

## Assurance Context

- **Decision and claim boundary:** the CLI exports source-grounded static facts;
  it does not execute evidence or assign verdicts.
- **Authoritative policy and schema:** quire-rs FR-067/FR-068 and its closed
  assurance-v1 schema own projection and validation; the installed
  `spec-artifacts-process` and `spec-artifacts-iso` modules own trace minting
  and artifact grammar policy.
- **Trust inputs:** caller-supplied repository identity, full revision, exact
  module/version, and complete active-schema digest set.
- **Failure posture:** build, upstream validation, and premise equality finish
  before stdout; missing Python, Node, or Linux strace fails the evidence gate.
- **Execution boundary:** the command performs static reads only and delegates
  graph/schema behavior to quire-rs.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-022 | high | **FIXED by policy-correct disposition.** US-006 now labels its examples illustrative, carries no fabricated `US-006-AC-*` IDs, and explicitly delegates binding acceptance to FR-020. The matrix no longer claims user-story criteria. | spec/usecase/US-006-export-assurance-facts.md; spec/tests.md |
| FND-023 | high | **FIXED.** Canonical `// Trace:` comments without an absorbed trailing period bind FR-020-AC-1..9; the executable coverage gate requires all nine backed. | tests/cli_assurance.rs; scripts/check_assurance_traceability.py |
| FND-024 | high | **FIXED.** `make spec` validates the changed assurance artifacts, exports coverage, checks all issue targets, and is a dependency of `make ci`. | Makefile; scripts/check_assurance_traceability.py |
| FND-025 | medium | **FIXED.** IT-143 requires strace and IT-144 requires Python/jsonschema and Node; missing tools fail rather than return a green no-op. | tests/audit_no_network.rs; tests/assurance_cross_language.rs |
| FND-026 | medium | **FIXED.** Pretty output lexically re-indents the already validated compact bytes, preserving every non-whitespace byte and key order and eliminating the independent object-serialization path. IT-138 removes only insignificant whitespace and requires the exact upstream bytes. | src/io.rs; src/commands/assurance.rs; IT-138 |
| FND-027 | low | **FIXED.** Expected schemas split on the rightmost slash, with a unit regression for slash-bearing module identities. | src/commands/assurance.rs |
| FND-028 | low | **FIXED.** FR-020 states that a missing traceability model is valid, and IT-141 pins static artifact/symbol output with empty obligations and no verifies/implements relations. | spec/functional/FR-020-assurance-export-subcommand.md; tests/cli_assurance.rs |
| FND-029 | low | **FIXED.** Review and PR evidence use the observed 214-test remediation total rather than the stale 197 count; SR-012 retains its historical 213 count for `c5f8c43f`. | SR-010; SR-012; PR #75 |

## Verification

- Targeted assurance, cross-language, contract, and static suites pass.
- Linux IT-143 passes with strace actually executed.
- Five trace-gate mutations (unbacked target, fabricated US criterion,
  unmatched FR tag, status lie, and missing report field) are all caught.
- `make spec` and `make ci` are the closing local gates; their exact results are
  recorded in the PR re-review request.
