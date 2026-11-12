---
id: SR-20261112034125
title: "PLAT-1419 quire-cli PR172 spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR172 head e5fb56f; spec/functional/FR-029-publish-check-subcommand.md, spec/functional/index.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-1419. This review covers the new FR-029 (`quire publish-check`), its index entry and three IT rows (IT-258..260). It also checks FR-029 against the companion quire-rs PR #657 (FR-083). Failure-domain and EARS lenses were applied inline over FR-029 only. The names agree across the two PRs: `check_publishable`, `RestrictedClauseSet` with the fields `authority`, `id`, `version`, `path` and `text_clauses`, and the engine ordering. Exit 2 matches `ix_cli_kit` `Outcome::Refused` and FR-007-AC-2. Fixtures are synthetic `example.invalid` data, and no real standard text or roadmap appears. `quire validate` exits 0 with only the pre-existing module `unused-import` warning. The review found two medium and two low defects. FR-029-AC-3's `path` field inherits the quire-rs PR #657 review's finding that `LoadedModule` holds no clause-set path. Whatever that finding settles must also be applied to FR-029.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-029-AC-7 cannot be verified as written. IT-260 runs the binary, which calls the engine's free function `check_publishable` directly, so no stub can replace the rights rule. The literal `text` token also already appears in src (src/commands/properties.rs:270), so a src-wide scan fails on unrelated code. | spec/functional/FR-029-publish-check-subcommand.md:46, spec/tests.md:319 |
| FND-002 | medium | FR-029-AC-5 says "a path escaping the scope", but publish-check has no `--scope`. FR-005 defines `--module` escape as a `..` segment or a symlink resolving outside the module root. The AC also names no exit code for that refusal, and FR-005-AC-1 (exit 1) conflicts with FR-007-AC-2 (exit 2). | spec/functional/FR-029-publish-check-subcommand.md:44 |
| FND-003 | low | The "opens no network socket" clause of FR-029-AC-6 gives no method and duplicates NFR-004-AC-2, which already checks every subcommand with strace. | spec/functional/FR-029-publish-check-subcommand.md:45 |
| FND-004 | low | Stream contract gaps. FR-029-AC-1 does not say which stream carries the "no restricted set" statement. FR-029-AC-2 and AC-3 do not say what stderr carries under `--format json`. | spec/functional/FR-029-publish-check-subcommand.md:40-42 |

## Recommended fixes

- FND-001: Split AC-7 into two checks. First, a unit test feeds the CLI's verdict-to-outcome and render step a hand-built `Vec<RestrictedClauseSet>`: an empty vector gives 0 and a non-empty one gives 2. Second, a source scan scoped to the publish-check module finds no `citation_only`, `clearance_ref` or `rights` token. Map AC-7 to a new Unit TC row instead of IT-260.
- FND-002: Name the cases. A `--module` path with a `..` segment, or a `clause_sets` symlink that resolves outside the module root, exits 2. A missing path or a strict-load failure exits 3.
- FND-003: Replace the socket clause with "covered by NFR-004-AC-2", or state the strace method.
- FND-004: Specify stdout for the clean verdict, and the stderr content in json mode.

## Verdict

Changes requested (two medium findings). FR-029-AC-1..4 and AC-6 (the byte-identity part) can be tested and can fail, the thin-boundary intent is correct, and the IT rows are new, unique and map every AC.
