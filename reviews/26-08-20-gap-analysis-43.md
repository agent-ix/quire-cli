---
id: SR-002
title: "Gap analysis — quire-cli matrix trace binding (#43)"
type: SpecReview
analysis: gap-analysis
scope: "spec/tests.md, spec/functional/, spec/non-functional/, src/, tests/"
review_set: subset
---

## Summary

Post-implementation verification of `agent-ix/quire-cli#43` on branch
`fix/43-status-lies`. The deliverable was that every Test Matrix row marked
complete is backed by a real tracking tag on a test that verifies it. Matrix
rows now read 96/105 backed with **zero status lies** and **zero untracked
symbols**; every one of the nine unbacked rows is a retired (⛔) render-era
trace. Two gaps beyond the ticket's stated scope were found and closed here (six
FR-016 rows whose tests existed untagged, and the TC-093 static row), and one
ecosystem-level finding is recorded but deliberately not acted on.

## Verdict

**CONDITIONAL** — no incomplete tasks and no matrix row claiming evidence it
lacks. One high finding is recorded (FND-001) as a measured instrument gap that
is corpus-wide rather than introduced by this branch; acting on it is an
ecosystem decision, not a quire-cli one.

## Findings

| ID      | Severity | Summary                                                              | Refs                       |
| ------- | -------- | -------------------------------------------------------------------- | -------------------------- |
| FND-001 | high     | Acceptance-criterion coverage reads 0 for 22 of 24 requirement documents because AC ids written inside a leading parenthetical are never bound — only the first id after `//` is captured | spec/functional/, tests/   |
| FND-002 | medium   | Six FR-016 rows (IT-083, IT-084, TC-085..087, TC-093) had passing tests carrying no tracking tag; they escaped the status-lie check because their `⚠️` status marker is not in the module's declared vocabulary | tests/cli_update.rs, src/self_update/mod.rs |
| FND-003 | medium   | `⚠️` is used as a status value in spec/tests.md but is declared nowhere in `traceability.status` (complete/pending/failed/retired). Any row carrying it is silently exempt from the lie check | spec/tests.md:73            |
| FND-004 | low      | TC-093 (`self_update` is package-agnostic) had no test; added as a source-inspection test, the sanctioned last resort for a property no runtime path can observe | tests/audit_static.rs:82   |
| FND-005 | low      | Id collisions predating this branch: IT-060/IT-061 and IT-095/IT-096 each name two different behaviours | tests/cli_schema.rs:19, tests/output_contract.rs:95 |

## FND-001 in detail

The module's `rust-comment-id` form is
`(?m)//\s*(<ID>(?:\s*,\s*<ID>)*)\s*(?:[:,/()\[\]]|[-–—]\s|$)`. Capture group 1
starts immediately after `//`, so in the corpus's dominant convention —

```rust
// IT-033 (FR-011-AC-1, US-005-AC-2): `lookup --heading --level 1` returns …
```

— only `IT-033` binds. `FR-011-AC-1` and `US-005-AC-2` sit inside the
parenthetical, which no `//` precedes, and are read as prose.

Measured, not inferred. Rewriting that one comment to the comma form the pattern
does admit —

```rust
// IT-033, FR-011-AC-1, US-005-AC-2: `lookup --heading --level 1` returns …
```

— moved `spec/functional/FR-011-lookup-subcommand.md` from **0/6** to **1/6**
backed criteria and the rollup from 98 to 99. The probe was reverted.

This is not a defect this branch introduced, and it is not confined to
quire-cli: the module manifest itself documents quire-rs writing 344 comments in
the `// TC-480 / FR-025-AC-1: …` shape, which binds `TC-480` alone by the same
rule. Converting quire-cli's ~45 tags unilaterally would make this repo read
differently from every sibling, so the choice is between changing the corpus
convention everywhere or widening the pattern — and the pattern's delimiter
guard exists for a documented reason (an unanchored form bound
`# FR-003-CON-1 sweep found in real matrices`, prose that verified nothing).
Recorded for an ecosystem decision rather than settled here.

## Coverage

| Measure | Before (#43) | After |
| --- | --- | --- |
| Matrix + criterion rows backed | 41/234 | 98/238 |
| Matrix rows (`spec/tests.md`) backed | — | 96/105 |
| Status lies | 53 | **0** |
| Untracked symbols (dead tags) | 4 | **0** |
| Unbacked matrix rows | — | 9, all retired (⛔) |

Measured with the repo's own release build under an isolated
`CARGO_TARGET_DIR`. The `quire` on PATH is 0.23.0 and must not be used for this.

Semantic review (intent↔test↔code) was run over the changed surface, not the
whole repo. It produced the six over-claim corrections recorded in
`reviews/26-08-20-coverage-trace-binding.md` and the FND-002 discovery above.
No plan bundle governs this work — `plan/plan.md` predates it — so the
plan-completion step does not apply.
