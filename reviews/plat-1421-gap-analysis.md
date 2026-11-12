---
id: SR-20261112184202
title: "PLAT-1421 quire-cli PR174 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR174 head b59c201; spec/functional/FR-029-publish-check-subcommand.md, spec/tests.md IT-258..261, src/commands/publish_check.rs, tests/cli_publish_check.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1421. Plan completion: not assessed.

`quire matrix --scope .` reports all eight FR-029 criteria (AC-1..AC-8) as `tagged`. AC-7 is bound to the unit test in `src/commands/publish_check.rs`, and the rest are bound to `tests/cli_publish_check.rs`. No stub or untraced production code was found in the diff. Test-strength defects in the AC-5 and AC-6 tests are recorded in the companion code-review SR (SR-20261112184201, FND-002 and FND-003) and are not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The IT-260 row still says an outside-root `clause_sets` symlink exits 2. Under the lead ruling it exits 3 as a strict-load failure. When FR-029-AC-5 is amended in this PR, amend the IT-260 row to match. | spec/tests.md:319 |

## Examined scope

```yaml
scope:
- {id: FR-029-AC-1, path: spec/functional/FR-029-publish-check-subcommand.md, role: examined, excerpt: "For a synthetic module whose clause sets are all publishable, the command exits 0, writes the line `no restricted clause sets` to stdout, and writes nothing to stderr."}
- {id: FR-029-AC-2, path: spec/functional/FR-029-publish-check-subcommand.md, role: examined, excerpt: "For a synthetic `example.invalid` module with restricted sets, the command exits 2 (`Outcome::Refused`) and stderr names each restricted set as `authority/id@version`, one per line, in the engine's order."}
- {id: FR-029-AC-3, path: spec/functional/FR-029-publish-check-subcommand.md, role: examined, excerpt: "`--format json` writes to stdout an object with a `restricted` array (empty when clean) whose entries carry `authority`, `id`, `version` and `text_clauses`, plus tool provenance."}
- {id: FR-029-AC-4, path: spec/functional/FR-029-publish-check-subcommand.md, role: examined, excerpt: "Neither stdout nor stderr contains clause text from the fixture in any format; a test searches for the fixture's text sentinel and fails if found."}
- {id: FR-029-AC-5, path: spec/functional/FR-029-publish-check-subcommand.md, role: examined, excerpt: "`..` in `--module`, or a `clause_sets` symlink that resolves outside the module root, exits 2 (FR-005 path refusal). A missing module path, or a module whose clause set fails strict load, exits 3."}
- {id: FR-029-AC-6, path: spec/functional/FR-029-publish-check-subcommand.md, role: examined, excerpt: "The module directory is byte-identical after any run, and the command opens no network socket, as verified for every subcommand by NFR-004-AC-2."}
- {id: FR-029-AC-7, path: spec/functional/FR-029-publish-check-subcommand.md, role: examined, excerpt: "The exit code is a function of the engine verdict only: given a hand-built `Vec<RestrictedClauseSet>`, an empty vector maps to exit 0 and a non-empty vector maps to exit 2."}
- {id: FR-029-AC-8, path: spec/functional/FR-029-publish-check-subcommand.md, role: examined, excerpt: "The publish-check module source contains none of the tokens `citation_only`, `clearance_ref`, `rights` or `licensed`, so no rights policy lives in the CLI (FR-021-AC-7)."}
```

## Verdict

Changes requested only for FND-001, a spec-text consistency item. Every criterion has a tagged test.
