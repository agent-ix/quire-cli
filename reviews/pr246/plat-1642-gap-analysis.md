---
id: SR-967175026302197768900474181941935084792
title: "Gap analysis — quire-cli anomaly-set source ownership"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli; spec/functional/FR-047-quire-selftest.md, src/commands/selftest.rs, tests/cli_selftest.rs, Cargo.toml, Cargo.lock"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-cli", type: references }
---

## Summary

Audited the local self-test record criterion, its computed trace matrix, and the changed source and tests. The initial review found the source-set contract lacked a local criterion and the test used an untracked Quoin trace id. The follow-up adds FR-047-AC-11, binds the integration test to it, and closes that gap.

## Verdict

**PASS** — the new local criterion and trace binder resolve FND-001; the focused integration test passes.

## Scope

- FR-047-AC-4: "Given a quire self-test run, `quire selftest --run` emits a run record conforming to Quoin FR-230; `--out <file>` writes those exact retained JSON bytes to the named file and emits no run-result JSON to stdout, while omitting `--out` writes those bytes to stdout only when a repository evidence store is available; without a store, the command rejects before starting any case." (examined)
- FR-047-AC-7: "Inspection of the embedded case resources finds quire-cli cases sourced from this repository and quire-rs cases sourced from the quire-rs repository, with no cases copied between repositories or from qa-corpus." (examined)
- FR-047-AC-8: "Given run results from quire-cli and quire-rs, the `quire` self-test output contains only its own case/run evidence and does not generate or duplicate the combined tool validation report or its engineering-assurance and filament-analyze report behavior." (examined)
- FR-217-AC-8 source-set rule from PLAT-1642 (context only): "The quire-cli binary shows quire-cli and quire-rs records and no quoin records."
- FR-047-AC-11 (examined on disposition pass): "Given a selftest run, its anomaly set contains each authored anomaly document embedded by quire-cli and linked quire-rs with exact raw text and stable paths prefixed by the owning source repository; it contains no Quoin records, and its `producerVersion` exactly equals the run record's `toolVersion`."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test tags Quoin FR-217-AC-8, which is untracked in quire-cli's matrix; local FR-047-AC-4 only requires general FR-230 conformance and does not specify anomaly source composition, provenance, or Quoin exclusion. Add a local FR-047 criterion for this contract and tag the test to it, retaining the Quoin criterion only as context. | tests/cli_selftest.rs:98; spec/functional/FR-047-quire-selftest.md:107 |

## Coverage

- Reconciliation: quire matrix (quire 0.52.0) — no run evidence read.
- Plan completion: not assessed
- Criteria across the repository: tagged 438 · untagged 52 · tagged-by-ignored-test 15 · method-without-symbol 41.
- Initial relevant criterion: FR-047-AC-4 was tagged with five binders, including the run-record integration test.
- Initial matrix evidence: the new test carried untracked `FR-217-AC-8`. Disposition matrix: FR-047-AC-11 is `tagged` to `tests/cli_selftest.rs::retained_anomaly_set_joins_only_the_two_owning_repositories`; no untracked symbols remain.
- Reverse gap: the initial source-set omission is addressed by FR-047-AC-11; no new stub or hollow test found.
- Focused disposition check: `cargo test -p quire-cli --test cli_selftest retained_anomaly_set_joins_only_the_two_owning_repositories -- --exact` — PASS.
- Semantic review: skipped (not requested).


## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-047-AC-11 now specifies the anomaly source composition, raw text, stable source-qualified paths, Quoin exclusion, and producer version. The integration test is tagged to this local criterion, its existing assertions cover each clause, and the computed matrix reports one binder with no untracked symbols. |
