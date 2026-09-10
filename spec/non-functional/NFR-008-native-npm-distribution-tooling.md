---
id: NFR-008
title: "Native npm distribution tooling boundary"
type: NFR
quality_attribute: maintainability
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "constrains"
    cardinality: "1:1"
---

# NFR-008: Native npm distribution tooling boundary

## Statement

The repository-local distribution tool SHALL implement first-party npm
distribution decisions, generators, validators, and test assertions in Rust
1.98.1. The only approved executable foreign-language production file is the
minimal Node npm-bin host bounded by ADR-0002; all other foreign execution is an
explicitly named external package manager or release host.

## Scope and Containment

- Rust owns the typed target catalog, binary identity validation, package
  manifests, version synchronization, parity checks, and test assertions.
- Node executes only the installed npm `bin` host seam. Local Rust tests invoke
  an exact Node/npm toolchain to qualify that seam; Node owns no test assertion
  or package decision.
- The manual GitHub workflow may configure credentials and invoke pinned
  external actions, Cargo, GitHub CLI, npm, and the Rust distribution tool.
  Generic GitHub release archive/checksum remediation is separately owned by
  `agent-ix/quire-research#63`.
- `npm/build-packages.mjs`, `scripts/set_version.sh`, and embedded Node/Perl
  mutators are removed rather than wrapped.
- WASM/browser parity is deferred by the owner and is not an admission or exit
  gate for this change.
- The distribution tool does not depend on `quire-rs` and does not parse Quire
  artifacts. It encodes no extraction rule, formal-clause profile, grammar,
  temporal/protocol behavior, or source semantic; those remain governed by the
  Quire language redesign.

## Executable-Path Inventory and Disposition

| Existing path | Classification | #61 disposition |
|---|---|---|
| `npm/quire-cli/bin/quire.js` | Shipped npm executable host | Retain under the owner-approved ADR-0002 exact bounded seam. |
| `npm/build-packages.mjs` | First-party package generator and manifest mutator | Replace with the Rust distribution tool and remove. |
| `scripts/set_version.sh` | First-party SemVer validator plus Perl/Node mutator | Replace with the Rust distribution tool and remove. |
| npm target/version assertions inside `.github/workflows/release.yml` | Executable inline decision and validation logic | Replace with calls to the Rust distribution tool. |
| Pinned GitHub actions, Cargo builds, artifact transfer, credential setup, `gh release`, and `npm publish` | External release-host orchestration | Retain as manually dispatched host integration; it owns no package semantics or test verdict. |
| GitHub release archive normalization/checksum shell | Non-npm ecosystem packaging | Preserve in #61 and route remediation to `agent-ix/quire-research#63`. |
| WASM/browser paths | Deferred product surface | No implementation or qualification work in #61. |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| First-party npm decision/generator/assertion paths outside Rust | 0 | 0 | Executable-path audit |
| Foreign production files beyond the approved Node launcher | 0 | 0 | Executable-path audit |
| Unreviewed target catalogs | 0 | 0 | Catalog and workflow mutation test |
| Distribution dependencies on Quire parsing/extraction/profile crates | 0 | 0 | Manifest/source audit |

## Verification

`ix-trace-rs`-marked Rust tests SHALL exercise the generated packages, invoke
the exact Node/npm host tools where the npm protocol requires them, and assert
all results. A production-derived static audit SHALL enumerate the executable
paths and fail when foreign decision logic, an additional launcher, an
undeclared target catalog, or Quire semantic dependencies appear.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-008-AC-1 | The only repository-owned JavaScript executed in npm distribution is `npm/quire-cli/bin/quire.js`, and it remains within every ADR-0002 boundary. | Static/mutation test (TC-815, TC-819) |
| NFR-008-AC-2 | All distribution test or audit verdicts are asserted by Rust tests carrying `ix-trace-rs` links to the owning acceptance criteria. | Static test (TC-820) |
| NFR-008-AC-3 | The distribution tool and launcher contain no document, extraction, clause, profile, grammar, temporal, protocol, or source-semantic implementation and the Rust tool has no `quire-rs` dependency. | Static test (TC-820) |
| NFR-008-AC-4 | Local qualification uses `CARGO_BUILD_JOBS=2`, exact Rust 1.98.1, and no hosted-CI dispatch or publication. | Inspection (TC-821) |
| NFR-008-AC-5 | The retained Node/npm qualification reliance is limited to packing, clean offline installation, dependency resolution, and exercising the launcher process seam; the Rust tests decide every result. | Test/review (IT-159, IT-163, TC-821) |
| NFR-008-AC-6 | Rust distribution dependencies pass the repository's license/source policy, the launcher imports only Node built-ins, and every emitted npm package declares and carries the repository's `AGPL-3.0-or-later` license. | Test/inspection (IT-163, TC-821) |

## Dependencies

- **Upstream**: owner acceptance of ADR-0002.
- **Downstream**: `agent-ix/quire-research#63` may consume the executable-path
  inventory but SHALL NOT reopen this npm boundary without a new spec cycle.
