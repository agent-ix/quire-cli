---
id: SR-003
title: "Code review — quire-cli trace id collisions (#45)"
type: SpecReview
analysis: code-review
scope: "spec/functional/, spec/tests.md, tests/"
review_set: subset
---

## Summary

Review of `95853b0` on `fix/45-trace-id-collisions`: four colliding trace ids
given their own matrix rows (IT-101..104) and four backfilled acceptance
criteria (FR-004-AC-19/20, FR-018-AC-8/9). No new test logic — existing tests
were retagged and renamed. Two acceptance criteria claimed more than their
tests assert; both are fixed in the branch. The gates were re-run rather than
assumed.

## Verdict

**CONDITIONAL** — two `medium` findings, both fixed in-branch before this
review was written. Nothing `high`. The retagging itself is verified by
falsification, not by a rising count.

## Findings

| ID      | Severity | Summary                                                                 | Refs                                  |
| ------- | -------- | ----------------------------------------------------------------------- | ------------------------------------- |
| FND-001 | medium   | **FIXED IN BRANCH.** FR-018-AC-9 claimed `obligation: null` on *every* criterion record; the test asserted `criteria[0]` only, and the fixture carries two criteria with differing `Verification` values. Test strengthened to `.all()` plus a non-vacuity length assertion | tests/output_contract.rs:170          |
| FND-002 | medium   | **FIXED IN BRANCH.** FR-004-AC-19 and the IT-101 row claimed the document validates against "modules **discovered** from" the scope. The fixture root holds `manifest.yaml`, so the run takes the exact-module branch — a different path from the search-root discovery AC-13/IT-081 cover. Reworded in the AC, the row, and the test comment | spec/functional/FR-004-validate-subcommand.md:169 |
| FND-003 | low      | `it_102`'s stderr assertion is `contains("line")` — the substring could originate anywhere in the message. The behaviour is genuinely line-numbered (verified: `broken-fr.md: line 16: [FR] …`), so FR-004-AC-20 is accurate; the assertion is weaker than the criterion it backs | tests/cli_validate.rs:325             |
| FND-004 | low      | `cli_errors::unknown_archetype_exits_1_with_named_error` is a strict subset of `cli_validate::it_050_unknown_archetype_reports_unknown`, which asserts the same behaviour plus empty stdout. Leaving it untagged is correct — tagging it IT-050 would recreate the collision this branch removes — but it is redundant coverage kept only as an errors-lane smoke check | tests/cli_errors.rs:95                |

## Priority 1 — do the four ACs match what their tests assert?

Checked line by line against the test bodies. Two passed, two did not.

**FR-018-AC-8 — accurate.** Every clause is backed: the schema is read through
`cargo metadata` from the resolved `quire-rs` manifest path
(`schemas/output/properties-v1.schema.json`), not vendored; the payload is
validated against it; and the non-vacuity clause is real — the test asserts
`criteria.len() == 2` and that some record carries
`obligation.method == "Test"`, so conformance is not asserted over an empty
payload.

**FR-004-AC-20 — accurate.** Relative glob, exit 1, empty stdout, stderr naming
the offending file. "Line-numbered" was verified against live output rather
than assumed (FND-003 records that the *assertion* is weaker than the fact).

**FR-018-AC-9 — over-claimed (FND-001).** The criterion said "every criterion
record carries `obligation: null`". The test read:

```rust
assert!(payload["documents"][0]["criteria"][0]["obligation"].is_null(), …);
```

One record. The fixture document declares two criteria with different
`Verification` values (`Test (TC-001)` and `Demonstration`), so a regression
that attached an obligation to the second would have passed. Fixed by asserting
over every record, with a length check so `.all()` cannot pass vacuously on an
empty array.

**FR-004-AC-19 — over-claimed (FND-002).** The criterion said "validates
against modules **discovered** from it". `validate_module()` resolves to
`tests/fixtures/validate-mod`, which contains `manifest.yaml` — and FR-004's
own Behavior section states the branch explicitly: *"If `--scope` itself
contains `manifest.yaml`, it is loaded as one exact module; otherwise Quire
loads module search roots from the scope…"*. The test therefore exercises the
exact-module branch, while discovery is what AC-13 specifies and IT-081
verifies. Reworded in all three places so the two branches stay distinguishable.

## Priority 2 — is the untagged duplicate the right call?

Yes. `IT-050` is already bound by `cli_validate.rs:82`
(`it_050_unknown_archetype_reports_unknown`), which asserts the same exit code
and `UnknownArchetype` on stderr **and** requires stdout to be empty. Tagging
the `cli_errors` test IT-050 would put two symbols on one row — reintroducing
exactly the defect `#45` exists to remove, in the same commit that removes it.
Left untagged, with the reasoning in a comment above the test so a later reader
does not "fix" the missing tag. Recorded as FND-004 because the duplication
itself remains.

## Priority 3 — do the IT-101/102 descriptions match path vs glob?

Yes, and this was the second thing the old names got wrong.
`it_060_scope_glob_validates_matching_documents` passed `docs/valid-fr.md` — a
plain path, no glob metacharacter. The row and the new name
(`it_101_scoped_relative_path_validates_without_module`) both say *path*, and
IT-102 is the only glob case. The test comment now names both distinctions
(path-not-glob, exact-module-not-discovery) so neither is re-lost.

## Verification of the retag itself

A higher `backed` count proves nothing about collisions — it rises whenever
rows are added. The property `#45` asks for is that a row fails when its
binder goes, so that was tested directly.

| Probe | Result |
| --- | --- |
| before fix: retag `output_contract.rs` IT-095 → undeclared id | `backed` 215 → **215** (unchanged), `dead` 0 → 1 — row stayed green on its twin |
| before fix: rename `fn it_060_…` → `fn it_902_…` | no change at all — `rust-test-name-id` is `\bfn (?i:tc)(\d+)_`, TC only, so it never bound |
| after fix: retag `cli_properties.rs` IT-095 → undeclared id | `backed` 223 → **222**, `dead` 0 → 1 — row now fails with its single binder gone |

Rollup: backed 215/240 → **223/248**, status lies **0**, dead tags **0**.

## Gates

Re-run, not assumed:

- `make ci` — fmt-check, clippy `-D warnings`, tests, `cargo deny` licenses +
  bans, unsafe audit, thin-boundary audit: green
- `cargo test --test cli_validate --test output_contract --test cli_errors` —
  31 passed, 0 failed; all five renamed tests execute under their new names
- `quire validate --scope . "spec/**/*.md"` — exit 0, zero errors. 19 grammar
  warnings, all pre-existing in files this branch does not touch (FR-009,
  FR-010, NFR-001/004/005/006); the three edited spec files produce none
- measured with the repo's own release build under a repo-local
  `CARGO_TARGET_DIR`; the `quire` on PATH is 0.23.0 and stale

## Gap analysis

The reverse gap is what this branch is mostly about: both retagged behaviours
were **unspecified**, and the criteria were backfilled rather than pointing the
tests at whichever existing criterion was nearest. Scoped glob resolution was
described in FR-004's synopsis (`<DOC.md|GLOB|->`) and Behavior text but had no
AC; the `properties --json` envelope's conformance to the published schema had
no AC in this repo at all, because quire-rs publishes the schema and never
constructs an envelope.

No new unowned behaviour was introduced. One pre-existing gap stays open and is
noted rather than fixed here: the coverage report exposes no per-target list of
which symbols back a row, which is why a collision can only be found by
removing a tag and watching a number move. That is the root reason `#45` had to
be discovered by hand.
