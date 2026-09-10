---
id: SR-059
title: "Rust review of exact Rust 1.98.1 qualification"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli#82; Cargo.toml; Cargo.lock; rust-toolchain.toml; clippy.toml; rustfmt.toml; deny.toml; Makefile; .github/workflows/*.yml; tests/toolchain_policy.rs; Rust-1.98.1 migration repairs"
review_set: subset
---

## Summary

Self-review performed with
`/home/peter/dev/agent-skills/rust-review/SKILL.md` after the reviewed NFR-007
and FR-020 pin specifications. Four findings were reproduced and fixed. The
final change contains no new public API, unsafe code, async/concurrency surface,
wire format, or runtime parsing path; its new executable policy logic is a Rust
integration test marked by `ix-trace-rs`.

## Verdict

**PASS after fixes.** No open Rust-review finding remains. Exact Rust 1.98.1
format, Clippy, all-target/all-feature tests, strict docs, release build,
dependency policy, security audit, static audits, and the release binary's
dynamic-link inspection pass locally. Hosted CI was not dispatched.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|----|----------|---------|------|--------------|
| FND-001 | high | **FIXED.** The canonical dependency gate ran only cargo-deny licenses and bans, so a new unknown Git source or advisory could pass `make ci`. `make deny` now executes the full policy, cargo-audit is in the aggregate, and the Rust audit requires both. | `Makefile:54`; `Makefile:58`; `Makefile:147`; `tests/toolchain_policy.rs:231` | correct-requirement-no-evidence |
| FND-002 | high | **FIXED.** The newly activated advisory gate found `anyhow` 1.0.102 affected by RUSTSEC-2026-0190 and `crossbeam-epoch` 0.9.18 affected by RUSTSEC-2026-0204. The lock now selects fixed 1.0.103 and 0.9.20; cargo-deny and cargo-audit both pass. | `Cargo.lock:79`; `Cargo.lock:246` | correct-requirement-no-evidence |
| FND-003 | medium | **FIXED before commit.** The first Rust audit could retry a failed file read, silently drop an unreadable workflow-directory entry, or associate a missing selector with the following action block. Reads are now single-shot typed findings, entry errors fail the production assertion, and selector search stops at the next action. | `tests/toolchain_policy.rs:47`; `tests/toolchain_policy.rs:76`; `tests/toolchain_policy.rs:171` | implementation-bug-despite-evidence |
| FND-004 | low | **FIXED.** Exact 1.98.1 qualification surfaced three `manual_repeat_n` lints, four invalid rustdoc HTML placeholders, and two nightly-only rustfmt keys. The equivalent stable APIs/docs are repaired and the unsupported keys removed without formatting drift. | `src/io.rs:178`; `src/commands/assurance.rs:83`; `src/commands/validate.rs:33`; `rustfmt.toml:1` | correct-requirement-no-evidence |

## Gate results

| Gate | Result |
|------|--------|
| `cargo +1.98.1 fmt --all -- --check` | PASS; zero nightly-option warnings |
| `cargo +1.98.1 clippy --locked --all-targets --all-features -- -D warnings` | PASS |
| `cargo +1.98.1 test --locked --all-targets --all-features` | PASS; every unit/integration target passed, including 10/10 strace network/process tests with ptrace enabled |
| `RUSTDOCFLAGS="-D warnings" cargo +1.98.1 doc --locked --all-features --no-deps` | PASS |
| `cargo +1.98.1 build --locked --release` | PASS |
| `ldd target/release/quire` | PASS; only linux-vdso, libgcc_s, libm, libc, and ld-linux |
| `cargo deny --locked check` | PASS; advisories, bans, licenses, and sources OK |
| `cargo audit` | PASS; 202 dependencies scanned, no vulnerability reported |
| `make audit-unsafe audit-thin-boundary audit-tool-drift` with Rust 1.98.1 | PASS |
| Changed-file `quire validate` for NFR-007, FR-020, Test Matrix, log, SR-057, SR-058 | PASS |

The first sandboxed full-suite attempt failed only because ptrace was denied;
the identical suite with ptrace enabled ran all ten strace tests and passed. The
whole specification bundle retains an unrelated pre-existing FR-021 structural
error (missing `Dependencies`); #82 neither causes nor masks it.
