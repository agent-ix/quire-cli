---
id: REV-001
title: "Spec Review — quire-cli (first pass)"
type: Review
---

# Spec Review — quire-cli (first pass)

Date: 2026-05-28
Reviewer: Claude Opus 4.7 (initial author + reviewer)
Status: **APPROVED with minor findings** — proceed to `/spec-to-plan`.

## Scope

Reviewed `spec/spec.md`, 4 StRs, 4 USs, 8 FRs, 6 NFRs, `tests.md` (32 IT/BENCH/AUDIT entries).

## Checklist Outcomes

| Gate | Outcome |
|------|---------|
| ID format + uniqueness | ✅ All identifiers 3-digit, no duplicates, sequential |
| US "As/I want/So that" + ≥2 AC | ✅ All four USs comply |
| FR has Behavior + AC + traces to US + cites upstream quire-rs FR where applicable | ✅ FR-001..004 cite the wrapped quire-rs FR; FR-005..008 are CLI-originated and trace to a StR |
| NFR has measurable acceptance | ✅ All NFRs have either numeric (p95, exit code, ldd shape) or static-analysis (cargo deny, check_unsafe_comments) gates |
| Test coverage: every AC has ≥1 IT/BENCH/AUDIT | ✅ Verified per `tests.md` traceability table |
| Error path rule | ✅ FR-007 catalogs every exit code; tests.md IT-026 covers each |
| Sandbox boundary rule | ✅ FR-005 + StR-003; ITs 5/6/7/22/23 cover `..`, symlink escape, `--data` escape, `--out` escape, stdin bypass |
| Cross-ref + relationship frontmatter | ✅ FR-001..004 declare `consumes` edges to upstream `quire-rs` FRs by ID |

## Findings

### F-1 (Informational) — `IT-XXX` instead of `TC-XXX`

`spec.md` § 6 declares `IT-XXX` (integration test) as the test-case identifier in place of the template default `TC-XXX`. Intentional: every CLI test is process-level. `quire-rs` uses `TC-XXX`. No action required, but cross-spec consumers should be aware.

### F-2 (Minor) — `quire-rs` parity-fixture sourcing

FR-001-AC-6 and IT-018 require a render parity sweep against the 8 ISO archetypes. The fixture set lives in `quire-rs/tests/render_parity/`. The plan should decide between:

- **(a)** vendor a copy of the fixtures into `quire-cli/tests/fixtures/` (independent, no submodule),
- **(b)** depend on `spec-artifacts-iso` directly as a test-fixture path,
- **(c)** symlink to `quire-rs/tests/render_parity/` at build time.

Recommendation: **(a)** — copy into `tests/fixtures/iso/` with a `make refresh-fixtures` target that re-pulls from upstream. Keeps the repo self-contained, matches the rust-lib-cookiecutter test pattern.

### F-3 (Minor) — Platform sensitivity of IT-008 (strace)

`IT-008` uses `strace -e network` to verify zero socket calls — Linux-only. Mac/Windows CI lanes would need `dtruss`/`Procmon` equivalents or to skip the test. Plan should:

- mark the IT as `linux-only`,
- add `AUDIT-003 (cargo deny bans)` as the cross-platform substitute for the no-network guarantee.

### F-4 (Minor) — `--help` snapshot location

NFR-006-AC-2 pins `quire --help` via snapshot test. Plan should specify the snapshot lives at `tests/snapshots/help.txt` (or use `insta`-style inline snapshots).

### F-5 (Design Decision Open) — Does `extract` auto-validate?

`extract` parses the document and runs the body-extraction DSL but does **not** schema-validate the document's frontmatter. The spec is silent. Recommended behavior: extract does NOT validate (use `validate` for that, fail fast); ITs assume this. If we want validation-on-extract, add a `--validate` flag in a follow-up MINOR release.

Captured here so the plan author records the decision rather than inventing one.

### F-6 (Style) — Long-form prose in `Behavior` sections

FR-001 and FR-005 lean prose-heavy. Acceptable for first-pass authoring; if quality gates later prefer bulleted normative clauses, refactor — not blocking.

## Coverage Summary

- **Stakeholder requirements**: 4/4 trace to at least one US + one FR + one test.
- **User stories**: 4/4 each have ≥ 2 ACs that trace to at least one IT.
- **Functional requirements**: 8/8 each have ≥ 4 ACs that trace to at least one IT.
- **Non-functional requirements**: 6/6 each have a measurable acceptance gate.

No orphan IDs. No AC without a trace. No FR without a US. No US without a StR.

## Next Step

Proceed to `/spec-to-plan` to generate `plan/plan.md` + per-task files under `plan/tasks/`.
