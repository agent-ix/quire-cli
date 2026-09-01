---
id: SR-010
title: "Assurance-export CLI implementation code review"
type: SpecReview
analysis: code-review
scope: "origin/main..issue/74-assurance-export: src/commands/assurance.rs, Cargo.toml, Cargo.lock, src/engine.rs, tests/cli_assurance.rs, tests/assurance_cross_language.rs, tests/audit_no_network.rs, tests/audit_static.rs, scripts/check_thin_boundary.sh, public documentation and snapshots"
review_set: subset
---

## Summary

Local pre-PR review of Quire CLI issue #74 against `origin/main` at
`4f6ed024`. The review followed the Rust review and assurance-context lenses:
fail-closed premise handling, atomic stdout, thin-boundary ownership, path and
process seams, deterministic bytes, diagnostics separation, dependency pinning,
and whether the tests fail under load-bearing mutations. The reviewed
implementation pins quire-rs 0.46.0 at merge
`e3352a0644abcfd5f0ebad348bc7aca235925ecc` and delegates construction and
validation to the upstream assurance API.

## Verdict

**PASS after fixes and external-review remediation.** No open correctness,
safety, compatibility, or ownership finding remains. The capability snapshot,
canonical source-symbol trace bindings, fixture/golden EOF normalization,
fail-closed compatibility probes, and validated-byte pretty path are fixed.
Both deliberate mutations were caught by the intended tests and then reverted.
The exact repository `make ci` gate passes with 214 tests and zero failures.

## Assurance Context

- **Decision and claim boundary:** this change emits static source-grounded
  facts only. It does not execute evidence, retain results, assign verdicts,
  attest, or publish a second evidence envelope.
- **Authoritative policy and schema:** Engineering Assurance #5, contract IR
  #38, quire-rs FR-067/FR-068 and its closed `assurance-v1` schema. No
  `AssuranceProfile` exists in this repository, so no profile-specific control
  set is applicable.
- **Trust inputs:** caller-selected repository identity, a full lowercase
  revision, one exact module/version, and the complete active schema-digest set.
  The revision is recorded and shape-validated, not discovered or vouched for.
- **Failure posture:** all fallible construction, upstream schema reading, and
  exact-premise comparison complete before the single stdout write. Refusals
  leave stdout empty.
- **Execution boundary:** the command performs filesystem parsing and static
  symbol extraction only. Linux strace observed no network syscall and no child
  `execve` beyond the initial CLI process on success and refusal paths.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | medium | **FIXED.** Adding `assurance_export.v1` changed the shared engine capability envelope, but the extract snapshot initially retained the old list. The full suite caught IT-129; the snapshot now includes the sorted token and all six provenance tests pass. | tests/snapshots/extract-envelope.json; tests/cli_provenance.rs |
| FND-002 | medium | **FIXED.** The new IT-137..142, IT-144, IT-145, and TC-814 evidence ran but was not bound to source symbols. The first repair retained a trailing period that the trace grammar absorbed into the last ID; external review exposed the remaining gap. Canonical `// Trace:` lines now bind all of IT-136..145, TC-814, and FR-020-AC-1..9. | tests/cli_assurance.rs; tests/assurance_cross_language.rs; tests/cli_assurance_contract.rs; tests/audit_static.rs; SR-014 |
| FND-003 | low | **FIXED.** Five new text fixtures carried duplicate EOF blank lines; removing them changed the four source digests embedded in the golden. EOFs are normalized, the golden diff was reviewed as digest-only, and IT-137/IT-138 revalidate the exact bytes. | tests/fixtures/assurance/; IT-137; IT-138 |
| FND-004 | low | **FIXED.** Python and Node are required compatibility probes; a missing runtime now fails the gate. Linux IT-143 likewise requires `strace` instead of silently passing. | tests/assurance_cross_language.rs; tests/audit_no_network.rs; IT-143; IT-144 |

## Mutation checks

All mutations were made locally, their failures observed, and the production
source restored before the final gate.

| Mutation | Expected pin | Result |
|----------|--------------|--------|
| Remove the structural equality check after `read_assurance_export`, allowing an unused extra expected schema premise. | IT-139 refuses the extra premise. | **CAUGHT:** filtered IT-139 failed because the mutated command exited 0 instead of 1. |
| Remove the required final newline from compact stdout. | IT-138 exact golden bytes. | **CAUGHT:** filtered IT-138 failed its byte equality assertion. |

## Review notes

- `Registry::load_module` is used through the existing CLI loader, so sibling
  modules and environment/default discovery do not enter the premise set.
- The CLI parses only argv premise syntax. Artifact projection, obligations,
  symbol records, relation kinds, availability, sorting, schema ownership, and
  reader validation remain in quire-rs.
- `read_assurance_export` rejects emitted premises outside the accepted set;
  the additional structural equality check rejects accepted supersets. The
  latter is load-bearing under the first mutation above.
- Compact bytes come directly from `AssuranceExport::to_json_bytes`; pretty
  output is separately deterministic and semantically equal. No CLI engine
  provenance is appended to the upstream closed envelope.
- Request paths contain no `unwrap`, unsafe block, network client, Git lookup,
  package manager, runner, solver, or downstream consumer invocation.

## Recorded gates (2026-09-01)

- `env CARGO_TARGET_DIR=/tmp/quire-cli-74-target make ci` → exit 0: fmt,
  clippy `-D warnings`, 214 tests / 0 failures, licenses, bans, unsafe audit,
  thin-boundary audit, tool-drift audit, and specification traceability gate.
- `env CARGO_TARGET_DIR=/tmp/quire-cli-74-target cargo build --locked --release`
  → exit 0.
- Targeted Quire validation over SR-009, FR-020, US-006, StR-004, and the test
  matrix → `5/5 docs grammar-clean`, exit 0.
- No hosted CI workflow was dispatched.
