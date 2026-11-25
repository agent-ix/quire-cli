---
id: SR-967175026302197768900474181941935084793
title: "Code review — quire-cli strict selftest trace markers"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli; src/commands/matrix.rs, src/commands/selftest.rs, tests/cli_matrix.rs, tests/cli_selftest.rs, selftest/cases/selftest-list-cases, spec/functional/FR-047-quire-selftest.md"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/FR-047"
    type: references
---

## Summary

Reviewed quire-cli PR #247 at 2e61933d49f08c821fa3783da0fcacd6d70687c7 against origin/main. The focused test and formatting checks pass, but the new marker parser rejects valid forms of the declared shell trace grammar, and the fixture test assumes a marker entry path absent from the active module.

## Verdict

**FAIL** — strict mode can reject valid shell markers, and the test's synthetic module configuration hides that actual shell markers are not bound by the active repository module.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Strict marker inventory recognizes only the literal # trace: prefix, although the declared shell trace grammar also accepts #trace: and other whitespace forms. | src/commands/matrix.rs:313-315 |
| FND-002 | high | The fixture adds a selftest entry path absent from the active module, so actual markers do not bind into the computed matrix. | tests/cli_matrix.rs:83-90 |

## Coverage

- Examined FR-047-AC-3, including its strict selftest marker and binding contract.
- Examined run, check_selftest_trace_markers, and collect_run_scripts in src/commands/matrix.rs; the embedded case inventory and trace test in src/commands/selftest.rs.
- Examined selftest_shell_marker_binds_and_removal_names_the_case_script, its synthetic marker configuration, and the changed selftest integration assertions.
- Examined the shell marker grammar in active spec-artifacts-process@v0.36.0: pattern #\\s*trace:\\s*(.+)$ and only entry path scripts/audits/**/*.sh. It does not currently include selftest paths. The new test supplies selftest/cases/**/*.sh itself, so it does not verify current module configuration.
- Focused locked-build test passed (1/1); cargo fmt --all -- --check passed; strict validation of the edited FR passed. No aggregate or long-running check was run.

