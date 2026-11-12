---
id: SR-20261112234811
title: "PLAT-1506 quire-cli PR179 spec-review"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-cli PR179 head de40ab3; spec/functional/FR-003, FR-011, FR-017 (new ACs only); src/commands/extract.rs, lookup.rs, coverage.rs; quire-rs coverage/binding_diagnostics.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1506. Spec-review of the three new criteria. Criterion strength and EARS form were checked by hand because Jev was not used. Each criterion was compared with its test body and with the code path that produces the behaviour.

Mutation evidence: a mutant whose `heading_matches` compared trimmed headings without `normalize_heading` failed `lookup_by_heading_uses_number_normalization` at tests/cli_lookup.rs:75. The other two criteria can also fail. A `harvest_edges` without dedup gives 2 `implements` edges. An engine that does not emit `no-symbol-bound` when `bound == 0` fails the reasons assertion.

Placement is right. FR-017 owns the coverage `--json` payload, and FR-006 is the stream contract. The ids do not appear on origin/main except as fix proposals in SR-20261112184204. They continue each table's numbering. They contain no TC/IT ids and no real standard names. `quire validate` passes on all three files.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-017-AC-24 claims more than its test asserts. The general clause says there is "one record per language the binder examined, each with `candidates` and `bound` counts". The test only checks that `binding_census` is non-empty and holds a `rust` record. Extra records, missing languages, or non-rust records without the counts all pass. Fix: assert the language set (exactly `["rust"]` for this rust-only scope) and that every record has numeric counts. Or narrow the clause to the rust record. | spec/functional/FR-017-coverage-subcommand.md:171, tests/cli_stream_contract.rs:299-308 |
| FND-002 | medium | FR-011-AC-7 says `--heading` matches on the normalized heading, but the test covers only the `--level` branch. That branch uses the CLI's own `normalize_heading`. Without `--level`, the CLI calls `quire_rs::section`, a separate normalizer. No test sends a numbered heading through that path. The CLI also matches case-insensitively (`eq_ignore_ascii_case`), which the criterion leaves out. Fix: add a no-`--level` assertion on `## 1. Behavior` and state the case rule. Or narrow the criterion to `--heading ... --level`. | spec/functional/FR-011-lookup-subcommand.md:67, src/commands/lookup.rs:74-108, tests/cli_lookup.rs:63-77 |
| FND-003 | low | FR-003-AC-6 names the dedup key `(source, type, target)`, but the test cannot detect a key that drops `type`. Its two edge groups differ in both type and target. A probe shows the engine keeps `implements`, `depends_on` and `references` edges to the same StR-001 as three edges. A target-only dedup would still pass the test. Fix: add a second relationship of another type to the same target, and assert one edge per type. | spec/functional/FR-003-extract-subcommand.md:91, tests/cli_extract.rs:183-226 |

## Verdict

All three criteria can fail and are tagged on the right tests. FR-017-AC-24 and FR-011-AC-7 state more than their tests check. Narrow the wording or extend the tests (FND-001, FND-002). FR-003-AC-6 is sound apart from the untested `type` component of its key.
