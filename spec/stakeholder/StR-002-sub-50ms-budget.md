---
id: StR-002
title: "Sub-50 ms end-to-end budget per artifact"
artifact_type: StR
---

## Stakeholder Need

Per-artifact generation is on the agent's critical path. A spec-authoring session may render 20-100 artifacts in a single workflow. At 200 ms per artifact (Python/Node CLI baseline), that is 4-20 seconds of pure CLI overhead — visible latency that agents experience as "slow tool use" and human authors experience as a stuttering shell.

The CLI MUST hit a p95 end-to-end budget of **50 ms** (cold start → load module → validate → render → write) on a modern dev workstation, matching the headline target of the parent plan and the upstream `filament-core` NFR-006.

## Priority

Must-Have

## Acceptance

- **StR-002-AC-1**: `hyperfine --warmup 3 'quire render FR --module $ISO --data ctx.json'` reports p95 ≤ 50 ms against the `spec-artifacts-iso` FR archetype.
- **StR-002-AC-2**: Same hyperfine harness runs in CI on each push and gates merges.
- **StR-002-AC-3**: Release-profile binary uses LTO thin + codegen-units=1 (matches `quire-rs`).
