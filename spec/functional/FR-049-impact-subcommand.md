---
id: FR-049
title: "quire impact subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-099"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/non-functional/NFR-006"
    type: "constrains"
    cardinality: "1:1"
---
# FR-049: quire impact subcommand

## Description

The `quire` CLI SHALL expose the source-change analysis produced by quire-rs
FR-099 as `quire impact --base <revision> --head <revision> [--repo <path>]`.
The command SHALL present changed source, selected tests, related symbols,
unlinked or unavailable areas, and the engine's reason chains without running
the selected tests or building the analyzed project.

## Inputs

- Required `--base <revision>` and `--head <revision>` Git revisions.
- Optional `--repo <path>` naming the local Git repository; when omitted, the
  current working directory is used.
- Optional `--format human|json`, defaulting to `human`.
- Optional `--caller-depth <n>`, a finite nonnegative depth forwarded to the
  engine for caller following.
- Optional `--check-tags` to include requirement-link findings in the impact
  report and `--fail-on-tag-findings` to request a failing exit status when
  those findings exist. `--fail-on-tag-findings` requires `--check-tags`.
- Zero or more repeatable `--dependency-move <repository-path> <old-revision>
  <new-revision>` groups naming local upstream snapshots to compare.

## Outputs

- A human-readable report by default, or a JSON report when
  `--format json` is selected, written to standard output.
- The JSON report SHALL retain the engine's snake_case fields:
  `base_revision`, `head_revision`, `changed_symbols`, `unmapped_changes`,
  `selected_tests`, `related_implementations`, `selection_complete`, and
  `findings`. When requested, caller-search and dependency-move sections are
  represented by `caller_search` and `dependency_moves` respectively.
- The report includes base/head revision identities, changed symbols and
  source locations, selected test names with Rust test-runner filters, related
  symbols, reason chains, unmapped changes, link findings when requested, and
  the engine's completeness and caller-search status.
- For each dependency move, the report includes the old and new revisions,
  changed upstream files and extracted items, local references, selected tests
  with upstream-to-local-to-test reason chains, and unmatched changes.
- Errors that prevent analysis are written to standard error and use the
  repository exit-code contract in [FR-007](./FR-007-exit-codes.md).

## Behavior

- The CLI SHALL read the requested revisions and tracked source and
  specification artifacts from the named local repository and SHALL pass the
  resulting snapshots to quire-rs FR-099. It SHALL resolve each revision to a
  full commit id and use the head revision's traceability model and
  specification artifacts. It SHALL NOT fetch revisions or modify the
  analyzed repository.
- The CLI SHALL pass every selected test filter and reason chain through from
  the engine without running a test, build, or target-project executable.
- The CLI SHALL render incomplete analysis, extraction failures, unresolved
  calls, unavailable specifications, and caller-depth truncation explicitly.
  An incomplete or unavailable result SHALL NOT be rendered as a complete
  empty selection or as evidence that unselected tests are unaffected.
- The CLI SHALL render the engine's supported-language and extraction status
  for each analyzed source area. Rust is the supported implementation
  language. When the CLI analyzes a source language other than Rust, it SHALL
  report that source as unsupported and uncovered. The CLI SHALL NOT assign a
  numeric extraction-confidence score.
- When `--check-tags` is supplied, the CLI SHALL show missing, invalid,
  unavailable, and non-binding requirement links returned by the engine.
  These findings SHALL NOT produce a nonzero status unless
  `--fail-on-tag-findings` is also supplied. If that option is supplied and at
  least one such finding exists, then the CLI SHALL return a nonzero status.
- For every `--dependency-move` group, the CLI SHALL pass the named upstream
  snapshots to the engine and present their findings separately from the
  local base/head diff. When an upstream revision or source snapshot is
  unavailable, the CLI SHALL identify the unavailable input. The CLI SHALL
  NOT represent that input as having no impact.
- For identical repository snapshots, revisions, options, and available
  specifications, the CLI SHALL produce byte-identical JSON output and stable
  ordering in both formats, without timestamps, temporary paths, or
  host-specific paths.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-049-CON-1 | The CLI SHALL delegate symbol extraction, trace binding, impact selection, and reason-chain construction to quire-rs FR-099. | Design | Inspection |
| FR-049-CON-2 | The CLI SHALL NOT implement a second parser, trace-marker grammar, or requirement hierarchy. | Design | Inspection |
| FR-049-CON-3 | The CLI SHALL NOT run builds, tests, or target-project executables while analyzing a change. | Design | Inspection |
| FR-049-CON-4 | The CLI SHALL NOT access the network or write into the analyzed repository while resolving source snapshots. | Security | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-049-AC-1 | Given a local repository and readable base/head revisions, `quire impact` reports changed symbols and source locations, selected tests with filters usable by the Rust test runner, related symbols, and complete reason chains from changed source to each result; the command does not run the selected tests or build the repository. | Test |
| FR-049-AC-2 | Given `--format json`, the command emits a machine-readable report with the engine's `base_revision`, `head_revision`, `changed_symbols`, `unmapped_changes`, `selected_tests`, `related_implementations`, `selection_complete`, and `findings` fields, plus `caller_search` or `dependency_moves` when requested; without the option it emits the corresponding human-readable information. | Test |
| FR-049-AC-3 | Given an unreadable revision, unavailable specification, extraction failure, unresolved call, or truncated caller search, the output names the affected scope and its status; it does not present incomplete analysis as an empty complete selection or state that unselected tests are unaffected. | Test |
| FR-049-AC-4 | Given a non-Rust changed source file, the report identifies the language as unsupported and the source as uncovered; for Rust extraction, it reports the engine's complete, partial, or unavailable status and does not emit a numeric confidence score. | Test |
| FR-049-AC-5 | Given `--check-tags`, missing, invalid, unavailable, and non-binding link findings appear in the report and do not fail the command by default; given `--fail-on-tag-findings` as well, the command exits nonzero when at least one such finding exists; using `--fail-on-tag-findings` without `--check-tags` is rejected. | Test |
| FR-049-AC-6 | Given one or more `--dependency-move <repository-path> <old-revision> <new-revision>` groups, the report separately identifies each old/new revision, changed upstream files and extracted items, local references, selected tests with upstream-to-local-to-test reason chains, and unmatched changes as uncovered; an unavailable upstream tree is explicit and is not reported as having no impact. | Test |
| FR-049-AC-7 | Given identical repository snapshots, revisions, options, and available specification artifacts, repeated invocations produce byte-identical JSON and stable result ordering without timestamps, temporary paths, or host-specific paths. | Test |
| FR-049-AC-8 | Inspection of the CLI implementation confirms it reads local snapshots without network access or writes to the analyzed repository, delegates analysis to quire-rs FR-099, and never runs the analyzed project, its build, or its tests. | Inspection |

## Dependencies

- **Upstream**: quire-rs [FR-099](ix://agent-ix/quire-rs/spec/functional/FR-099)
  defines diff-impact results; it reuses quire-rs
  [FR-062](ix://agent-ix/quire-rs/spec/functional/FR-062) for binding and
  [FR-077](ix://agent-ix/quire-rs/spec/functional/FR-077) for trace lookup.
  [NFR-006](../non-functional/NFR-006-cli-stability.md) governs the stable
  command and output contract.
