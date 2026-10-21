---
id: SR-14560
title: code-review review of IR-736 distributed lock gate
type: SpecReview
analysis: code-review
scope: agent-ix/quire-cli; IR-736; src/, tests/, tools/, Cargo.toml, Cargo.lock, README.md,
  complete computed spec matrix
review_set: subset
---

## Summary

Frozen candidate: `2005e457edf9485a0b84f4f30e929a5297f61caa`. Ticket: IR-736. Reviewer-only: no product edits, Cargo, rustc, metadata, build, native tests or aggregate gate executed by this reviewer. Existing producer receipts were inspected. Model identifier is not exposed by this harness and is recorded as unavailable rather than guessed.

## Verdict

PASS — no implementation defect found in the reviewed candidate. Exact-head candidate Titan installation is qualified; released and other-host installations are unqualified. Final aggregate remains pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

```yaml
- id: FR-027-AC-1
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: A lock with two or more distinct package names from one agent-ix repository at the
    same resolved commit passes; empty and registry/path-only locks also pass.
- id: FR-027-AC-2
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Distinct package names from one canonical repository at different resolved commits
    fail with exit 2 and diagnostics naming the lock, repository, all affected names, and revisions.
- id: FR-027-AC-3
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Duplicate agent-ix Git package names fail even with identical sources or different
    repositories; legitimate repeated registry and non-agent-ix package names pass.
- id: FR-027-AC-4
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'Branch/tag/rev/no-query and terminal .git/slash variants normalize to one repository:
    aligned fragments pass and split fragments fail, including GitHub case and transport variations.'
- id: FR-027-AC-5
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Missing, empty, shortened, and non-hex agent-ix resolved fragments, malformed URLs,
    invalid package field types, and malformed TOML fail as invalid input rather than clean
    results.
- id: FR-027-AC-6
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Multiple independently aligned locks may use different revisions; any failing lock
    fails the invocation and its diagnostic names that lock. A missing or unreadable path cannot
    yield a successful report.
- id: FR-027-AC-7
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Success and failure streams, global diagnostic formatting, missing arguments, stdin
    rejection, path traversal, and typed I/O outcomes conform to the existing CLI contract;
    repeated invocations produce identical observations.
- id: FR-027-AC-8
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: A split-revision fixture passes the original consumer nine-line AWK checker and fails
    this command. Qualification executes the existing original checker externally without adding
    an executable copy to quire-cli.
- id: FR-027-AC-9
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Execution leaves all inputs unchanged and performs no network, Cargo, Git, or shell
    invocation.
- id: FR-027-AC-10
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: The normal quire-cli release/install route supplies this same command; installation
    and representative aligned/split invocation are qualified separately on each IR/CG/RT gate
    host, with unavailable hosts explicitly unqualified.
- id: FR-027-CON-1
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Coherence logic SHALL have one Rust owner in quire-cli; consumer Makefiles invoke
    the released binary without copied executable checker logic.
- id: FR-027-CON-2
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: The command SHALL remain a developer gate without introducing runtime dependency
    edges into IR, CG, or RT.
- id: FR-027-CON-3
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Local qualification SHALL NOT publish a release or claim remote-host installation
    from a local receipt.
```

## Coverage

The maintained TOML and URL libraries own parsing; this module owns only the explicitly scoped Git coherence policy. Native command, shared typed failures, path guard, shared diagnostic renderer, sorted repository/package/revision sets, nine integration tests, two data fixtures and help snapshot were examined. All ten changed files were inspected; workflows are unchanged. No copied checker, catalog, checksum ledger, bypass, new unsafe, panic path, async/lock surface or duplicated helper was found. Source inventory of src/, tests/ and private tools maps existing CLI commands to FR-002..FR-027 and shared I/O/path/numeric/distribution requirements; stub scan finds only deliberate test-fixture TODO strings. This inventory is not a new semantic audit of every existing implementation.

Actual focused receipts: format, targeted locked Clippy, nine cli_check_lock tests and one help test pass at 0bd9457. Source/tests/manifests/lock bytes at the frozen candidate are identical to that receipt head (only README changed). Scope actual MemoryMax 16 GiB, MemorySwapMax 0; focused peak 411299840 bytes; zero memory events. Repository-wide aggregate remains the root lead's later gate, not asserted here.

Actual current-head external controls: original IR/CG/RT AWK checkers each return 0 on the same retained split input; native candidate returns 2 on split and 0 on aligned. Input files, originals and production source/lock remained byte-identical. Receipt external-controls-qualified.json succeeds at frozen head, peak 15753216 bytes. No checker code was copied into quire-cli.

Exact-head normal Git Cargo release installation PASS is now verified: install-titan-final-head.log/.json show resolved candidate 2005e457, actual 16 GiB/Swap0, 41.636 seconds, peak 630816768 bytes, no memory events. Cargo .crates2.json independently records that same full revision and release profile. Actual installed PATH invocation returns aligned 0 and split 2; installed traces each show exactly one execve and zero internet sockets with unchanged inputs. Only Titan candidate is qualified; release publication, remote gate hosts and IR-641 consumer adoption remain explicitly unqualified. The original workspace-ambiguous install failure and later stale branch-cache 0bd9457 receipt remain distinct preserved results, neither presented as final-head installation. README explicitly selects the existing quire-cli package; no new installer or product revision pin is introduced.

No AssuranceProfile applies (no AssuranceProfile artifact under spec/). Spec-review and its sub-analyses are not run: this candidate has no spec/ or plan/ changes; FR-027/US-008 were reviewed and merged in prerequisite PR132. This pass reads those contracts as implementation inputs. Full unsupported old CLI coverage remains owned separately by PLAT-1177.

Reconciliation: quire matrix --scope /home/peter/dev/worktrees/ir736-check-lock --format json (CLI 0.36.2, engine 0.50.2); no run evidence read. Criteria: 199 tagged, 51 untagged, 21 method-without-symbol; total 271. No ignored-only binder reported. The 21 inspection/method-only rows are distinguished from the 51 untagged rows.

Plan completion: not assessed

Optional semantic review: skipped; ordinary code-review spec/code/test alignment is assessed in SR-14560.
