---
id: NFR-003
title: "Zero unsafe Rust in this crate"
artifact_type: NFR
quality_attribute: security
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-003"
    type: "consumes"
    cardinality: "1:1"
---

## Constraint

`quire-cli`'s own source code SHALL contain zero `unsafe` blocks. The crate inherits `quire-rs`'s NFR-003 stance and adds no new unsafe surface.

Transitive `unsafe` from dependencies (`clap`, `serde`, `serde_json`, `quire-rs`'s deps) is permitted; it is the upstream crates' responsibility to justify their own usage.

## Acceptance

- **NFR-003-AC-1**: `scripts/check_unsafe_comments.sh` (inherited from rust-lib-cookiecutter) reports zero `unsafe` blocks in `src/` and `tests/`.
- **NFR-003-AC-2**: CI fails the build if any `unsafe` block appears without a `// SAFETY:` comment (script defaults).
