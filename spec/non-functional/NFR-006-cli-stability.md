---
id: NFR-006
title: "CLI surface stability under SemVer"
type: NFR
quality_attribute: compatibility
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "traces_to"
    cardinality: "1:1"
---

## Statement

The set of subcommands, their required arguments, and their exit-code semantics
SHALL be considered the **public API** of this crate for SemVer purposes:

- Adding a new subcommand, new optional flag, or new exit code distinguished by stderr diagnostic class: **MINOR** version bump.
- Removing a subcommand, renaming a required argument, changing an exit code's meaning, or making a previously-optional flag required: **MAJOR** version bump and a one-release deprecation cycle (warn-on-stderr in the prior release).
- Adding a previously-undocumented optional flag, fixing bugs, internal refactors: **PATCH** bump.

JSON output schemas for `parse` and `extract` follow the same rule; field additions are MINOR, field removals or renames are MAJOR.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Surface changes (flag/subcommand/exit-code) released without a SemVer-labelled CHANGELOG entry | 0 | 0 | CHANGELOG review |
| Unreviewed `quire --help` snapshot drift merged | 0 | 0 | Snapshot test |

## Verification

A CHANGELOG review confirms every flag, subcommand, or exit-code change carries a
SemVer label, and a snapshot test pins `quire --help` so any surface drift fails
CI until a reviewer explicitly approves the updated snapshot.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-006-AC-1 | A CHANGELOG entry documents every flag change, subcommand change, or exit-code change with the corresponding SemVer label | Inspection |
| NFR-006-AC-2 | A snapshot test pins `quire --help` output; updates require explicit reviewer approval | Test |
