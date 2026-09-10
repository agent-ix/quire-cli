---
id: SR-058
title: "Base review of the Rust-1.98.1 engine-pin follow-up"
type: SpecReview
analysis: base
scope: "spec/functional/FR-020-assurance-export-subcommand.md, spec/non-functional/NFR-007-exact-qualified-rust-toolchain.md, spec/tests.md, agent-ix/quire-cli#82"
review_set: base
---

## Summary

Preimplementation follow-up review triggered when #82's dependency update made
IT-145 refuse the stale normative engine SHA. The FR-020 change advances only
the exact compatible quire-rs revision to the #422 merge already qualified on
Rust 1.98.1; the assurance schema, payload, behavior, and ownership boundary do
not change.

## Verdict

**PASS.** The new SHA is exact, is both the current quire-rs main revision and
the merge identity of PR #422, remains quire-rs 0.46.0, and is consistently
specified for the manifest, lockfile, changelog, contract test, and dependency
section. The existing IT-145 agreement test is intentionally retained and must
turn green only after all executable and normative surfaces agree.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | low | No issues found; the exact-pin update is a compatibility-coordinate change required by NFR-007 and introduces no assurance semantic or interface change. | FR-020-CON-1; FR-020-AC-9; NFR-007-AC-2; IT-145 |

## Validation and evidence checks

- Changed-file `quire validate` passes for FR-020, NFR-007, and the Test Matrix.
- GitHub reports `85dfe9d5a937c52af6456f2e6aa3a6bc4c82db9f` as both
  quire-rs `main` and PR #422's merge commit.
- The first exact-pin test run failed at IT-145 on the stale constant, proving
  the compatibility agreement guard is load-bearing.
- No hosted CI dispatch, release, publication, or WASM work is authorized.

## Implementation authorization

Update the manifest, lockfile, IT-145 constant, current FR-020 compatibility
coordinate, and current changelog entry to the reviewed SHA. Historical entries
must retain their historical revisions. Any assurance payload or command
behavior change requires a new specification cycle.
