---
id: SR-967175026302197768900474181941935084881
title: "Specification integrity review of PLAT-1918"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli; PLAT-1918; spec/functional/FR-025-rust-qualification-tooling.md, spec/validation/FRR-002-not-high-process-risk.md, src/commands/selftest.rs, src/commands/trace.rs, tests/cli_export.rs, tests/cli_selftest.rs, tests/snapshots/help.txt, tools/quire-qualify/src/coverage.rs, tools/quire-qualify/tests/coverage.rs, tools/quire-qualify/tests/policy.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1918. Reviewed committed head da5171b51ab65b7a7d36d2f680f434881806f0cf. The previously reported deleted-target and orphaned-constraint defects, and missing implements marker, are resolved. No additional defect in the PR changes was found.

No applicable AssuranceProfile exists under spec/; therefore no enforced review-policy run or human acceptance gate applies to this review. Review selection: requested subset (base and integrity). No dependency version move occurs in the reviewed committed diff.

## Verdict

CONDITIONAL — no new PR defect found; latest-source build gates and repository baseline assurance limitations are disclosed below. This is not a repository-wide assurance PASS.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Completeness mapping: StR-004 -> FR-025 -> AC-1..AC-6 -> qualification coverage, benchmark, source, unsafe, and policy tests. FR-025 references NFR-003 and NFR-009, with StR-004 providing the requirement owner. FRR-002 -> FR-025-AC-1..AC-6 remains accurate. No US requirement, object, relationship edge, external service, authentication, lookup tie-break, or external API was added. Structural deletion removes the overly broad script/JavaScript inventory obligations consistently across specification, validation mapping, production population, and tests.

Consistency, atomicity and failure-domain checks found no new contradictory obligations, ambiguous identity, purity violation, graph/topological dependency, or untestable remaining criterion caused by the deletion. The remaining six ACs each name observable gate outcomes. Existing syntax-parser, exact-locus unsafe baseline, explicit-update, malformed report, and threshold failures remain covered. This review does not rewrite pre-existing unrelated specification obligations.

## Validation and limitations

Commands ran on terra in /home/peter/dev/worktrees/codex-plat-1918-quire-cli-full-suite. Installed quoin: /home/peter/.npm-global/bin/quoin 0.36.8. The available checkout binary target/debug/quire reports 0.53.1 (engine 0.63.1), while source Cargo.toml is 0.53.2; engine-derived results are explicitly provisional until the lead supplies the rebuilt current binary.

cargo fmt --check passed. cargo clippy --locked --workspace --all-targets --all-features -- -D warnings failed before compilation: read-only /home/peter/.cargo-target/debug/.cargo-build-lock (exit 101). cargo deny --locked check failed acquiring read-only /home/peter/.cargo/advisory-dbs/db.lock (exit 1). These are unavailable gate evidence, not source-code defects. Focused coverage/policy checks previously passed 8/8; the lead owns the latest-source test run. No full suite was run by this reviewer.

Default spec validation exited 3 for missing FeatureRiskRating/IntendedUse archetypes. Explicit module loading with spec-artifacts-process v0.38.0 (/home/peter/dev/worktrees/codex-plat-1918-spec-artifacts-v038/spec_artifacts_process, caa02e57eefd3b8e86de77ca4ff35962a4c1f3c2) plus all other installed modules validates both changed specification documents (exit 0). It prints UnknownEdgeType segregates_for_requirement in the process module ADR. Full spec validation with these modules exits 3 only on untouched spec/evidence/suites.md (SuiteRegistry id SUITE-004 does not match the module id regex; required Suites table missing). This is disclosed baseline/tooling evidence, not a new PR finding.

## Examined scope

```yaml
{
  "scope": [
    {
      "id": "spec/functional/FR-025-rust-qualification-tooling.md",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "---\nid: FR-025\ntitle: \"Rust-owned CLI qualification tooling\"\ntype: FR\nrelationships:\n  - target: \"ix://agent-ix/quire-cli/StR-004\"\n    type: \"implements\"\n---\n\n# FR-025: Rust-owned CLI qualification tooling\n\n## Description\n\nThe repository SHALL expose one private Rust qualification package for its\nassurance-traceability, benchmark, thin-boundary, and unsafe-comment gates.\n\n## Inputs\n\n- A Quire coverage JSON report and the repository-owned target set.\n- A hyperfine JSON report and a finite non-neg"
    },
    {
      "id": "spec/validation/FRR-002-not-high-process-risk.md",
      "path": "spec/validation/FRR-002-not-high-process-risk.md",
      "role": "examined",
      "excerpt": "---\nid: FRR-002\ntitle: \"Quire CLI proposed not-high process-risk supplier ratings\"\ntype: FeatureRiskRating\nscope: supplier\nrating: not_high_process_risk\nstatus: proposed\nrelationships:\n  - target: \"ix://agent-ix/quire-cli/IU-001\"\n    type: uses_intended_use\n  - target: \"ix://agent-ix/quire-cli/FR-001\"\n    type: rates_feature\n  - target: \"ix://agent-ix/quire-cli/FR-002\"\n    type: rates_feature\n  - target: \"ix://agent-ix/quire-cli/FR-003\"\n    type: rates_feature\n  - target: \"ix://agent-ix/quire-cl"
    },
    {
      "id": "src/commands/selftest.rs",
      "path": "src/commands/selftest.rs",
      "role": "examined",
      "excerpt": "//! List and run the self-test cases embedded in the installed `quire` binary.\n\nuse std::collections::BTreeSet;\nuse std::fs;\nuse std::io::Read;\nuse std::path::{Component, Path, PathBuf};\nuse std::process::{Command, Stdio};\nuse std::thread;\nuse std::time::{Duration, Instant};\n\nuse anyhow::{bail, Context};\nuse clap::{ArgGroup, Parser};\nuse ix_trace_rs::implements;\nuse serde::Serialize;\nuse tempfile::TempDir;\n\nuse crate::commands::{failure, Ctx};\n\n#[derive(Parser, Debug)]\n#[command(group(\n    ArgGr"
    },
    {
      "id": "src/commands/trace.rs",
      "path": "src/commands/trace.rs",
      "role": "examined",
      "excerpt": "//! `quire trace` \u2014 structural forward/inverse lookup over the trace-search\n//! index (quire-rs FR-077, PLAT-844, `agent-ix/quire-rs#477`).\n//!\n//! # Why this command exists\n//!\n//! PLAT-844's own framing: agents hand-roll `grep`/`rg`/`jq` pipelines for\n//! \"what backs FR-047\" and \"what does this file verify\" \u2014 each slow, each\n//! subtly different, each able to silently miss a hit or, worse, silently\n//! promote a citation to evidence. The engine API alone does not change that;\n//! this subcomma"
    },
    {
      "id": "tests/cli_export.rs",
      "path": "tests/cli_export.rs",
      "role": "examined",
      "excerpt": "//! `export` ITs (FR-031, FR-032, FR-033).\n//!\n//! Quire launches only `engineering-assurance`. These tests run the installed\n//! `engineering-assurance producer-execute` over shell stand-ins for pandoc and\n//! typst, written into a temporary directory, so the repository holds no\n//! converter. The ignored tests need the real tools and fail naming the missing\n//! one.\n\nmod common;\n\nuse std::collections::BTreeMap;\nuse std::fs;\nuse std::os::unix::fs::{symlink, PermissionsExt};\nuse std::path::{Path"
    },
    {
      "id": "tests/cli_selftest.rs",
      "path": "tests/cli_selftest.rs",
      "role": "examined",
      "excerpt": "//! Process-boundary tests for the embedded quire self-test suite.\n\nmod common;\n\nuse std::fs;\n\nuse common::quire;\nuse ix_trace_rs::trace;\nuse serde_json::Value;\n\n#[trace(\"FR-047-AC-1\", \"FR-047-AC-10\")]\n#[test]\nfn list_includes_quire_cli_and_linked_quire_rs_cases() {\n    let output = quire()\n        .args([\"selftest\", \"--list\"])\n        .output()\n        .expect(\"selftest list starts\");\n    assert!(\n        output.status.success(),\n        \"stderr: {}\",\n        String::from_utf8_lossy(&output.std"
    },
    {
      "id": "tests/snapshots/help.txt",
      "path": "tests/snapshots/help.txt",
      "role": "examined",
      "excerpt": "Thin CLI over quire-rs (documents, contracts, clause sets, and assurance export)\n\nUsage: quire [OPTIONS] <COMMAND>\n\nCommands:\n  assurance      Emit source-grounded assurance facts through quire-rs assurance-v1\n  check-lock     Check agent-ix Git dependency coherence in Cargo.lock files\n  clauses        Evaluate or compare rights-aware module clause sets\n  publish-check  Refuse a module that carries restricted clause text\n  selftest       List and run the self-test cases embedded in this binary\n "
    },
    {
      "id": "tools/quire-qualify/src/coverage.rs",
      "path": "tools/quire-qualify/src/coverage.rs",
      "role": "examined",
      "excerpt": "//! Typed assurance-coverage qualification.\n\nuse std::collections::{BTreeMap, BTreeSet};\nuse std::path::Path;\n\nuse ix_trace_rs::implements;\nuse serde::Deserialize;\nuse serde_json::Value;\n\nuse crate::{read, Error, Result};\n\n#[derive(Debug, Deserialize)]\nstruct MintedTarget {\n    id: String,\n    backed: bool,\n    #[serde(flatten)]\n    _extensions: BTreeMap<String, Value>,\n}\n\n#[derive(Debug, Deserialize)]\nstruct UnmatchedTag {\n    trace_id: String,\n    #[serde(flatten)]\n    _extensions: BTreeMap<St"
    },
    {
      "id": "tools/quire-qualify/tests/coverage.rs",
      "path": "tools/quire-qualify/tests/coverage.rs",
      "role": "examined",
      "excerpt": "use ix_trace_rs::trace;\nuse serde_json::{json, Value};\n\nfn complete_report() -> Value {\n    let minted = quire_qualify::coverage::required_targets()\n        .into_iter()\n        .map(|id| json!({\"id\": id, \"backed\": true, \"origin\": \"retained-extra-field\"}))\n        .collect::<Vec<_>>();\n    json!({\n        \"minted_targets\": minted,\n        \"unmatched_tags\": [],\n        \"status_lies\": [],\n        \"untracked_symbols\": []\n    })\n}\n\nfn check(report: &Value) -> quire_qualify::Result<String> {\n    quir"
    },
    {
      "id": "tools/quire-qualify/tests/policy.rs",
      "path": "tools/quire-qualify/tests/policy.rs",
      "role": "examined",
      "excerpt": "use std::fs;\nuse std::path::{Path, PathBuf};\n\nuse ix_trace_rs::trace;\n\nfn repo_root() -> PathBuf {\n    Path::new(env!(\"CARGO_MANIFEST_DIR\"))\n        .parent()\n        .and_then(Path::parent)\n        .expect(\"tools/quire-qualify is two levels below the repository root\")\n        .to_path_buf()\n}\n\nfn recipe<'a>(makefile: &'a str, target: &str) -> Vec<&'a str> {\n    let marker = format!(\"{target}:\");\n    let mut lines = makefile.lines().skip_while(|line| *line != marker);\n    assert_eq!(\n        lin"
    },
    {
      "id": "FR-025-CON-1",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-CON-1 | The Rust-source gates SHALL NOT classify syntax with grep, regular expressions alone, or a hand-written lexer | Maintainability | Inspection + mutation |"
    },
    {
      "id": "FR-025-CON-2",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-CON-2 | Make targets SHALL contain orchestration only | Boundary | Static test |"
    },
    {
      "id": "FR-025-CON-3",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-CON-3 | Make targets SHALL NOT interpret coverage, benchmark, or Rust-source content | Boundary | Static test |"
    },
    {
      "id": "FR-025-AC-1",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-AC-1 | A complete backed assurance report passes, while each missing, unbacked, wrongly promoted, unmatched, lying, or untracked case fails naming the condition | Test |"
    },
    {
      "id": "FR-025-AC-2",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-AC-2 | Missing fields, wrong field types, malformed JSON, empty benchmark results, and non-finite or negative thresholds fail before a success observation is emitted | Test |"
    },
    {
      "id": "FR-025-AC-3",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-AC-3 | Nearest-rank p95 is computed deterministically and values on either side of the configured threshold produce opposite verdicts | Test |"
    },
    {
      "id": "FR-025-AC-4",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-AC-4 | The thin-boundary gate accepts admitted dispatch and rejects forbidden direct, aliased, and nested-group references with a file and syntax locus | Test |"
    },
    {
      "id": "FR-025-AC-5",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-AC-5 | The unsafe gate accepts a documented block, rejects an undocumented block, honours only exact baseline loci, rejects stale exemptions, and changes the baseline only through the explicit update operation | Test |"
    },
    {
      "id": "FR-025-AC-6",
      "path": "spec/functional/FR-025-rust-qualification-tooling.md",
      "role": "examined",
      "excerpt": "| FR-025-AC-6 | Repository Make targets invoke Rust-owned gates without inline report parsing or source classification | Test |"
    },
    {
      "id": "FR-047-AC-1",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "context_only",
      "excerpt": "| FR-047-AC-1 | Given `quire selftest --list`, the command lists quire-cli-owned cases and quire-rs-owned cases exposed by the linked crate; `quire selftest --run` delegates shared case execution and result semantics to Quoin FR-229 and writes the resulting run record as defined by Quoin FR-230. | Test |"
    },
    {
      "id": "FR-047-AC-2",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "examined",
      "excerpt": "| FR-047-AC-2 | Given an installed `quire` binary on Linux and an empty `HOME`, `quire selftest --list` uses only embedded quire-cli and quire-rs cases; `quire selftest --run --out <path>` uses those cases and retains its record without source checkout or network access. Without a repository evidence store or `--out`, the command rejects before starting any case. | Test |"
    },
    {
      "id": "FR-047-AC-3",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "context_only",
      "excerpt": "| FR-047-AC-3 | Given one untagged case in each owning repository, `quire matrix --repo agent-ix/quire-cli=<quire-cli-root> --repo agent-ix/quire-rs=<quire-rs-root> --strict` reports both cases as untagged with their source repository identities; trace tags remain in each repository's own selftest sources, and no case is copied between repositories. | Test |"
    },
    {
      "id": "FR-047-AC-4",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "examined",
      "excerpt": "| FR-047-AC-4 | Given a quire self-test run, `quire selftest --run` emits a run record conforming to Quoin FR-230; `--out <file>` writes those exact retained JSON bytes to the named file and emits no run-result JSON to stdout, while omitting `--out` writes those bytes to stdout only when a repository evidence store is available; without a store, the command rejects before starting any case. | Test |"
    },
    {
      "id": "FR-047-AC-5",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "context_only",
      "excerpt": "| FR-047-AC-5 | Given a case declaring an executable absent from `PATH`, `quire selftest --run` skips its script and records `not-executed` with the missing executable name and reason; given all declared executables available, the runner executes the case under Quoin FR-229 semantics. | Test |"
    },
    {
      "id": "FR-047-AC-6",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "examined",
      "excerpt": "| FR-047-AC-6 | On Linux, given networking disabled and `HOME` set to an empty directory, the installed binary lists and runs its self-tests without making a network request or requiring a source checkout. | Test |"
    },
    {
      "id": "FR-047-AC-7",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "context_only",
      "excerpt": "| FR-047-AC-7 | Inspection of the embedded case resources finds quire-cli cases sourced from this repository and quire-rs cases sourced from the quire-rs repository, with no cases copied between repositories or from qa-corpus. | Inspection |"
    },
    {
      "id": "FR-047-AC-8",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "context_only",
      "excerpt": "| FR-047-AC-8 | Given run results from quire-cli and quire-rs, the `quire` self-test output contains only its own case/run evidence and does not generate or duplicate the combined tool validation report or its engineering-assurance and filament-analyze report behavior. | Inspection |"
    },
    {
      "id": "FR-047-AC-9",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "context_only",
      "excerpt": "| FR-047-AC-9 | On Linux, given a case script that attempts to create a file at a parent-relative path and at an absolute path outside `$CASE_TMP`, the OS-enforced sandbox denies both writes, leaves both target paths absent, and the retained run record marks the case `error` with each denied path even when the script handles both denials and exits 0. On macOS, Windows, or another unsupported operating system, the CLI fails closed before case execution and reports no case as passed. | Test |"
    },
    {
      "id": "FR-047-AC-10",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "context_only",
      "excerpt": "| FR-047-AC-10 | Given no `--case` option, `quire selftest --run` runs all available quire-cli and quire-rs cases; given repeated `--case <id>` options, it runs only the named available cases once each in declaration order, and an unknown id is rejected before any case starts and named in the error. The initial case set includes at least one quire-rs-owned case whose trace marker names a quire-rs acceptance criterion; its source and trace remain in quire-rs, and listing or running it does not imply an accepted risk rating. | Test |"
    },
    {
      "id": "FR-047-AC-11",
      "path": "spec/functional/FR-047-quire-selftest.md",
      "role": "context_only",
      "excerpt": "| FR-047-AC-11 | Given a selftest run, its anomaly set contains each authored anomaly document embedded by quire-cli and linked quire-rs with exact raw text and stable paths prefixed by the owning source repository; it contains no Quoin records, and its `producerVersion` exactly equals the run record's `toolVersion`. | Test |"
    }
  ]
}
```

## Disposition pass 1

Latest reviewed head: 1def514125bbec3e707b4bf703ec63dc47a4ada6. Inspected exact delta da5171b..1def514125bbec3e707b4bf703ec63dc47a4ada6: the identical ix-trace-rs dependency is promoted from dev-dependencies to dependencies. Version =0.1.0 and revision 2ce4ebf47f726b9d76388220545cd0abda8a5cfb are unchanged; this is not a dependency revision move. The production dependency makes the existing bare implements attribute available in ordinary qualification-library builds. No additional code/spec change or new defect was found.

A default cargo check --locked -p quire-qualify --tests attempt remained blocked by the read-only target lock. A separate /tmp target check was started before the lead clarified the required locked-build runbook and was interrupted with exit 130 at that instruction. No result is claimed from it. Latest compile/tests must come from the lead's queued /home/peter/dev/worktrees/locked-build.sh run with CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0; reviewer will not launch a competing build.

Disposition status: no unresolved source finding, but merge readiness is not asserted while latest-source required gate evidence remains pending. Existing baseline/tooling limitations from the original review remain recorded. No GitHub approval submitted.

## Disposition pass 2

Reviewed head: 1def514125bbec3e707b4bf703ec63dc47a4ada6 (unchanged since disposition round 1). The lead reports successful locked post-rebase checks against quire 0.53.2 / engine 0.63.2: trace regression 1/1; export default template 1/1; help snapshot 1/1; both selftest sandbox cases 1/1 each; qualification policy 2/2; qualification coverage 6/6. These execution outcomes come from the lead's approved locked-build run, not a second reviewer build. The rebuilt checkout binary independently reports quire 0.53.2 (engine 0.63.2).

Reviewer independently reran current-binary matrix and trace (exit 0). required_targets resolves as an implements claim for FR-025-AC-1. Current repository matrix counts: {"method-without-symbol": 41, "tagged": 443, "tagged-by-ignored-test": 15, "untagged": 45}. Changed FR-025 criteria and constraints remain tagged. Baseline repository-wide assurance gaps remain disclosed; this does not assert a repository-wide assurance PASS.

The lead confirms changed FR-025 and FRR-002 validate exit 0 with process v0.38.0 plus installed modules, retaining only the existing process-module UnknownEdgeType warning. GitHub check state independently inspected: every executed check completed SUCCESS, with only the conditional CLA subcheck skipped. No code delta, new finding, or unresolved prior source finding remains. PR review outcome: mergeable=yes.

GitHub approval was attempted with `gh pr review 256 -R agent-ix/quire-cli --approve --body PLAT-1918`; exit 1, exact response: `GraphQL: Review Can not approve your own pull request (addPullRequestReview)`. Authenticated account and PR author both resolve to kreneskyp. No GitHub approval was created; independent-account approval or the authorized admin merge path is required by GitHub.
