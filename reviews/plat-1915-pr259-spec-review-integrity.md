---
id: SR-967175026302197768900474181941935084894
title: "spec-review/integrity of quire-cli PR 259"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli; PLAT-1915; PR 259; src/commands/validate.rs; tests/cli_validate.rs; spec/functional/FR-004-validate-subcommand.md; FR-004-AC-25"
review_set: subset
---

## Summary

Reviewed PR #259 at 8eabb2118480e5a5357714d70fe8af440423b958. The semantic module-load error is reported before the misleading document-type failure, with precise local criterion, test and implementation links.

## Verdict

PASS — no findings in the changed scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

No AssuranceProfile is declared in this repository's spec tree. No enforced review operation was selected. Unavailable selected context: none. Active exceptions: none.

## Coverage

Reviewed AC-25 and its CR note for completeness, consistency and atomicity. It defines one response to an invalid semantic module: stop before document validation, emit the useful key/path diagnostic, return exit 3 and leave stdout empty. These are observables of one failure case and are testable together, rather than independent feature obligations. The test covers the three existing registry-loading branches.

Traceability context is US-003 to FR-004 to StR-004 (the US links StR-004), with Test verification for AC-25. The engine remains the semantic-contract owner and the CLI remains its process boundary. The new rule is consistent with FR-007's current invalid-request exit 3. Existing legacy exit wording in US-003 is outside the change and not used as the governing contract. The change adds no new external CLI/API, pagination, authentication, concurrency or lookup tie-breaking rule. The affected boundary's failure domain is retained module name plus absent archetypes; the criterion explicitly prevents the later unknown-type diagnostic from masking that failure.

## Examined units and bindings

```yaml
scope:
- id: FR-004-AC-25
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: If loading a module rejects an unknown key in its `semantic:` block, `validate`
    exits **3** before validating the document, with stderr identifying the semantic
    failure key and the module's `manifest.yaml` path; stdout is empty. This diagnostic
    takes precedence over an unknown document type.
- id: FR-004
  path: spec/functional/FR-004-validate-subcommand.md
  role: context_only
  excerpt: The CLI SHALL expose a single-mode (markdown-only) `validate` subcommand
    that structurally validates committed markdown artifacts against their archetype,
    delegating all validation and composition to `quire-rs`.
- id: FR-007-AC-3
  path: spec/functional/FR-007-exit-codes.md
  role: context_only
  excerpt: Unknown archetypes and invalid or missing input exit 3.
- id: StR-004
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: context_only
  excerpt: For artifact operations, the CLI SHALL be a **thin process boundary** over
    `quire-rs`.
bindings:
- test_id: semantic_module_failure_is_reported_before_unknown_document_type
  ac_id: FR-004-AC-25
  trace: correct
```

## Checks

Independent focused execution of current supplied compiled artifacts through locked-build.sh passed: semantic_module_failure_is_reported_before_unknown_document_type 1/1 (all three loading modes), it_050_unknown_archetype_reports_unknown 1/1, and missing_manifest_reports_real_reason_not_unknown_archetype 1/1. No selected tests were ignored. These were artifact reruns, not a fresh rebuild. The first grouped attempt used an incorrect cli_lint filename after the two validate tests passed and exited 127; the corrected cli_lint-a3936a1fbf2117e1 command then passed independently. No application failure is attributed to that command typo.

Independent cargo fmt --all --check, git diff --check, and quire validate --scope . spec/functional/FR-004-validate-subcommand.md all passed. Quire trace for FR-004-AC-25 reports the regression as verifies and load_registry as implements. Its two pre-existing cfg-twin symbol-collision warnings are in selftest::is_executable and safety::libc_eloop, outside the changed paths. Quire 0.52.0/engine 0.62.2 and Quoin 0.37.0 were used for review tooling. Fresh unique-ID exact-Test matrix: 432 tagged, 21 untagged and 15 ignored-only. AC-25 is Test/tagged; static binding is not a claim of bound execution evidence. Broad workspace lint/deny/test gates were not run under the owner's focused-only rule.

No Cargo.toml/Cargo.lock or dependency revision changed. The review does not infer release or Python-engine accessor coverage from this CLI fix. PR branch fix/semantic-module-errors and title fix: surface semantic module load failures remain neutral; the refreshed body contains Refs PLAT-1915. The lead explicitly assigned PLAT-1915 and verified its issue attachment, so ticket resolution is manual rather than inferred from branch/title. New/changed code and the regression now use the exact local AC-25; the earlier unmatched upstream FR-069-AC-3 claim is removed.

## Artifact retention

The lead/coder commits this scratchpad artifact as reviews/plat-1915-pr259-spec-review-integrity.md. The reviewer does not edit or push the branch.
