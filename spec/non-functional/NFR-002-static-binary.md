---
id: NFR-002
title: "Single statically-linked binary"
artifact_type: NFR
quality_attribute: deployability
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Constraint

The shipped `quire` binary SHALL be a single statically-linked executable:

1. No dynamic linkage to any project-supplied shared library.
2. `quire-rs` is consumed as a Rust dependency, not as a dynamic library.
3. The binary's only runtime dependencies are `libc` and the platform dynamic loader.

## Rationale

Static linking makes installation a single file copy, eliminates "missing .so" failure modes, and allows `cargo install quire-cli` to produce a runnable binary on any supported platform without extra setup.

## Acceptance

- **NFR-002-AC-1**: On Linux x86_64, `ldd target/release/quire` lists at most: `linux-vdso.so.1`, `libgcc_s.so.1` (if used), `libc.so.6`, `libpthread.so.0`, `libdl.so.2`, `libm.so.6`, `/lib64/ld-linux-x86-64.so.2`. No `libquire_rs.so`, no `libssl`, no other project-supplied `.so`.
- **NFR-002-AC-2**: `cargo install --git https://github.com/agent-ix/quire-cli quire-cli` produces a working binary on a fresh Linux machine with only `rustc` installed.
