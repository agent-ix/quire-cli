---
id: SR-012
title: "Code review — PR #75 deterministic assurance export"
type: SpecReview
analysis: code-review
scope: "src/commands/assurance.rs, tests/cli_assurance.rs, tests/assurance_cross_language.rs, tests/audit_no_network.rs, tests/cli_assurance_contract.rs, scripts/check_thin_boundary.sh, spec/functional/FR-020, spec/usecase/US-006, spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/spec/functional/FR-020
    type: reviews
  - target: ix://agent-ix/quire-cli/spec/stakeholder/StR-004
    type: references
---
# SR-012: Code review — PR #75 deterministic assurance export

## Summary

Independent review of `issue/74-assurance-export` at `c5f8c43f`, covering the
`quire assurance` command, its ten integration tests, the thin-boundary audit,
and the FR-020/US-006/spec-tests traceability. Gates were run, not assumed:
`env CARGO_TARGET_DIR=… make ci` exits 0 with 213 tests passing across 30 test
binaries. The command itself is well built — the refusal ordering is correct by
construction and the thin boundary holds. The findings are in the traceability
layer beneath the matrix rows, and in what no gate re-runs.

## Verdict

**FAIL** — three `high` findings, all in traceability and gating. None is a
defect in the command's runtime behavior.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-022 | high | `US-006`'s acceptance criteria are prose bullets under `## Acceptance`, not an `ID`-column table under `## Acceptance Criteria`, so `quire coverage` mints no `US-006-AC-*` target at all. `spec/tests.md:54` nonetheless marks `US-006 \| AC-1..4 \| … \| ✅`, and the `US-006-AC-1/2/3` tags on `it_136`, `it_138` and `it_139` appear in `unmatched_tags`. | spec/usecase/US-006-export-assurance-facts.md:19 |
| FND-023 | high | `quire coverage` reports `spec/functional/FR-020-assurance-export-subcommand.md: 5/9 (55%)`. `FR-020-AC-4`, `-AC-5`, `-AC-8` and `-AC-9` are minted with `backed: false`, while `spec/tests.md:79` marks the FR-020 row `AC-1..9 … ✅`. The IT rows citing them (IT-141/142/144/145) are themselves backed, so the break is between row and criterion. | spec/tests.md:79 |
| FND-024 | high | No gate runs `quire validate` or `quire coverage`. `make ci` is `fmt-check lint test deny deny-bans audit-unsafe audit-thin-boundary audit-tool-drift`, and the Makefile has no spec or coverage target. Both traceability claims in the PR description were produced by hand and nothing re-runs them; FND-022 and FND-023 are what an ungated layer accumulates. | Makefile:124 |
| FND-025 | medium | `it_144` returns green when `python3` or `node` is absent (`ErrorKind::NotFound` → `eprintln!` then pass), and `it_143` does the same when `strace` is absent. FR-020-AC-8 requires the golden to be consumed by both probes and FR-020-AC-7 requires an observed no-socket/no-child run; on a host missing those runtimes cargo still prints `ok` with no marker. Both probes did execute in this run — zero `skipping` lines — so the evidence is real here; nothing distinguishes that from the no-op case. | tests/assurance_cross_language.rs:40 |
| FND-026 | medium | `--pretty` emits `serde_json::to_string_pretty(&export)` rather than a re-indent of the validated `to_json_bytes()` bytes, so the emitted pretty document is produced by a CLI-owned serialization path. FR-020 step 4 says compact is "the exact `AssuranceExport::to_json_bytes()` value" and `--pretty` "changes whitespace only"; that equality currently rests on IT-138's `Value` comparison over one golden fixture rather than on construction. Re-indenting `compact` would make it structural and remove the CLI-owned path (StR-004-AC-2). | src/commands/assurance.rs:172 |
| FND-027 | low | `ExpectedSchema::from_str` splits the identity on the first `/`, so `mod/ule/FR@<digest>` parses as module `mod` with archetype `ule/FR`. It is rejected today only because `accepted_premises` compares the module name against `--expect-module`. | src/commands/assurance.rs:60 |
| FND-028 | low | When the module declares no `traceability:` model the command substitutes `SymbolGraph::default()` and emits an export with no bindings rather than refusing. FR-020 does not state which is correct; given FR-020-CON-4's fail-closed stance the intended behavior should be explicit. | src/commands/assurance.rs:142 |
| FND-029 | low | The PR description reports "197 tests"; this run reports 213 passing across 30 binaries, presumably measured before `c5f8c43`. Worth refreshing so the merge record matches the branch. | reviews/SR-010-assurance-export-code-review.md:1 |

## Gates

| Gate | Result |
|---|---|
| `make ci` (fmt-check, lint, test, deny, deny-bans, audit-unsafe, audit-thin-boundary, audit-tool-drift) | exit 0 |
| Test total | 213 passed, 0 failed, 0 ignored, across 30 binaries |
| Skipped probes in this run | 0 (`strace`, `node`, `python3`, `jsonschema` 3.2.0 all present) |
| `quire validate` over the 8 changed spec docs | 8/8 grammar-clean |
| `quire validate` repo-wide | 39/47 — the 19 findings are all in documents this PR does not touch |
| `quire coverage` | 286/337 backed; 0 status lies; 0 untracked symbols; 23 unbacked rows (pre-existing); FR-020 5/9 |

## What is sound

The refusal ordering is correct by construction: build → `to_json_bytes` →
`read_assurance_export` → exact-premise comparison → one `write_all`. Every
error path precedes the single write, so "non-zero exit with empty stdout"
holds structurally rather than by test.

Revision and repository validation live upstream in quire-rs
(`AssuranceError::InvalidRevision` and `EmptyRepository`, `src/assurance.rs:362`)
instead of being duplicated in the CLI — StR-004 respected — and
`tests/cli_assurance.rs:315` proves the refusal with `--revision main`.

`scripts/check_thin_boundary.sh` requires all five delegation surfaces plus
`to_json_bytes`, and forbids a CLI-owned `AssuranceExport`, `ASSURANCE_V1_SCHEMA`,
`jsonschema::`, `parse_document`, and `Command::new` inside the command.
IT-139's premise-drift table covers wrong name, wrong version, missing digest,
extra premise and malformed syntax; IT-138 pins the golden against
`to_json_bytes()` output directly.
