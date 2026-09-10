---
id: SR-064
title: "Rust review of native CLI qualification tooling"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli#85; tools/quire-qualify; Cargo.toml; Cargo.lock; Makefile; tests/audit_static.rs; tests/toolchain_policy.rs; FR-025; NFR-003; NFR-009; TM-001"
review_set: subset
---

## Summary

Self-review used `/home/peter/dev/agent-skills/rust-review/SKILL.md` against the
base-reviewed #85 specification. The implementation is one private Rust
workspace package. It replaces four executable Python/shell qualification
scripts and one inline Python benchmark decoder without changing product CLI
behavior, the npm distribution host, Quire language semantics, or hosted
workflows.

The verdict is **PASS after fixes**. Ten findings were reproduced and closed.
The final gates use exact Rust 1.98.1, locked resolution, `CARGO_BUILD_JOBS=2`,
canonical `ix-trace-rs` bindings, and local execution only.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | **Fixed.** Coverage intake initially treated unmatched records as arbitrary JSON and searched their rendered bytes. It now requires the typed `trace_id` field and rejects malformed or wrongly typed records before evaluation. | `tools/quire-qualify/src/coverage.rs:19`; `tools/quire-qualify/tests/coverage.rs:100`; FR-025-AC-2 | implementation-bug-despite-evidence |
| FND-002 | high | **Fixed.** Repeated minted ids could overwrite an earlier target and make contradictory coverage appear valid. The typed projection now rejects duplicate ids before missing/backed evaluation. | `tools/quire-qualify/src/coverage.rs:53`; `tools/quire-qualify/tests/coverage.rs:139`; FR-025-AC-1 | implementation-bug-despite-evidence |
| FND-003 | high | **Fixed.** A first AST pass could miss forbidden references hidden by aliases, nested-group imports, `extern crate` renames, or parseable macro bodies. The syntax index now expands those forms and mutation fixtures exercise each escape. | `tools/quire-qualify/src/source.rs:67`; `tools/quire-qualify/src/source.rs:121`; `tools/quire-qualify/src/source.rs:178`; `tools/quire-qualify/tests/source.rs:36`; FR-025-AC-4 | implementation-bug-despite-evidence |
| FND-004 | high | **Fixed.** Source and unsafe-baseline walkers initially allowed symlink redirection. Rust source symlinks and both live and broken baseline symlinks now fail closed before reads or writes. | `tools/quire-qualify/src/walk.rs:28`; `tools/quire-qualify/src/unsafe_comments.rs:74`; `tools/quire-qualify/tests/source.rs:100`; `tools/quire-qualify/tests/unsafe_comments.rs:95`; FR-025-AC-4; FR-025-AC-5 | implementation-bug-despite-evidence |
| FND-005 | high | **Fixed.** Text resembling `// SAFETY:` inside a Rust string literal could satisfy the first unsafe-comment implementation. Literal spans are now excluded, and the mutation is covered with exact/stale baseline behavior. | `tools/quire-qualify/src/source.rs:172`; `tools/quire-qualify/src/source.rs:526`; `tools/quire-qualify/tests/unsafe_comments.rs:12`; FR-025-AC-5 | implementation-bug-despite-evidence |
| FND-006 | medium | **Fixed.** The initial package returned unstructured string errors and used unchecked collection indexing in verdict paths. One `thiserror` boundary now retains causes and typed refusal variants; fallible collection access is explicit. | `tools/quire-qualify/src/error.rs:39`; `tools/quire-qualify/src/benchmark.rs:44`; `tools/quire-qualify/src/coverage.rs:48` | implementation-bug-despite-evidence |
| FND-007 | medium | **Fixed.** The root test aggregate initially exercised only the default package, so the new qualification package could fail while `make test` passed. The aggregate now tests the locked whole workspace with all targets and features. | `Makefile:43`; TC-840 | correct-requirement-no-evidence |
| FND-008 | medium | **Fixed.** The local composite lacked strict workspace rustdoc. `make docs` now applies `-D warnings` to all workspace packages and participates in the local composite gate. | `Makefile:47`; `Makefile:167`; NFR-009 | correct-requirement-no-evidence |
| FND-009 | medium | **Fixed.** The new path dev-dependency had no exact version, weakening the workspace's exact dependency declaration policy. The root manifest now declares `quire-qualify =0.1.0` at its path. | `Cargo.toml:29`; NFR-009 | implementation-bug-despite-evidence |
| FND-010 | high | **Fixed.** The proposed TC-815..821 identifiers collided with the already-landed npm-distribution tests, so trace coverage could bind the wrong implementation. FR-025/NFR-009 and TM-001 now own the unique TC-828..840 range, and the policy test enumerates every marker. | `spec/tests.md`; `tools/quire-qualify/tests/policy.rs:179`; TM-001 | wrong-requirement |

## Gate Results

| Gate | Result |
|---|---|
| `cargo +1.98.1 fmt --all -- --check` | PASS |
| `CARGO_BUILD_JOBS=2 cargo +1.98.1 clippy --workspace --locked --all-targets --all-features -- -D warnings` | PASS |
| `CARGO_BUILD_JOBS=2 cargo +1.98.1 test --workspace --locked --all-targets --all-features -- --test-threads=2` | PASS; every product, distribution, qualification, ptrace/strace, and policy test passed |
| `CARGO_BUILD_JOBS=2 cargo +1.98.1 test --locked -p quire-qualify -- --test-threads=1` | PASS; 15 qualification integration tests |
| `RUSTDOCFLAGS="-D warnings" cargo +1.98.1 doc --workspace --locked --no-deps --all-features` | PASS |
| `cargo +1.98.1 deny --locked check` | PASS; advisories, bans, licenses, and sources (three pre-existing unused-license allowances warn) |
| `cargo +1.98.1 audit` | PASS; 210 dependencies scanned after loading 1,243 advisories |
| `make audit-unsafe audit-thin-boundary spec` | PASS; native unsafe and thin-boundary gates clean, 11/11 governed documents grammar-clean, 22/22 required assurance targets backed |
| Targeted Quire validation | PASS; all changed specification and review artifacts grammar-clean |
| Change inspection | PASS; no `.github` delta, both workflows remain manual-dispatch-only, and no hosted workflow was dispatched |

## Boundary Review

Rust owns typed coverage and hyperfine models, syntax-aware source inspection,
unsafe-locus lifecycle, and every verdict. Make only orchestrates those gates.
The npm launcher remains the sole JavaScript executable and contains no
qualification semantics. Declarative fixtures and specification data remain in
their native formats. The package has no `quire-rs` dependency and introduces
no product parser, extraction behavior, formal-clause profile, source
semantics, or cross-repository registry.
