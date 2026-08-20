---
id: SR-001
title: "Code review — quire-cli matrix trace binding (#43)"
type: SpecReview
analysis: code-review
scope: "tests/, spec/tests.md, spec/functional/FR-003, spec/functional/FR-004, spec/usecase/US-004"
review_set: subset
---

## Summary

Reviewed the `fix/43-status-lies` change, which binds every quire-cli Test
Matrix trace to the test that verifies it: 18 test files retagged, one new test
file, four new tests, and four spec files edited. The review's own priority was
tautology and over-claim detection — a trace tag that names an acceptance
criterion the test does not actually assert marks that criterion backed while
proving nothing, which is the same defect class the change exists to remove.
Six such over-claims were found in the change under review and fixed before this
artifact was written; the measured instrument reading went 41/234 backed with 53
status lies and 4 dead tags, to 92/238 backed with **zero** of either.

## Verdict

**CONDITIONAL** — one high and eight medium findings, all fixed in the branch
before merge; two low findings (id collisions, a misnamed test) tracked in
agent-ix/quire-cli#45 rather than fixed here.

The review was run twice. The first pass covered the tagging commit; the second
was re-run before merge because the largest commit — converting 120 tags from
the parenthetical form to the comma form — landed *after* the first pass. That
conversion promoted roughly 200 previously-decorative acceptance-criterion
citations into binding evidentiary claims, none of which had ever had to be
true. The second pass diffed every bound tag against its matrix row's declared
traces and found seven disagreements (FND-009, FND-011). Merging on the strength
of the first review would have shipped one false claim.

## Findings

| ID      | Severity | Summary                                                              | Refs                       |
| ------- | -------- | -------------------------------------------------------------------- | -------------------------- |
| FND-001 | medium   | IT-026 exit-1 test tagged FR-007-AC-2/AC-3 (path-safety, unknown archetype) but asserts neither — it is AC-4, structural-validation failure | tests/cli_errors.rs:49     |
| FND-002 | medium   | IT-026 exit-2 test tagged FR-007-AC-4 (structural failure) though it exercises an argv error — AC-5 plus FR-014-AC-7 | tests/cli_errors.rs:63     |
| FND-003 | medium   | FR-007-AC-2 and FR-007-AC-3 had no test carrying them once FND-001 was corrected; bound to the tests that do verify them | tests/cli_sandbox.rs:14, tests/cli_validate.rs:84 |
| FND-004 | medium   | IT-024 tagged FR-006-AC-1 (failure → empty stdout, non-empty stderr) but asserts only a success run; AC-1 moved to IT-031, which walks five failure classes | tests/cli_io.rs:36, tests/cli_io.rs:100 |
| FND-005 | medium   | IT-011 tagged FR-006-AC-4, whose text names the retired FR-001 render subcommand; tag dropped rather than claimed through a CR note the AC text never received | tests/cli_io.rs:14         |
| FND-006 | medium   | Three criteria were backed by assertions weaker than the criterion: FR-002-AC-1 grepped the payload instead of reading `.frontmatter.id`; FR-002-AC-2 never compared `parse -` against `parse <file>`; FR-002-AC-4 asserted "is an object" rather than empty `sections[]` | tests/cli_parse.rs:20, tests/cli_io.rs:14, tests/cli_parse.rs:38 |
| FND-009 | high     | The tag conversion turned ~200 decorative AC citations into binding claims; a systematic diff of every bound tag against its matrix row's declared traces found IT-098 binding FR-018-AC-6 (the thin-boundary *Inspection* criterion, traced by TC-090) when it verifies AC-7 | tests/cli_properties.rs:188 |
| FND-010 | medium   | TC-092 asserted only that the unsafe-comment gate exits 0 today, which a script that unconditionally returned 0 also satisfies; NFR-003-AC-2 is about the gate *failing* on a violation | tests/audit_static.rs:68 |
| FND-011 | medium   | Six matrix rows did not declare acceptance criteria their tests demonstrably verify (IT-005, IT-031, IT-050, TC-090, TC-091, TC-092) — the tags were right and the rows were stale | spec/tests.md |
| FND-007 | low      | Id collisions predating this change: IT-060/IT-061 name both the `schema` tests and the validate scope-glob tests; IT-095/IT-096 name both the `properties` tests and the output-contract envelope tests | tests/cli_schema.rs:19, tests/cli_validate.rs:304, tests/output_contract.rs:95 |
| FND-008 | low      | `cli_errors::it_013_unknown_archetype_exits_1` is misnamed — matrix IT-013 is "empty document parses"; the unknown-archetype behaviour is IT-050's | tests/cli_errors.rs:78     |

## Tautology check

Every new or strengthened assertion was checked against the question the skill
asks: *what change to the source makes this fail?*

| Test | Fails when |
| --- | --- |
| IT-015 edge dedup | `harvest_edges` collects into a `Vec` instead of a `BTreeSet` — the test counts occurrences of a twice-declared relationship and a twice-linked body target, so "an edge exists" is not enough to pass |
| IT-031 diagnostic classes | any of five failure classes emits a bare `anyhow` line, drops `kind`/`severity`, or writes a partial document to stdout |
| IT-043 `--content` | `--content` is treated as literal text (the path leaks into the document), or an absent path is inserted verbatim rather than named in a diagnostic |
| TC-090 thin boundary | a parse/render/validate primitive is called outside `src/main.rs` or `src/commands/*.rs` |
| TC-091 HTTP bans | a ban is dropped from `deny.toml`, or an HTTP client crate enters `Cargo.lock` — asserted in-process, so it does not depend on cargo-deny being installed |
| TC-092 unsafe comments | an `unsafe` block appears without a `// SAFETY:` comment and outside the reviewed baseline |

## Spec-code faithfulness

- **FR-004-AC-15..18** (`--summary`, `--severity`) were checked against
  `src/commands/validate.rs:74-89` and `apply_severity_overrides`. The flags are
  implemented and unchanged by this branch; the requirement was the gap. Each
  new AC is traced to an existing passing test (TC-714/720/721/755).
- **FR-003-AC-3** required `extract` to harvest frontmatter sugar fields
  (`dependencies:`). `quire_rs::harvest_edges` reads the frontmatter
  `relationships:` array and body `ix://` links only, and `dependencies` appears
  nowhere in the quire-rs sources. The criterion described behaviour no engine
  ever had; retired with a CR note rather than satisfied with a fabricated test.
  `US-004`'s prose carried the same claim and was corrected.

## Gates

Run with an isolated `CARGO_TARGET_DIR` — the machine's shared
`/home/peter/.cargo-target` can serve stale artifacts across worktrees.

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | pass |
| `cargo clippy -D warnings` | pass |
| `cargo test` | pass — 130 tests, 23 suites, 0 failed |
| `cargo deny check licenses` | pass |
| `cargo deny check bans` | pass |
| `scripts/check_unsafe_comments.sh` | pass |
| `scripts/check_thin_boundary.sh` | pass |
| `quire coverage --scope .` | 92/238 backed, 0 status lies, 0 dead tags |
