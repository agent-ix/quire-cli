---
id: SR-14564
title: "Code review of quire-cli 0.38.0 compatibility release"
type: SpecReview
analysis: code-review
scope: "CHANGELOG.md, Cargo.toml, Cargo.lock, npm/quire-cli/package.json, src/commands/assurance.rs, spec/functional/FR-020-assurance-export-subcommand.md, spec/spec.md, spec/tests.md, spec/usecase/US-006-export-assurance-facts.md, tests/audit_no_network.rs, tests/cli_assurance.rs, tests/fixtures/assurance/v1.json, tests/cli_schema.rs, tests/fixtures/validate-mod/manifest.yaml"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/FR-009"
    type: references
---

## Summary

Code review examined the current semantic manifest loader fix, matrix engine pin, Cargo/npm version alignment, the assurance-v1 CLI contract update, and regression tests. The CLI source/revision arguments and output premise are removed to match quire-rs 0.50.4; no open code findings remain.

## Verdict

**PASS** — no open code findings; all 7 assurance integration tests pass locally, Clippy passes, and hosted CI passes on the release candidate.

## Assurance Context

No applicable AssuranceProfile was found in the Quire CLI spec tree for this release-pin and schema-test change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Rust Review

Rust review: version metadata is aligned at 0.38.0 across Cargo and npm packages; the CLI pins the merged Quire-RS 0.50.4 commit and Cargo.lock records the same revision. The changelog describes the breaking assurance-v1 source-premise removal. Tests exercise the real CLI process for both current semantic manifest compatibility and assurance export behavior. `cargo fmt`, Clippy, and all 7 assurance integration tests pass locally. The full local suite's cross-language probe could not run because Python lacks the `jsonschema` module; hosted CI passes on the release candidate.
