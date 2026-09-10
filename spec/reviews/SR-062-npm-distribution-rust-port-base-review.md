---
id: SR-062
title: "Base review of the npm distribution Rust port specification"
type: SpecReview
analysis: base
scope: "US-007; FR-022; FR-023; FR-024; NFR-008; ADR-0002; TM-001 rows IT-159 through IT-164 and TC-815 through TC-821"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-007"
    type: reviews
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-022"
    type: reviews
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-023"
    type: reviews
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-024"
    type: reviews
  - target: "ix://agent-ix/quire-cli/spec/non-functional/NFR-008"
    type: reviews
---

## Summary

The owner-selected base review examined the complete #61 npm launcher/package
scope for correctness, completeness, consistency, testability, traceability,
and boundary discipline. The specification is **PASS after remediation and
owner disposition**: all drafting findings are closed in the reviewed text,
every behavior and adverse path has a planned Rust-owned trace, and no Quire
language or extraction semantic entered scope. The owner approved retaining the
cross-platform npm distribution package with the exact bounded Node host in
ADR-0002 on 2026-09-09.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | medium | **Closed.** “Byte-for-byte arguments” was not meaningful for Node host strings. FR-022 now requires an identical ordered sequence of string values and separately requires inherited byte streams. | FR-022-AC-2; IT-159 | wrong-requirement |
| FND-002 | medium | **Closed.** The first draft promised byte identity after any cross-file write failure, which cannot be guaranteed atomically across Cargo and npm manifests. FR-024 now makes complete validation before the first replacement the fail-closed boundary and does not disguise an operating-system failure as transactional storage. | FR-024-AC-3; TC-818 | wrong-requirement |
| FND-003 | medium | **Closed.** The required executable inventory originally appeared only as scattered scope prose and did not disposition release-host shell or the deferred WASM surface. NFR-008 now classifies every discovered executable path, routes generic archive/checksum remediation to #63, and preserves external host orchestration without treating it as product semantics. | NFR-008 Executable-Path Inventory and Disposition | missing-requirement |
| FND-004 | medium | **Closed by owner disposition.** npm cannot make one `bin` entry select four optional native packages without a host executable. The owner approved retaining the npm distribution package with the exact bounded Node file in ADR-0002; the direct platform-package alternative is rejected. | ADR-0002; FR-022 Dependency; NFR-008 Dependency | correct-requirement-no-evidence |
| FND-005 | low | **Closed.** The issue requires license review, but the first draft covered emitted license files without the Rust dependency or Node-import surfaces. NFR-008-AC-6 and TC-821 now cover all three. | NFR-008-AC-6; IT-163; TC-821 | missing-requirement |
| FND-006 | low | **Closed during implementation preflight.** “Exact Node/npm” incorrectly implied that local qualification must equal the separately pinned release-host versions. The ADR and NFR now require exact release pins plus recorded, compatible local host versions; Rust remains exact 1.98.1 in both. | ADR-0002; NFR-008 Verification; TC-821 | wrong-requirement |

## Base Checklist Result

| Check | Result | Evidence |
|---|---|---|
| Stakeholder intent | PASS | US-007 preserves the existing one-package npm installation and native process contract. |
| Scope | PASS | Only `quire-cli` npm distribution is active; WASM, Filament, extraction, language profiles, and generic release archives are explicitly excluded or routed. |
| Correctness and consistency | PASS after FND-001/FND-002 | One Rust catalog governs all target/package/version projections; launcher and generator rules agree. |
| Completeness | PASS | Happy, missing, wrong, unsupported, signal, stream, mode, version, license, and clean-install paths are specified; ADR-0002 carries the owner disposition. |
| Testability | PASS | IT-159..164 and TC-815..821 name observable inputs, refusals, and oracles. Test assertions are Rust and new Rust tests must carry `ix-trace-rs`. |
| Traceability | PASS | Every FR/NFR criterion maps to at least one pending Test Matrix row; status remains pending until a recorded local run. |
| Language-redesign containment | PASS | The tool treats the executable as opaque and is forbidden from depending on `quire-rs` or encoding grammar, profile, temporal, protocol, extraction, or source semantics. |

## Owner Decision

On 2026-09-09 the owner approved retaining the single cross-platform
`@agent-ix/quire-cli` package with the exact minimal Node host boundary stated
in ADR-0002. No finding remains open and implementation may proceed.
