---
id: SR-057
title: "Base review of exact Rust 1.98.1 qualification"
type: SpecReview
analysis: base
scope: "spec/non-functional/NFR-007-exact-qualified-rust-toolchain.md, spec/non-functional/index.md, spec/tests.md, agent-ix/quire-cli#82"
review_set: base
---

## Summary

Preimplementation base review of the narrow quire-cli #82 toolchain contract.
After resolving two traceability and testability findings, NFR-007 defines one
exact Rust 1.98.1 policy, a mutation-sensitive exhaustive declaration audit,
an upstream-qualified engine pin, and a reproducible local qualification set.
It changes no CLI behavior and authorizes no hosted-CI dispatch or release.

## Verdict

**PASS after specification fixes.** The reviewed requirement is atomic at the
policy level, names every governed declaration surface, defines the evidence
needed to distinguish a real tool incompatibility from repairable formatting or
lint drift, and traces each acceptance criterion to planned evidence.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|----|----------|---------|------|--------------|
| FND-001 | medium | **FIXED before implementation.** The first draft summarized engine provenance and the qualification run in the NFR matrix row but gave AC-2 and AC-3 no individually addressable evidence row. TC-143 now owns the exact pinned-revision inspection and complete local gate record. | NFR-007-AC-2; NFR-007-AC-3; TC-143 | correct-requirement-no-evidence |
| FND-002 | medium | **FIXED before implementation.** “Locked qualification gates” did not identify the executable command set, so a partial test run could be reported as compliance. Verification now enumerates formatting, all-target/all-feature Clippy and tests, strict docs, release/static-binary, dependency, and static-audit commands and requires the record to retain their results. | NFR-007 Verification; NFR-007-AC-3; TC-143 | missing-requirement |

## Validation and boundary notes

- `quire validate` passes for every changed specification and review artifact.
- Whole-bundle validation still reports the pre-existing FR-021 structural
  defect (missing `Dependencies`); #82 neither caused nor silently repairs it.
- `agent-ix/quire-rs#422` is the upstream exact-1.98.1 qualification boundary.
  The implementation must select a descendant revision and retain its exact SHA.
- `.github/workflows` remains manual-dispatch-only. This review authorizes local
  verification of workflow declarations, not a hosted run.
- LR05/#61 remains downstream and separate: this review does not choose npm
  launcher or package-tooling architecture.

## Implementation authorization

Implementation may begin only from the commit containing NFR-007 and SR-057.
If a required command is incompatible, stop and return to the specification
cycle with reproduced evidence; do not weaken or silently omit the command.
