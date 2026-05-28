---
id: NFR-006
title: "CLI surface stability under SemVer"
artifact_type: NFR
quality_attribute: compatibility
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Constraint

The set of subcommands, their required arguments, and their exit-code semantics SHALL be considered the **public API** of this crate for SemVer purposes.

- Adding a new subcommand, new optional flag, or new exit code distinguished by stderr diagnostic class: **MINOR** version bump.
- Removing a subcommand, renaming a required argument, changing an exit code's meaning, or making a previously-optional flag required: **MAJOR** version bump and a one-release deprecation cycle (warn-on-stderr in the prior release).
- Adding a previously-undocumented optional flag, fixing bugs, internal refactors: **PATCH** bump.

JSON output schemas for `parse` and `extract` follow the same rule; field additions are MINOR, field removals or renames are MAJOR.

## Acceptance

- **NFR-006-AC-1**: A CHANGELOG entry documents every flag change, subcommand change, or exit-code change with the corresponding SemVer label.
- **NFR-006-AC-2**: A snapshot test pins `quire --help` output; updates require explicit reviewer approval.
