---
id: SR-967175026302197768900474181941935084893
title: "gap analysis — PLAT-1915 PR #263"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli; spec/functional/FR-006-io-contract.md, spec/functional/FR-007-exit-codes.md, spec/functional/FR-020-assurance-export-subcommand.md, spec/functional/FR-022-npm-distribution.md, tests/release_workflow.rs; .github/workflows/tag-attestation.yml; .github/workflows/tag-attestation.yml (context)"
review_set: subset
---

## Summary

PR-scoped matrix and behavior review: each of the five candidate criteria has a real tagged binder. The PR-reported older-cohort baseline is 213 Test/Unit Test criteria with 10 untagged; after excluding two retired criteria, the withdrawn criterion and the two human-evidence criteria, 208 eligible remain with 0 previously untagged. Plan completion: not assessed. This is scoped to the PR criteria, not a whole-repository reverse-gap or stub inventory.

## Verdict

**PASS** — reviewed head `690be290a26cac7bcf39c77cdca4b07173850e90`; no findings in this method’s scope.

## Scope

- **FR-006-AC-4** (`spec/functional/FR-006-io-contract.md`, examined): `quire parse -` reads a document from piped stdin and emits the same result as `quire parse <file>` for identical input bytes.
- **FR-007-AC-7** (`spec/functional/FR-007-exit-codes.md`, examined): Strict coverage and matrix reports exit 1 for incomplete or failing records while emitting the complete requested payload.
- **FR-007-AC-9** (`spec/functional/FR-007-exit-codes.md`, examined): A converter failure of `quire export` (including failed Mermaid render) exits 4; a missing converter (including `mmdr`) exits 3.
- **FR-020-CON-5** (`spec/functional/FR-020-assurance-export-subcommand.md`, examined): A supported producer/consumer pair preserves every required field identity and meaning; unsupported shapes are refused without dropping or reinterpreting fields.
- **FR-022-AC-6** (`spec/functional/FR-022-npm-distribution.md`, examined): At the exact release candidate revision, the release owner runs `make test` and prepares a report with results, coverage, gaps, defects, and Quire/Quoin versions. Verification: Demonstration.
- **FR-022-AC-7** (`spec/functional/FR-022-npm-distribution.md`, examined): The release owner attaches the completed report manually before publishing; no automated attachment or attestation is claimed. Verification: Inspection.
- **FR-022-AC-8** (`spec/functional/FR-022-npm-distribution.md`, examined): If retained release evidence is unavailable, tag attestation fails closed without writing a release attestation or passing Test Result.
- **FR-022-AC-9** (`spec/functional/FR-022-npm-distribution.md`, examined): Withdrawn by owner decision; retained for history and never reused.
- **fail-closed workflow** (`tests/release_workflow.rs; .github/workflows/tag-attestation.yml`, context_only): The test parses the current workflow; the build has an unconditional no-evidence failure step and the attest job needs the build job.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |


## Coverage

Matrix source: `quoin matrix --repo . --json` using Quoin 0.37.0. This review checked the changed criteria and the explicit older-cohort exclusions. Plan completion: not assessed. Repository-wide reverse-gap and stub inventory was outside this PR-scoped pass.
