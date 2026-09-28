---
id: SR-069
title: "Code and Rust review of PR #104 quire matrix subcommand"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli@c9d022a31d8ba2779a6b2cd17777a5aed5fb60c8; src/commands/matrix.rs, src/commands/coverage.rs, src/commands/mod.rs, src/main.rs, Cargo.toml, Cargo.lock, tests/cli_matrix.rs, tests/snapshots/matrix-worked-example.md, tests/snapshots/help.txt, README.md, CHANGELOG.md"
review_set: subset
---

## Summary

Ticket: PLAT-1078. PR agent-ix/quire-cli#104, diff origin/main...HEAD (base
5648889). Code review with the rust-review lane folded in. Reviewed sha
c9d022a.

Checked:

- **`coverage` is unchanged by the refactor.** The engine-bump commit f7ff45a
  and HEAD c9d022a were built from the same quire-rs 0.48.0. Twelve `coverage`
  invocations gave byte-identical stdout, stderr and exit codes on both builds:
  human, `--json`, `--pretty --json`, `--format tsv`, `--strict`, a valid
  `--severity`, an unknown `--severity`, no `spec/`, `--diagnostics-format
  json`, and the quire-cli repo itself in human, JSON and TSV (365 KB of
  JSON). `coverage --help` is identical apart from the binary name. The call
  order is preserved: validate the scope, load the registry, reject unknown
  pack checks, apply the severity overrides, then compute from the overridden
  registry.
- **The pin.** `=0.48.0` at rev a4f3a7065a33d67f5ce70a9ae6a55c9ca6f69688. The
  upstream tag v0.48.0 resolves to that commit (`git ls-remote`). The pin has
  the IT-145 shape (by `rev`, exact `=` version), the lock agrees under
  `--locked`, and no prose restates the SHA.
- **The golden file.** `tests/snapshots/matrix-worked-example.md` is
  `cmp`-identical to FR-026 lines 118-122.
- **Truncate, then escape.** A statement of exactly 80 scalar values stays
  whole. One of 81 is cut to 77 plus `...`. A pipe at scalar 77 survives the
  cut and is escaped after it; this is the case that tells the two orderings
  apart. A pipe at scalar 78 is dropped by the cut under either ordering, so
  that case cannot discriminate between them.
- **Status tokens.** An exhaustive `match` with no catch-all is bound to the
  engine's `#[serde(rename_all = "kebab-case")]` spelling by a unit test. A new
  upstream variant fails to compile, and a renamed one fails the test.
- **The JSON payload.** It is a typed `Serialize` struct that mirrors the
  engine's `skip_serializing_if = "Vec::is_empty"`.
- **No panic surface in the production path.**
- **The shared computation.** `Target` and `compute_report()` are shared with
  `coverage` and have `pub(super)` visibility.

Mutation testing covered 14 mutants of `src/commands/matrix.rs`. Twelve were
killed: the escape/truncate order swap, empty passes `--strict`,
`method-without-symbol` counted as a gap, `tagged-by-ignored-test` passing,
the budget off by one, a 78-scalar keep, both binder separators, the status
token spelling, JSON emitting a present-but-empty array, the `(ignored)`
marker, and the `(none)` marker. Two survived; they are FND-001 and FND-002.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Pipe-escaping of the Criterion and Binders markdown cells is untested. Mutant `markdown_cell` → `tsv_cell(text)` (no escape) passes every test. A binder path containing `\|` is reachable: a trace-tagged test at `tests/a\|b.rs` renders `tests/a\|b.rs:3:1`. Without the escape that row splits into five columns, and no test would fail. | src/commands/matrix.rs:176-178, tests/cli_matrix.rs:368-460 |
| FND-002 | low | The AC-6 scrub of tabs, newlines and CRs in the markdown Statement cell is untested. Mutant `truncate_statement(&tsv_cell(statement))` → `truncate_statement(statement)` passes every test. The case is reachable: an FR criterion cell holding a literal tab reaches the engine verbatim (`"statement":"Tab\there."`), and dropping the scrub would emit a raw tab and count it toward the 80-character budget, with no test failing. | src/commands/matrix.rs:182-184, tests/cli_matrix.rs:464-521 |

## Verdict

Approve. The implementation matches FR-026 AC-1..14 as amended. The
`coverage` refactor preserves behaviour byte for byte. The pin, the golden
file and the status-token binding are correct. Both findings are low
test-oracle gaps, and the behaviour they guard is correct today. They do not
block merge.

## Dispositions

Reviewed at agent-ix/quire-cli@60338c8085092b4357772e8bf3be252ecfe27165 (round 1). The fix diff touches only tests/cli_matrix.rs (+55 lines, two new tests) and adds the SR copies under reviews/, which are byte-identical to the reviewer's files. The re-run mutants are both killed now. `tests/cli_matrix.rs` passes 15/15.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 60338c8. `it169_a_pipe_in_a_binder_path_is_escaped` pins the row `\| FR-001-AC-1 \| Bound from a piped path. \| tests/a\\\|b.rs:3:1 \| tagged \|` and a four-cell split. The no-escape mutant now fails that test. |
| FND-002 | fixed | 60338c8. `it170_structural_characters_become_spaces_before_truncation` feeds 40 a's, a tab and 40 b's (81 scalar values). It asserts the engine keeps the tab and that markdown renders the cell as 40 a's, a space, 36 b's and `...`, with no tab in the output. The no-scrub mutant now fails that test. |
