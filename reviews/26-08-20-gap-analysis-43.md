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

**PASS** — no matrix row claims evidence it lacks, no test carries a tag that
points at nothing, and every acceptance criterion that a test can reach is
backed. FND-001 was raised as a corpus-wide instrument gap and initially
deferred; that was wrong — the ticket's goal is the instrument telling the
truth, and stopping at "no status lies" would have shipped a matrix whose
requirement coverage still read 2/119. It is fixed in this branch.

## Findings

| ID      | Severity | Summary                                                              | Refs                       |
| ------- | -------- | -------------------------------------------------------------------- | -------------------------- |
| FND-001 | high     | **FIXED IN BRANCH.** Acceptance-criterion coverage read 2/119 because AC ids written inside a leading parenthetical are never bound — only the first id after `//` is captured. All 120 tags converted to the comma form the pattern admits; criteria now 108/119 | spec/functional/, tests/   |
| FND-002 | medium   | Six FR-016 rows (IT-083, IT-084, TC-085..087, TC-093) had passing tests carrying no tracking tag; they escaped the status-lie check because their `⚠️` status marker is not in the module's declared vocabulary | tests/cli_update.rs, src/self_update/mod.rs |
| FND-003 | medium   | `⚠️` is used as a status value in spec/tests.md but is declared nowhere in `traceability.status` (complete/pending/failed/retired). Any row carrying it is silently exempt from the lie check | spec/tests.md:73            |
| FND-004 | low      | TC-093 (`self_update` is package-agnostic) had no test; added as a source-inspection test, the sanctioned last resort for a property no runtime path can observe | tests/audit_static.rs:82   |
| FND-005 | low      | Id collisions predating this branch: IT-060/IT-061 and IT-095/IT-096 each name two different behaviours | tests/cli_schema.rs:19, tests/output_contract.rs:95 |

## FND-001 in detail — and why the canonical marker is not the answer

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

**Is this a parser bug, a tag-format bug, or comments not following the
format?** The third — but with a trap that explains why the whole corpus writes
the broken shape.

The module declares a *canonical* marker for Rust,
`#[trace("IT-033", "FR-011-AC-1")]`, which binds every id it names. It was
probed: the coverage engine reads it correctly, and `rustc` rejects it —
`error: cannot find attribute 'trace' in this scope`. No proc-macro crate
provides the attribute. Python's `@pytest.mark.trace(...)` and TypeScript's
`trace(...)` are real syntax in their languages; the Rust canonical marker is a
paper form. So the **only multi-id form a Rust repo can actually compile** is
the legacy comment list, and the corpus writes a parenthetical instead.

Not a parser bug: `rust-comment-id` does exactly what it documents, and its
delimiter guard exists for a measured reason — an unanchored form bound
`# FR-003-CON-1 sweep found in real matrices`, prose that verified nothing.

All 120 tags in this repo were converted to the comma form, in three passes:
106 mechanically (`// ID (a, b): prose` → `// ID, a, b: prose`), the slash form
`// IT-076 / FR-015-AC-1:` by hand, and the mixed ones where a parenthetical
carried non-id text. Five ids became **dead tags** on conversion and were
demoted back into prose: `FR-047-AC-8`, `FR-048-AC-6`, `FR-048-AC-10`, `FR-050`
and `TC-752` are quire-rs criteria that quire-cli's matrix does not declare, so
binding them would have created the opposite defect. A sixth, `US-004-AC-3`, was
a genuine gap — a real integration test with no matrix row at all — and got one
(IT-099).

Two follow-ons for the ecosystem, recorded here because this repo cannot fix
them: quire-rs writes 344 comments in the same broken shape, so its own
criterion coverage is understated by the same mechanism; and the engine's
rewrite suggestion for Rust points at `#[trace({ids})]`, which does not compile.

## Coverage

| Measure | Before (#43) | After |
| --- | --- | --- |
| All rows backed | 41/234 | **215/240** |
| Acceptance criteria backed | 2/119 | **108/119** |
| Matrix rows (`spec/tests.md`) backed | — | 105/113 |
| Status lies | 53 | **0** |
| Untracked symbols (dead tags) | 4 | **0** |

Every one of the 11 remaining unbacked criteria is unbackable by construction,
not by omission:

| Criteria | Why |
| --- | --- |
| FR-001-AC-1..6, NFR-001-AC-1..3 | the `render` subcommand and its latency budget, retired under spec.md §2bis |
| FR-003-AC-3 | sugar-field harvesting, retired this branch — no engine ever had it |
| FR-006-AC-4 | its text names the retired FR-001; left unbacked rather than claimed through a CR note the AC never received |
| NFR-002-AC-2 | `Demonstration` — a fresh-machine `cargo install`, which no test in this repo can perform |
| NFR-006-AC-1 | a CHANGELOG process rule; any test for it would assert its own fixture |
| FR-016-AC-3, FR-016-AC-7 | the npm install and registry-unreachable paths, documented in the matrix as having no automated trace (network + global-install side effects) |

The eight unbacked matrix rows are the retired render traces (IT-001, IT-009,
IT-010, IT-016, IT-017, IT-018, TC-088) and the `FR-001 render subcommand` row.

Measured with the repo's own release build under an isolated
`CARGO_TARGET_DIR`. The `quire` on PATH is 0.23.0 and must not be used for this.

Semantic review (intent↔test↔code) was run over the changed surface, not the
whole repo. It produced the six over-claim corrections recorded in
`reviews/26-08-20-coverage-trace-binding.md` and the FND-002 discovery above.
No plan bundle governs this work — `plan/plan.md` predates it — so the
plan-completion step does not apply.
