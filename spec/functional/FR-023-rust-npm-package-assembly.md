---
id: FR-023
title: "Rust-owned npm package assembly"
type: FR
object_type: build_tool
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

# FR-023: Rust-owned npm package assembly

## Description

A repository-local Rust distribution tool SHALL generate the four native npm
packages and synchronize the launcher package from one typed target catalog.
It replaces `npm/build-packages.mjs`; it is build tooling and SHALL NOT become a
second installed `quire` product binary.

The catalog contains exactly these release targets:

| Rust target | Host platform | Host architecture | Binary |
|---|---|---|---|
| `x86_64-unknown-linux-musl` | `linux` | `x64` | `quire` |
| `aarch64-unknown-linux-musl` | `linux` | `arm64` | `quire` |
| `aarch64-apple-darwin` | `darwin` | `arm64` | `quire` |
| `x86_64-pc-windows-msvc` | `win32` | `x64` | `quire.exe` |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-023-AC-1 | One typed Rust catalog is the production authority for Rust target, npm platform, npm architecture, package name, binary name, and expected binary format for all four targets. | Unit/static test (TC-815) |
| FR-023-AC-2 | Before changing the output tree, the tool requires one input binary for every catalog target and rejects an absent, extra, or incorrectly named target artifact. | Mutation test (TC-817) |
| FR-023-AC-3 | Before changing the output tree, the tool verifies that every input has the target's expected ELF machine, Mach-O CPU type, or PE/COFF machine and rejects a binary copied from another target. | Mutation test (TC-817) |
| FR-023-AC-4 | A successful run replaces the output with exactly four scoped platform packages containing only `bin/quire[.exe]`, `LICENSE`, and `package.json`; non-Windows binaries are mode `0755`. | Test (TC-816) |
| FR-023-AC-5 | Every generated manifest declares the exact release version, `AGPL-3.0-or-later`, the matching singleton `os` and `cpu`, the public npm registry/access policy, and the repository metadata. | Test (TC-816) |
| FR-023-AC-6 | The launcher manifest receives the same release version and exactly one same-version optional dependency for every catalog target, with no stale or additional platform dependency; its license file is refreshed from the repository `LICENSE`. | Test (TC-816) |
| FR-023-AC-7 | Repeated generation from identical inputs is byte-identical, and any preflight refusal leaves the prior output and launcher files byte-identical. | Test (TC-816, TC-817) |
| FR-023-AC-8 | Local `npm pack` inspection and an offline clean consumer installation prove the generated file membership, dependency resolution, license presence, and native executable selection without publishing or network access. | Test (IT-159, IT-163) |

## Dependencies

- **Upstream**: FR-024 supplies an accepted release version; ADR-0002 defines
  the retained launcher host boundary.
- **Downstream**: the manually dispatched release workflow consumes the
  generated packages. GitHub release archive/checksum remediation is not part
  of this requirement and remains in `agent-ix/quire-research#63`.
