---
id: SR-061
title: "Base review of the exact-Rust policy-gate remediation"
type: SpecReview
analysis: base
scope: "NFR-007; TC-142; TC-143; SR-060 FND-001 through FND-003 remediation"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/spec/non-functional/NFR-007"
    type: reviews
---

## Summary

The base checklist remains satisfied without changing NFR-007. Its four
acceptance criteria retain complete TC-142/TC-143 coverage and valid links. The
remediation makes the existing repository-policy evidence fail closed for an
action missing its revision and for all eight governed Makefile tokens; it also
replaces the ix-trace-rs tag with the already-resolved immutable revision.

No product CLI behavior, public API, wire contract, state transition, or new
acceptance criterion is introduced. The changes strengthen the existing local
qualification boundary and remain explicitly outside hosted CI.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Closed: `uses: actions/checkout` without `@revision` now produces the same full-SHA finding as a mutable revision, and deleting a revision from a production action is mutation-tested. | `tests/toolchain_policy.rs:126`; `tests/toolchain_policy.rs:163`; `tests/toolchain_policy.rs:457`; NFR-007-AC-1 | implementation-bug-despite-evidence |
| FND-002 | medium | Closed: eight Makefile mutations cover every governed locked Cargo command, cargo-deny invocation, and the aggregate cargo-audit token with rule-specific findings. | `tests/toolchain_policy.rs:377`; `tests/toolchain_policy.rs:502`; NFR-007-AC-3 | correct-requirement-no-evidence |
| FND-003 | low | Closed: ix-trace-rs now uses the immutable commit already recorded in Cargo.lock instead of a movable tag. | `Cargo.toml:28`; `Cargo.lock:673` | correct-requirement-no-evidence |
