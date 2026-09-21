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

A repository-local Rust distribution tool SHALL generate one native npm
package per platform actually built in a given run, from one typed target
catalog, and synchronize the launcher package's `optionalDependencies` to
exactly that built set. It replaces `npm/build-packages.mjs`; it is build
tooling and SHALL NOT become a second installed `quire` product binary.

The catalog contains exactly these release targets:

| Rust target | Host platform | Host architecture | Binary |
|---|---|---|---|
| `x86_64-unknown-linux-musl` | `linux` | `x64` | `quire` |
| `aarch64-unknown-linux-musl` | `linux` | `arm64` | `quire` |
| `aarch64-apple-darwin` | `darwin` | `arm64` | `quire` |
| `x86_64-pc-windows-msvc` | `win32` | `x64` | `quire.exe` |

A run's built set is any non-empty subset of this catalog — the manually
dispatched GitHub release workflow builds all four; the local publish path
has only ever built `linux-x64` (PLAT-885). Both are legitimate releases. A
platform this run did not build is simply absent from the launcher's
`optionalDependencies`; the launcher's own "unsupported platform" refusal
(FR-022-AC-4) already handles that on the platforms it was never told about.
What packaging never does is declare a platform dependency at a version this
run did not build and publish — that dangling pin is the defect, not the
absence.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-023-AC-1 | One typed Rust catalog is the production authority for Rust target, npm platform, npm architecture, package name, binary name, and expected binary format for all four targets. | Unit/static test (TC-815) |
| FR-023-AC-2 | Before changing the output tree, the tool requires one input binary for each artifact target present — a non-empty subset of the catalog — and rejects an artifact directory outside the catalog, an empty artifact set, or an incorrectly named target artifact. | Mutation test (TC-817, TC-822) |
| FR-023-AC-3 | Before changing the output tree, the tool verifies that every input has the target's expected ELF machine, Mach-O CPU type, or PE/COFF machine and rejects a binary copied from another target. | Mutation test (TC-817) |
| FR-023-AC-4 | A successful run replaces the output with exactly one scoped platform package per artifact target present — never a package for a platform this run did not build — containing only `bin/quire[.exe]`, `LICENSE`, and `package.json`; non-Windows binaries are mode `0755`. | Test (TC-816, TC-822) |
| FR-023-AC-5 | Every generated manifest declares the exact release version, `AGPL-3.0-or-later`, the matching singleton `os` and `cpu`, the public npm registry/access policy, and the repository metadata. | Test (TC-816) |
| FR-023-AC-6 | The launcher manifest receives the same release version and exactly one same-version optional dependency for each artifact target present in this run — derived from the artifacts, never carried over from a prior run's committed manifest — with no dependency for a platform outside the catalog or absent from this run. | Test (TC-816, TC-822) |
| FR-023-AC-7 | Repeated generation from identical inputs is byte-identical, and any preflight refusal leaves the prior output and launcher files byte-identical. | Test (TC-816, TC-817) |
| FR-023-AC-8 | Local `npm pack` inspection and an offline clean consumer installation prove the generated file membership, dependency resolution, license presence, and native executable selection without publishing or network access. | Test (IT-159, IT-163) |
| FR-023-AC-9 | After publish, a separate command asserts that every platform package named in the launcher's `optionalDependencies` resolves at its declared version against a supplied npm registry; it fails on any package/version that does not resolve, on a registry it cannot reach, and on an empty `optionalDependencies` map (never a zero-iteration pass). `verify_binary_version` does not cover this — it compares only the local binary's own reported version and asserts nothing about what a registry actually serves. | Test (IT-165) |

## Dependencies

- **Upstream**: FR-024 supplies an accepted release version; ADR-0002 defines
  the retained launcher host boundary.
- **Downstream**: the manually dispatched release workflow consumes the
  generated packages. GitHub release archive/checksum remediation is not part
  of this requirement and remains in `agent-ix/quire-research#63`.
