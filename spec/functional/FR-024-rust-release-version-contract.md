---
id: FR-024
title: "Rust-owned npm release version contract"
type: FR
object_type: build_tool
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

# FR-024: Rust-owned npm release version contract

## Description

The repository-local Rust distribution tool SHALL replace
`scripts/set_version.sh` and npm-specific inline release assertions with a
typed, fail-closed version contract. The manually dispatched workflow remains
an external host: it supplies the requested version and invokes external build,
artifact, GitHub, and npm commands, but it does not reimplement version or npm
package decisions.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-024-AC-1 | The version command accepts a complete SemVer value and rejects missing or invalid input before changing any file. | Unit/mutation test (TC-818) |
| FR-024-AC-2 | A successful update changes only the root Cargo package version, launcher package version, and every catalog-derived launcher optional-dependency version to the identical supplied value. | Test (TC-818) |
| FR-024-AC-3 | The command validates the complete proposed Cargo and launcher state before its first replacement; a malformed launcher manifest or a missing or extra platform dependency leaves every governed file byte-identical. | Mutation test (TC-818) |
| FR-024-AC-4 | The release verification command rejects disagreement among the requested version, root Cargo package, launcher package, launcher optional dependencies, and native binary `--version` output. | Test (IT-164, TC-818) |
| FR-024-AC-5 | The manual release workflow invokes the Rust tool for npm package generation and version assertions and contains no executable MJS, Perl, embedded Node mutation, or independently maintained npm target/version assertion. | Static test (TC-819) |
| FR-024-AC-6 | Publication remains opt-in on a manually dispatched workflow; local qualification neither dispatches the workflow nor publishes a GitHub, Cargo, or npm artifact. | Static inspection (TC-819) |

## Dependencies

- **Upstream**: NFR-007 exact Rust 1.98.1 qualification.
- **Downstream**: FR-023 consumes the verified version when assembling npm
  packages.
