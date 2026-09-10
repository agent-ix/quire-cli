---
id: NFR-007
title: "Exact qualified Rust toolchain"
type: NFR
quality_attribute: compatibility
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "constrains"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs"
    type: "depends_on"
    cardinality: "1:1"
---

# NFR-007: Exact qualified Rust toolchain

## Statement

The `quire-cli` repository SHALL declare Rust 1.98.1 as its single supported and
qualified stable compiler in the package manifest, repository toolchain,
Clippy policy, and every manual CI or release toolchain selection.

## Scope

- Applies to the `quire-cli` crate, its pinned `quire-rs` engine revision, and
  repository-owned build, test, audit, and release commands.
- Does not dispatch hosted CI or publish a crate, binary, or npm package.
- A newer compiler requires a new qualification change; a floating `stable`
  selector is not an acceptable substitute for the qualified version.

## Rationale

A split or floating compiler policy makes local evidence non-reproducible and
can qualify the CLI against a compiler different from the one used to release
it. Existing older declarations are migration inputs, not justification for an
older supported version.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Stable compiler declarations differing from 1.98.1 | 0 | 0 | Repository static audit |
| Required local qualification gates failing on Rust 1.98.1 | 0 | 0 | Locked local qualification run |
| Pinned `quire-rs` revisions not themselves qualified on Rust 1.98.1 | 0 | 0 | Dependency provenance inspection |

## Verification

A fail-closed repository audit enumerates the manifest, toolchain, Clippy
policy, and every `.yml` or `.yaml` workflow toolchain selection and refuses
any value other than 1.98.1. The qualification record SHALL retain the exact
dependency revision and results of these local commands, run without hosted CI:

- `cargo +1.98.1 fmt --all -- --check`;
- `cargo +1.98.1 clippy --locked --all-targets --all-features -- -D warnings`;
- `cargo +1.98.1 test --locked --all-targets --all-features`;
- `RUSTDOCFLAGS="-D warnings" cargo +1.98.1 doc --locked --all-features --no-deps`;
- `cargo +1.98.1 build --locked --release` and the `audit_ldd` integration test;
- `cargo deny --locked check`, `cargo audit`, and all repository static audits.

Formatting and lint changes are repaired. Only a reproduced incompatibility in
a required tool may be presented for an owner-approved exception that names its
owner, bounded scope, removal condition, and recheck date.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-007-AC-1 | `Cargo.toml`, `rust-toolchain.toml`, `clippy.toml`, and every manual CI or release Rust selection name exactly 1.98.1, with no floating stable selector. | Test (TC-142) |
| NFR-007-AC-2 | The pinned `quire-rs` revision contains the exact-Rust-1.98.1 qualification merged by `agent-ix/quire-rs#422`, and the qualification record names that revision. | Inspection (TC-143) |
| NFR-007-AC-3 | The enumerated locked formatting, all-target/all-feature Clippy and tests, strict documentation, release/static-binary, dependency-policy, and static-audit gates pass locally on Rust 1.98.1. | Test and inspection (TC-143) |
| NFR-007-AC-4 | Reverting any governed compiler declaration causes the static conformance audit to fail. | Mutation test (TC-142) |

## Dependencies

- **Upstream**: `quire-rs` exact-Rust-1.98.1 qualification, merged as
  `agent-ix/quire-rs#422`.
- **Downstream**: LR05 launcher and npm package-tooling remediation in
  `agent-ix/quire-research#61`.
