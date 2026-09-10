---
id: SR-063
title: "Rust review of native npm distribution tooling"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-research#61; tools/quire-dist; npm/quire-cli; Cargo.toml; Cargo.lock; Makefile; .github/workflows/release.yml; tests/toolchain_policy.rs; FR-022 through FR-024; NFR-008; ADR-0002"
review_set: subset
---

## Summary

Self-review used `/home/peter/dev/agent-skills/rust-review/SKILL.md` against the
owner-approved, base-reviewed #61 specification. The implementation is a
non-published Rust workspace tool rather than a second installed product
binary. It replaces the MJS package generator and shell/Perl/Node version
mutator, owns one typed target catalog, and leaves one owner-approved Node file
as the npm host seam only.

The verdict is **PASS after fixes**. Seven findings were reproduced and closed.
The final implementation adds no Quire parser, extraction behavior, formal
clause/profile grammar, source semantics, temporal/protocol behavior, WASM
surface, unsafe Rust, network client, hosted-CI dispatch, or publication.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | medium | **Fixed.** Binary identity validation initially read each entire release executable only to inspect its header. It now reads at most 4 KiB and checks the target-specific ELF machine, Mach-O CPU type, or PE/COFF machine. | `tools/quire-dist/src/lib.rs` `verify_binary_format`; FR-023-AC-3 | implementation-bug-despite-evidence |
| FND-002 | high | **Fixed.** If replacing `npm/dist` failed and restoring the renamed prior tree also failed, temporary-directory cleanup could delete the only recovery copy. The rollback now preserves the recovery directory and reports both failures and its exact path. | `tools/quire-dist/src/lib.rs` `replace_output_tree`; FR-023-AC-7 | implementation-bug-despite-evidence |
| FND-003 | medium | **Fixed.** The first CLI exposed arbitrary manifest, output, launcher, and license paths even though this is repository-local tooling and output replacement is destructive. The executable now accepts one repository root and derives those fixed loci; only the artifact input remains selectable. | `tools/quire-dist/src/main.rs`; NFR-008 | implementation-bug-despite-evidence |
| FND-004 | medium | **Fixed.** `NamedTempFile` made replaced Cargo/npm metadata mode `0600`. Atomic metadata writes now set `0644`, and TC-816 asserts launcher manifest and license modes. | `tools/quire-dist/src/lib.rs` `write_atomic`; `tools/quire-dist/tests/distribution.rs`; FR-023-AC-4 | implementation-bug-despite-evidence |
| FND-005 | medium | **Fixed.** The synthetic npm fixture omitted the README that npm always includes, so its member census was not the real package. The fixture now carries the real README, and package preflight requires regular launcher and README files before changing output. | `tools/quire-dist/tests/npm_host.rs`; `tools/quire-dist/src/lib.rs` `package_npm`; FR-023-AC-8 | correct-requirement-no-evidence |
| FND-006 | high | **Fixed.** Long multiline `#[trace(...)]` attributes compiled and tests passed, but Quire failed to bind IT-159 and three FR-023 criteria. Markers are now split into statically visible bare `#[trace(...)]` lines; coverage reports FR-022 7/7, FR-023 8/8, FR-024 6/6, and NFR-008 6/6 after TC-821. | `tools/quire-dist/tests/npm_host.rs`; `tools/quire-dist/tests/distribution.rs`; TM-001 | correct-requirement-no-evidence |
| FND-007 | low | **Fixed.** The release-host Node/npm versions were exact but not required by the production-derived policy mutation test. TC-819 now requires Node 22.15.0 and npm 11.6.2 and kills an npm-latest mutation. | `tools/quire-dist/tests/policy.rs`; NFR-008 | correct-requirement-no-evidence |

## Gate Results

| Gate | Result |
|---|---|
| `cargo +1.98.1 fmt --all -- --check` | PASS |
| `CARGO_BUILD_JOBS=2 cargo +1.98.1 clippy --workspace --locked --all-targets --all-features -- -D warnings` | PASS |
| `CARGO_BUILD_JOBS=2 cargo +1.98.1 test --locked --all-targets --all-features -- --test-threads=2` | PASS; 222 product unit/integration tests, including all 10 ptrace network/process tests |
| `CARGO_BUILD_JOBS=2 cargo +1.98.1 test --locked -p quire-dist -- --test-threads=1` | PASS; 14 distribution tests, including offline pack/install, every adverse host/artifact path, and the qualification-record gate |
| `RUSTDOCFLAGS="-D warnings" cargo +1.98.1 doc --workspace --locked --all-features --no-deps` | PASS |
| `cargo +1.98.1 build --locked --release`; `ldd` | PASS; release binary links only the prior baseline system libraries |
| `cargo +1.98.1 deny --locked check` | PASS; advisories, bans, licenses, and sources (three pre-existing unused-license allowances warn) |
| `cargo +1.98.1 audit` | PASS; 208 dependencies scanned, no vulnerability reported |
| Unsafe, thin-boundary, exact-toolchain, Makefile-lock, and distribution-policy mutation audits | PASS |
| Targeted Quire validation | PASS; 14/14 reviewed documents grammar-clean, zero grammar findings |
| New trace coverage | PASS; FR-022 7/7, FR-023 8/8, FR-024 6/6, NFR-008 6/6; IT-159..164 and TC-815..821 backed |

## Qualification Environment

- Rust and Cargo: exact 1.98.1.
- Build concurrency: `CARGO_BUILD_JOBS=2`; product tests used two threads and
  distribution tests used one.
- Local npm host: Node v22.15.0 and npm 10.9.2, used only with `--offline`,
  `--ignore-scripts`, and no audit/fund network operations.
- Manual release pins: Node 22.15.0 and npm 11.6.2, mutation-enforced by TC-819.
- License posture: the Rust workspace passes cargo-deny; the launcher imports
  only Node built-ins; all five npm artifacts declare `AGPL-3.0-or-later`, and
  pack inspection verifies carried license bytes.
- No GitHub workflow was dispatched. No GitHub release, Cargo package, or npm
  package was published.

## Boundary Review

The Node launcher derives support from the Rust-generated optional dependency
set and contains no second target allowlist. Rust owns all package/version/
binary validation and every test verdict. Generic GitHub archive/checksum shell
remains unchanged and routed to #63. Quire language/profile redesign and Agent
C extraction work remain separate; the distribution tool has no `quire-rs`
dependency and treats native binaries as opaque files.
