---
id: SR-20261102086808
title: "PLAT-1300 quire-cli PR158 code-review (Rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR158; src/commands/validate.rs, tests/cli_okf.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1300. Reviewed `git diff origin/main...HEAD` at head 4670eaf: `run_okf` now passes `repo_bundle_package(scope)` to `Spec::with_bundle_package`, plus new test `it089_okf_declares_repo_package_so_cross_repo_targets_are_not_dangling` and a `current_dir` change to `okf_tolerates_unknown_type_and_broken_link`. Rust lane (rust-review) folded in. Pinned quire-rs b2fe4b4 contains the cross-repo skip in `validate_bundle` (src/corpus/validate.rs). Focused `cargo test --test cli_okf`: 7 passed.

The default (no positional) path is correct. The positional-bundle path derives the package from `--scope` (default cwd), not from the bundle, so a bundle from another checkout inherits the caller's identity; reproduced with the built binary: an in-repo miss of the bundle's own repo is silently dropped when run from a different checkout and reported when run from the bundle's checkout. The edit to the existing test works around exactly this.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Bundle package comes from --scope (cwd), not the positional bundle root; an in-repo dangling ref in a bundle from another checkout is silently suppressed | src/commands/validate.rs:379 |
| FND-002 | low | Authority vs package comparison is case-sensitive and parse_remote_package does not normalize case; a remote spelled with different case than the links suppresses in-repo misses | src/commands/validate.rs:450 |

## Examined scope

```yaml
scope:
- id: run_okf
  path: src/commands/validate.rs
  role: examined
  excerpt: "let bundle_package = repo_bundle_package(scope); ... if let Some(package) = bundle_package.as_deref() { spec = spec.with_bundle_package(package); }"
- id: repo_bundle_package
  path: src/commands/validate.rs
  role: context_only
  excerpt: "fn repo_bundle_package(scope: &Path) -> Option<String> { git -C scope remote get-url origin ... }"
- id: okf_tolerates_unknown_type_and_broken_link
  path: tests/cli_okf.rs
  role: examined
  excerpt: "quire().current_dir(dir.path()).arg(\"validate\").arg(\"--okf\").arg(dir.path())"
- id: it089_okf_declares_repo_package_so_cross_repo_targets_are_not_dangling
  path: tests/cli_okf.rs
  role: examined
  excerpt: "cross [other](ix://other-org/other-repo/FR-9) own [missing](ix://acme/widgets/MISSING)"
```

## Verdict

Changes requested (medium). Main `--scope` path, error handling, and test assertions are sound; idiomatic Rust, no panics, no new unsafe. Fix FND-001 by resolving the package per root from `document_root` (git walks up from `<scope>/spec` to the same repo, so the default path is unchanged), revert the `current_dir` workaround, and add a positional-bundle-from-another-checkout test.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | Only the package side is lowercased; quire-rs still compares the link authority byte-for-byte, so an own-repo miss written with a mixed-case authority (ix://Acme/widgets/X) is now skipped as cross-repo. A full fix is a case-insensitive authority comparison in quire-rs | src/commands/validate.rs:458 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 8291a73: run_okf calls repo_bundle_package(&document_root) per root; current_dir workaround reverted; it179 covers a positional bundle from another cwd; reviewer repro now reports the own-repo miss |
| FND-002 | fixed | 8291a73: parse_remote_package lowercases org/repo; unit test remote_package_is_case_insensitive and it179 (git@github.com:Acme/Widgets.git) cover it; residual link-side case tracked as FND-003 |
