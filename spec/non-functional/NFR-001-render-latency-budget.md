---
id: NFR-001
title: "End-to-end render latency budget p95 ≤ 50 ms"
type: NFR
quality_attribute: performance_efficiency
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-001"
    type: "consumes"
    cardinality: "1:1"
---

> **RETIRED (render removal — 2026-06-04):** There is no render path to bench
> (mirrors quire-rs NFR-001 retirement, commit 500a3d3). The per-artifact latency
> need is retained at the stakeholder level for the surviving subcommands
> (`validate`/`parse`/`extract`/`lookup`/`edit`); see the revised StR-002. This
> render-latency NFR is kept for history only; ACs dropped from the
> required-coverage tally (ids retained, immutable). Recorded in `spec.md` §2bis.

## Statement

End-to-end wall-time for `quire render <archetype> --module <path> --data <json>`
SHALL satisfy **p50 ≤ 20 ms, p95 ≤ 50 ms, p99 ≤ 100 ms**, measured cold-start, on
a modern dev workstation (≥ 8 cores, NVMe, ≥ 16 GB RAM, Linux x86_64), against the
`spec-artifacts-iso` `FR` archetype with a representative context object (~2 KB
JSON). This requirement is RETIRED with the render path (2026-06-04) and kept for
history only.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Render p50 latency (cold-start) | 20 ms | 20 ms | hyperfine benchmark |
| Render p95 latency (cold-start) | 50 ms | 50 ms | hyperfine benchmark |
| Render p99 latency (cold-start) | 100 ms | 100 ms | hyperfine benchmark |

## Verification

A `hyperfine --warmup 3` benchmark over `quire render FR --module $ISO --data
ctx.json` measured the percentile latencies against the thresholds above. The
benchmark is retired with the render path; verification is no longer performed.

## Rationale

Per the parent plan, the **process boundary** is the cost driver — not template render speed. `quire-rs` is fast in-process (sub-millisecond render); the budget exists to bound argv parsing, file I/O, Registry load, and process startup. The 50 ms p95 figure matches the parent plan's design target and the upstream `filament-core` generation-performance NFR.

## Acceptance Criteria

The Measurement and Evaluation table above is the acceptance-criteria equivalent.
The following compliance checks are RETIRED (render removal — 2026-06-04); ids are
retained and immutable, dropped from the required-coverage tally:

- NFR-001-AC-1 (RETIRED): `make bench` runs `hyperfine --warmup 3 'target/release/quire render FR --module $ISO --data ctx.json'` and reports p95 ≤ 50 ms.
- NFR-001-AC-2 (RETIRED): CI runs the same hyperfine harness on each push; merge is blocked if p95 regresses by > 20 % from the prior release.
- NFR-001-AC-3 (RETIRED): Cargo release profile uses `lto = "thin"`, `codegen-units = 1`, matching `quire-rs/Cargo.toml`.
