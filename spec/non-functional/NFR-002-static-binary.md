---
id: NFR-002
title: "Single statically-linked binary"
type: NFR
quality_attribute: portability
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Statement

The shipped `quire` binary SHALL be a single statically-linked executable: no
dynamic linkage to any project-supplied shared library, `quire-rs` consumed as a
Rust dependency (not a dynamic library), and the binary's only runtime
dependencies being `libc` and the platform dynamic loader.

## Rationale

Static linking makes installation a single file copy, eliminates "missing .so" failure modes, and allows `cargo install quire-cli` to produce a runnable binary on any supported platform without extra setup.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Project-supplied shared libraries linked | 0 | 0 | `ldd` static audit |
| `cargo install` on a fresh Linux host (only `rustc`) yields a runnable binary | yes | yes | Install demonstration |

## Verification

On Linux x86_64, an `ldd` audit of `target/release/quire` confirms only `libc`,
the dynamic loader, and permitted system libraries are linked (no
`libquire_rs.so`, no `libssl`, no other project-supplied `.so`), and a clean-host
`cargo install` produces a working binary.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-002-AC-1 | On Linux x86_64, `ldd target/release/quire` lists at most: `linux-vdso.so.1`, `libgcc_s.so.1` (if used), `libc.so.6`, `libpthread.so.0`, `libdl.so.2`, `libm.so.6`, `/lib64/ld-linux-x86-64.so.2`. No `libquire_rs.so`, no `libssl`, no other project-supplied `.so` | Inspection |
| NFR-002-AC-2 | `cargo install --git https://github.com/agent-ix/quire-cli quire-cli` produces a working binary on a fresh Linux machine with only `rustc` installed | Demonstration |
