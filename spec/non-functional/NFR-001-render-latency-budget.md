---
id: NFR-001
title: "End-to-end render latency budget p95 ≤ 50 ms"
artifact_type: NFR
quality_attribute: performance
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-001"
    type: "consumes"
    cardinality: "1:1"
---

## Constraint

End-to-end wall-time for `quire render <archetype> --module <path> --data <json>` SHALL satisfy:

- **p50 ≤ 20 ms**
- **p95 ≤ 50 ms**
- **p99 ≤ 100 ms**

measured cold-start, on a modern dev workstation (≥ 8 cores, NVMe, ≥ 16 GB RAM, Linux x86_64), against the `spec-artifacts-iso` `FR` archetype with a representative context object (~2 KB JSON).

## Rationale

Per the parent plan, the **process boundary** is the cost driver — not template render speed. `quire-rs` is fast in-process (sub-millisecond render); the budget exists to bound argv parsing, file I/O, Registry load, and process startup. The 50 ms p95 figure matches the parent plan's design target and the upstream `filament-core` generation-performance NFR.

## Acceptance

- **NFR-001-AC-1**: `make bench` runs `hyperfine --warmup 3 'target/release/quire render FR --module $ISO --data ctx.json'` and reports p95 ≤ 50 ms.
- **NFR-001-AC-2**: CI runs the same hyperfine harness on each push; merge is blocked if p95 regresses by > 20 % from the prior release.
- **NFR-001-AC-3**: Cargo release profile uses `lto = "thin"`, `codegen-units = 1`, matching `quire-rs/Cargo.toml`.
