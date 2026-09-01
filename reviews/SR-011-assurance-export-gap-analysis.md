---
id: SR-011
title: "Assurance-export CLI gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli#74, FR-020, US-006, StR-004, NFR-004, IT-136..145, TC-814, issue/74-assurance-export implementation and local gates"
review_set: subset
---

## Summary

Companion gap analysis to [SR-010](./SR-010-assurance-export-code-review.md).
It compares issue #74 and FR-020 acceptance to the implemented process surface,
then distinguishes executed evidence from symbol-backed claims. The dependency
order remains valid after merged Quoin #324 and Engineering Assurance #13:
those changes do not move assurance projection, execution, retention, or verdict
ownership into this CLI.

## Verdict

**PASS after external-review remediation — no issue-#74 implementation gap
remains.** Every FR-020 criterion has a recorded passing run or static
inspection; US-006 carries illustrative examples and delegates binding
acceptance to FR-020 as the authoritative module requires. Quire's coverage
instrument now reports FR-020 9/9 and none of IT-136..145 or TC-814 in
`unbacked_rows`. Existing unrelated repository gaps remain unchanged and
`status_lies` is empty.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | medium | **FIXED.** IT-137..142, IT-144, IT-145, and TC-814 were passing but unbacked because their identifiers did not attach to Rust symbols. Direct test comments now bind every row; TC-814 binds to the Rust audit that executes `check_thin_boundary.sh`. | tests/cli_assurance.rs; tests/assurance_cross_language.rs; tests/cli_assurance_contract.rs; tests/audit_static.rs |
| FND-002 | low | **FIXED with this artifact.** The matrix and live-plan rows still showed pending after the required tests had recorded passing runs. Only the issue-#74 rollups and rows are flipped to complete; G13 stays pending for external review. | spec/tests.md; plan/plan.md |
| FND-003 | low | **OUT OF SCOPE, unchanged.** The coverage report still lists historical/known unbacked rows outside issue #74. This change neither flips nor claims those rows; the report has no `status_lies`. | Existing repository baseline outside #74 |

## Acceptance reconciliation

| Contract | Evidence | Result |
|----------|----------|--------|
| Complete upstream v1 facts and schema | IT-136, IT-137; non-empty records, all observation states, upstream type/schema | PASS |
| Deterministic compact/pretty bytes | IT-138 plus newline mutation | PASS |
| Exact fail-closed premises and load failures | IT-139, IT-140 plus accepted-superset mutation | PASS |
| Empty vs unknown vs unavailable | IT-141 | PASS |
| stderr/stdout separation | IT-142 | PASS |
| Thin upstream boundary | TC-814 / `tc090_src_is_a_thin_boundary_over_quire_rs` | PASS |
| No network or child execution | IT-143, success and premise-refused strace paths | PASS |
| Cross-language consumption | IT-144; Rust schema validation, Python/jsonschema and Node exact-byte probes | PASS |
| Public compatibility contract | IT-145; help, README, changelog, capability, manifest and lock pin | PASS |

## Recorded execution evidence

- `cargo test --locked --test cli_assurance` → `7 passed; 0 failed`.
- `cargo test --locked --test assurance_cross_language` → `1 passed; 0 failed`;
  Python/jsonschema and Node both ran on this host.
- `cargo test --locked --test cli_assurance_contract` → `1 passed; 0 failed`.
- Full Linux audit suite → `9 passed; 0 failed`, including IT-143 with strace.
- Static audit suite → `4 passed; 0 failed`; direct thin-boundary script →
  `thin-boundary audit ok`.
- Exact `make ci` gate → 214 tests, 0 failures, all policy and specification
  traceability audits green.
- Locked release build → exit 0.
- Targeted specification validation → 5/5 grammar-clean, exit 0.
- Branch coverage export after the binding fix → no IT-136..145/TC-814 row
  in `unbacked_rows`; `status_lies: []`.

## Closing gate

G13 completed through the independent SR-012/SR-013 review and the SR-014/SR-015
remediation pass. Every GitHub finding was read and dispositioned, exact-head
local gates were rerun, and the bounded administrative-merge exception can now
be recorded. Hosted CI remains manual-only and was not dispatched.
