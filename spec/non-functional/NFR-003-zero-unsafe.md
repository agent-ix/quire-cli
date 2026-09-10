---
id: NFR-003
title: "Zero unsafe Rust in this crate"
type: NFR
quality_attribute: security
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-003"
    type: "depends_on"
    cardinality: "1:1"
---

## Statement

`quire-cli`'s own source code SHALL contain zero `unsafe` blocks. The crate
inherits `quire-rs`'s NFR-003 stance and adds no new unsafe surface. Transitive
`unsafe` from dependencies (`clap`, `serde`, `serde_json`, `quire-rs`'s deps) is
permitted; it is the upstream crates' responsibility to justify their own usage.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Undocumented `unsafe` blocks in first-party Rust source and tests | 0 | 0 | `quire-qualify unsafe-comments` AST-backed static audit |

## Verification

The repository-local Rust qualification package runs locally and parses
first-party Rust source and tests, asserting that every `unsafe` block has a
nearby `// SAFETY:` comment or an exact reviewed baseline locus. It rejects
stale baseline entries as well as new undocumented blocks.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-003-AC-1 | The native AST-backed gate reports zero undocumented `unsafe` blocks in first-party Rust source and tests | Inspection (TC-092, TC-837) |
| NFR-003-AC-2 | Local qualification fails if an `unsafe` block lacks a nearby `// SAFETY:` comment and its exact locus is not in the reviewed baseline | Mutation test (TC-092, TC-837) |
