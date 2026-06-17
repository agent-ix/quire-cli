---
id: StR-002
title: "Sub-50 ms end-to-end budget per artifact"
type: StR
---

> **RETIRED (render removal — 2026-06-04):** This StR framed the sub-50 ms budget
> around the render hot path (`cold start → load module → validate → render → write`).
> With render removed (mirrors quire-rs render retirement, commit 500a3d3), the
> render-centric budget no longer applies. The surviving fast-CLI need for the
> retained subcommands (`validate`/`parse`/`extract`/`lookup`/`edit`) is carried by
> the revised [StR-001](./StR-001-static-binary-hot-path.md); the dedicated render-latency [NFR-001](../non-functional/NFR-001-render-latency-budget.md) is also retired. Kept for
> history only; ACs dropped from the required-coverage tally (ids retained,
> immutable). Recorded in `spec.md` §2bis.

## Stakeholder Need

Per-artifact generation is on the agent's critical path. A spec-authoring session may render 20-100 artifacts in a single workflow. At 200 ms per artifact (Python/Node CLI baseline), that is 4-20 seconds of pure CLI overhead — visible latency that agents experience as "slow tool use" and human authors experience as a stuttering shell.

The CLI MUST hit a p95 end-to-end budget of **50 ms** (cold start → load module → validate → render → write) on a modern dev workstation, matching the headline target of the parent plan and the upstream `filament-core` [NFR-006](../non-functional/NFR-006-cli-stability.md).

## Rationale

A spec-authoring session may render 20-100 artifacts in a single workflow. At the
~200 ms Python/Node CLI baseline that is seconds of pure CLI overhead per session —
latency agents experience as slow tool use and humans experience as a stuttering
shell. A sub-50 ms p95 budget keeps the per-artifact boundary cost negligible.
This need framed the budget around the render hot path; with render removed
(2026-06-04) the render-centric budget is RETIRED and the surviving fast-CLI need
is carried by the revised [StR-001](./StR-001-static-binary-hot-path.md).

## Validation Criteria

This need was considered satisfied when a `hyperfine` benchmark of the render hot
path reported p95 ≤ 50 ms and that benchmark gated CI. With the render path
removed, these criteria are RETIRED (ids retained, immutable, dropped from the
coverage tally):

- StR-002-AC-1 (RETIRED): `hyperfine --warmup 3 'quire render FR --module $ISO --data ctx.json'` reports p95 ≤ 50 ms against the `spec-artifacts-iso` FR archetype.
- StR-002-AC-2 (RETIRED): Same hyperfine harness runs in CI on each push and gates merges.
- StR-002-AC-3 (RETIRED): Release-profile binary uses LTO thin + codegen-units=1 (matches `quire-rs`).

## Priority

Must-Have
