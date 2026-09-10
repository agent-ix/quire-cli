---
id: SR-060
title: "Independent code review — exact Rust 1.98.1 qualification at c10dc3b"
type: SpecReview
analysis: code-review
scope: "PR #83 at c10dc3b against main 813019d; tests/toolchain_policy.rs; Makefile; deny.toml; Cargo.toml; rust-toolchain.toml; clippy.toml; NFR-007"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/spec/non-functional/NFR-007"
    type: reviews
---

## Summary

First independent review of #83 at `c10dc3b`. The change replaces a 55-line
Python drift checker with a 432-line Rust integration test that reads the same
policy inputs, and it is a real improvement: the audit is fail-closed on a
missing or unreadable input, it enumerates `.yml` and `.yaml`, and its mutation
loop derives its census from the tree rather than from a hand-written list, so a
new governed declaration cannot be added without the census assertion noticing.

Two gaps in that audit, both verified by running the branch's own code against a
probe rather than by reading it. Neither is a runtime defect; both are places
where the gate is narrower than NFR-007's stated verification.

## Verdict

**CONDITIONAL** — no high findings. Two mediums and one low.

## Gates run at `c10dc3b`

Exact Rust 1.98.1, isolated `CARGO_TARGET_DIR`.

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | pass |
| `cargo test --locked --all-targets --all-features` | pass, all suites |
| `RUSTDOCFLAGS="-D warnings" cargo doc --locked --all-features --no-deps` | pass |
| `cargo deny --locked check` | advisories, bans, licenses, sources ok |
| `cargo audit` | pass — **202 crate dependencies**, matching the PR body |

NFR-007-AC-2 verified independently: the pinned rev
`85dfe9d5a937c52af6456f2e6aa3a6bc4c82db9f` is the squash-merge commit of
`agent-ix/quire-rs#422` and is tree-identical to that PR's head `aac18a3`. The
pin names the qualification it claims to name.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | An action with no `@ref` at all escapes the action-pin gate entirely | tests/toolchain_policy.rs:126 | implementation-bug-despite-evidence |
| FND-002 | medium | The Makefile lane has zero mutation coverage; a broken rule there cannot fail the test | tests/toolchain_policy.rs:231 | correct-requirement-no-evidence |
| FND-003 | low | `ix-trace-rs` is pinned by a movable `tag`, while the sibling Git dependency is pinned by `rev` | Cargo.toml:28 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — the least-pinned form is the one that escapes

```rust
fn action_revision(line: &str) -> Option<&str> {
    let action = line.split_once("uses:")?.1.split_whitespace().next()?;
    action.rsplit_once('@').map(|(_, revision)| revision)
}
```

`rsplit_once('@')` returns `None` when there is no `@`, so `action_revision`
yields `None`, and the caller's `if let Some(revision)` body — the `is_full_sha`
check — never runs. A step written `uses: actions/checkout`, with no ref at all,
produces no finding. That is strictly less pinned than `uses: actions/checkout@v4`,
which *is* caught.

Demonstrated with a probe built from this branch's own `audit` and
`copy_policy_tree`, run under `cargo test` at this head and then deleted:

```
PROBE unpinned-no-at findings = []
```

The mutation suite cannot see this, because its action mutants replace a
40-hex revision with `v4` — a value that still has an `@`.

NFR-007's Verification section says the audit "refuses any value other than
1.98.1" for compiler selection and the AC table treats action pinning as part of
the same fail-closed audit. An absent ref is currently an accepted value.

Fix shape: treat a `uses:` line with no `@` as a finding rather than as
"nothing to check" — `action.rsplit_once('@')` becoming an `else` branch that
reports, plus one mutant that deletes `@<sha>`.

### FND-002 — `audit_makefile` is asserted only by the tree it audits

`governed_lines` builds its census from `Cargo.toml`, `rust-toolchain.toml`,
`clippy.toml` and the workflow files. `Makefile` is copied into every fixture by
`copy_policy_tree` but is never mutated. Verified against this branch:

```
PROBE governed Makefile declarations = 0
```

So three rules have positive-control coverage only — they pass because the
production tree happens to satisfy them:

- `make_cargo_command_needs_lock` — `--locked` on `$(CARGO) bench|build|check|clippy|test`
- the canonical `$(CARGO) deny --locked check` line, matched by exact string equality
- the `ci:` aggregate containing `cargo-audit`

If `make_cargo_command_needs_lock` were changed to return `false` unconditionally,
`tc142` would still pass. A gate that cannot fail is not yet evidence.

This does not violate NFR-007-AC-4 as written — a Makefile recipe is not a
"governed compiler declaration" — but the PR body claims the audit covers
"locked Cargo commands", and that claim rests on the untested half.

Note also that the deny check is exact-string: `$(CARGO) deny --locked check`.
Reordering to `$(CARGO) deny check --locked` keeps the same semantics and fails
the audit; a trailing comment does too. That is brittle rather than wrong, and a
Makefile mutant would have surfaced it.

### FND-003 — one Git dependency is pinned by rev, the other by tag

```toml
quire-rs   = { version = "=0.46.0", git = "…/quire-rs",   rev = "85dfe9d…" }
ix-trace-rs = { version = "=0.1.0",  git = "…/ix-trace-rs", tag = "v0.1.1" }
```

The PR body says it "version-pin[s] both Git packages". Both are version-pinned,
but only one is revision-pinned. A tag is movable at the origin; `rev` is not.
In practice the build is reproducible today because `Cargo.lock` records the
resolved commit:

```
source = "git+https://github.com/agent-ix/ix-trace-rs?tag=v0.1.1#2ce4ebf47f726b9d76388220545cd0abda8a5cfb"
```

so this is a manifest-provenance inconsistency, not a reproducibility defect.
Worth closing while `deny.toml` is being widened with `allow-git`, since that
allowance is what makes the origin trusted in the first place.

## Review checklist

- The Rust rewrite is idiomatic: `Finding` is a plain owned value with `PartialEq`
  for whole-vector assertion, errors are values rather than panics inside
  `audit`, and no `unwrap` reaches an input path — the two `expect` calls are in
  fixture setup, where a panic is the correct outcome.
- Fail-closed on absence is genuine and tested: an unreadable policy input, an
  unreadable workflow directory, and an empty workflow set each produce a
  finding rather than silence.
- The `dtolnay/rust-toolchain@` lookahead is bounded (`take(5)`) and stops at the
  next `- uses:`, so it cannot borrow a later step's `toolchain:` value.
- `#[trace("TC-142", "NFR-007-AC-1", "NFR-007-AC-4")]` is present and the
  repository's `ix-trace-rs` marker convention is followed.
- Removing the two nightly-only rustfmt options is correct for an exact-stable
  policy and changed no file's formatting.
- Widening `make lint`/`make test` to `--all-features` and `make deny` to the
  full four-check run are real strengthenings, and `cargo-audit` entering `ci:`
  is what NFR-007's Verification list requires.
- The pre-existing FR-021 missing-`Dependencies` structural error is declared in
  the PR body and is not introduced here; it stays open against its own ticket.
