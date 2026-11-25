---
id: FR-047
title: "Run quire self-tests against the installed binary"
type: FR
relationships:
  - target: "ix://agent-ix/quoin/FR-229"
    type: "references"
  - target: "ix://agent-ix/quoin/FR-230"
    type: "references"
  - target: "ix://agent-ix/quoin/NFR-042"
    type: "constrains"
  - target: "ix://agent-ix/quoin/NFR-043"
    type: "constrains"
---

# FR-047: Run quire self-tests against the installed binary

## Description

The `quire` CLI SHALL let a user list and run the quire-cli and quire-rs
self-test cases shipped with the installed binary, so the user can verify
those tools without checking out either source repository. Quoin
[FR-229](ix://agent-ix/quoin/FR-229) defines shared case layout, execution,
comparison, status, and trace-marker behavior. Quoin
[FR-230](ix://agent-ix/quoin/FR-230) defines the versioned run-record format
and canonical retention. This requirement defines the quire-specific entry
point, case ownership, and embedding boundary. Quoin [FR-217](ix://agent-ix/quoin/FR-217)
owns combined validation reporting.

## Inputs

- The installed `quire` binary
- Case selection using the options defined by Quoin [FR-229](ix://agent-ix/quoin/FR-229)
- An optional `--out <file>` path for the Quoin [FR-230](ix://agent-ix/quoin/FR-230)
  run record; it is required when no repository evidence store is available

## Outputs

- The quire-cli and quire-rs case list for `quire selftest --list`
- A Quoin FR-230 run record from `quire selftest --run`, written to `--out <file>`
  when supplied or to standard output otherwise

## Behavior

- `quire selftest --list` SHALL identify the cases available from the installed
  binary.
- The `quire` CLI SHALL run every available case when no `--case` option is
  supplied. When one or more `--case <case-id>` options are supplied, it SHALL
  run only the named cases, once each, in declaration order.
- When a selected case id is unknown, the `quire` CLI SHALL reject it before
  starting any case and SHALL name the id in its error.
- The quire-cli repository SHALL own and embed its self-test cases. The linked
  quire-rs crate SHALL expose quire-rs-owned cases, authored and maintained in
  the quire-rs repository, to the installed `quire` binary.
- The `quire` CLI SHALL include at least one quire-rs-owned case exposed by the
  linked crate in its initial self-test case set.
- The quire-rs repository SHALL own the case source. Its trace marker SHALL
  identify an acceptance criterion in a quire-rs FR.
- A case or its result SHALL NOT constitute or imply an accepted
  `FeatureRiskRating`. Only the named-person workflow accepts a rating.
- The quire-cli repository SHALL NOT copy cases from quire-rs or qa-corpus.
- The quire-rs repository SHALL NOT copy cases from quire-cli or qa-corpus.
- For every case, its repository SHALL maintain a `required-tools` file at
  the case root. The file SHALL list required external executable names, one
  bare name per line in lexical order; an empty file means no external tools
  are required.
- The runner SHALL resolve each name against the `PATH` inherited by
  `quire selftest` in declaration order and SHALL consider it available only
  when the first matching path is a regular executable file.
- If any declared tool is unavailable, the runner SHALL skip that case script
  and record `not-executed` with the missing executable name and reason in the
  Quoin FR-230 run record. The runner SHALL NOT count the case as passing.
- If every declared tool is available, the runner SHALL run the case. The
  runner SHALL report execution failures according to Quoin
  [FR-229](ix://agent-ix/quoin/FR-229).
- The CLI SHALL implement the quire-cli case execution boundary required by
  Quoin [FR-229](ix://agent-ix/quoin/FR-229): each case process SHALL run under
  an operating-system-enforced write sandbox on Linux that permits
  writes beneath its `$CASE_TMP` directory and denies writes elsewhere. The
  sandbox SHALL expose denied write attempts to the runner so the run record
  marks that case `error` and identifies the attempted path, even if the case
  script handles the denial and exits successfully. On macOS, Windows, or any
  other unsupported operating system, `quire selftest --run` SHALL fail closed
  with `unsupported-platform` before starting a case and SHALL NOT report a
  case as passed. `quire selftest --list` MAY list embedded cases on an
  unsupported operating system; listing SHALL NOT imply that they can run
  there.
- `quire selftest --run` SHALL write each run record in the format and with
  the retention behavior defined by Quoin
  [FR-230](ix://agent-ix/quoin/FR-230). The `quire` command SHALL NOT assign
  process-risk ratings or generate the combined
  validation report, whose behavior is defined by Quoin
  [FR-217](ix://agent-ix/quoin/FR-217), [FR-218](ix://agent-ix/quoin/FR-218),
  [FR-219](ix://agent-ix/quoin/FR-219), and
  [FR-220](ix://agent-ix/quoin/FR-220).
- Each run record SHALL carry the authored anomaly documents embedded by
  quire-cli and exposed by its linked quire-rs crate. The documents SHALL retain
  their exact raw text and use stable source-qualified paths rooted at
  `quire-cli/` or `quire-rs/`. The anomaly set's `producerVersion` SHALL equal
  the run record's `toolVersion`. Quire CLI SHALL NOT add Quoin-owned anomaly
  documents to this set; Quoin FR-217 consumes the retained set when producing
  the combined validation report.
- The runner SHALL materialize cases from resources embedded in the installed
  binary. The runner SHALL NOT require a source checkout or network access,
  as specified by Quoin [NFR-042](ix://agent-ix/quoin/NFR-042).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-047-AC-1 | Given `quire selftest --list`, the command lists quire-cli-owned cases and quire-rs-owned cases exposed by the linked crate; `quire selftest --run` delegates shared case execution and result semantics to Quoin FR-229 and writes the resulting run record as defined by Quoin FR-230. | Test |
| FR-047-AC-2 | Given an installed `quire` binary on Linux and an empty `HOME`, `quire selftest --list` uses only embedded quire-cli and quire-rs cases; `quire selftest --run --out <path>` uses those cases and retains its record without source checkout or network access. Without a repository evidence store or `--out`, the command rejects before starting any case. | Test |
| FR-047-AC-3 | Given one untagged case in each owning repository, `quire matrix --repo agent-ix/quire-cli=<quire-cli-root> --repo agent-ix/quire-rs=<quire-rs-root> --strict` reports both cases as untagged with their source repository identities; for each corpus member, strict mode checks every `selftest/**/run.sh` for at least one shell trace marker that names an acceptance criterion in the corpus, and reports a missing or invalid marker with the exact script path. A quire-cli selftest case with a valid shell trace marker binds its named criterion. Trace tags remain in each repository's own selftest sources, and no case is copied between repositories. | Test |
| FR-047-AC-4 | Given a quire self-test run, `quire selftest --run` emits a run record conforming to Quoin FR-230; `--out <file>` writes those exact retained JSON bytes to the named file and emits no run-result JSON to stdout, while omitting `--out` writes those bytes to stdout only when a repository evidence store is available; without a store, the command rejects before starting any case. | Test |
| FR-047-AC-5 | Given a case declaring an executable absent from `PATH`, `quire selftest --run` skips its script and records `not-executed` with the missing executable name and reason; given all declared executables available, the runner executes the case under Quoin FR-229 semantics. | Test |
| FR-047-AC-6 | On Linux, given networking disabled and `HOME` set to an empty directory, the installed binary lists and runs its self-tests without making a network request or requiring a source checkout. | Test |
| FR-047-AC-7 | Inspection of the embedded case resources finds quire-cli cases sourced from this repository and quire-rs cases sourced from the quire-rs repository, with no cases copied between repositories or from qa-corpus. | Inspection |
| FR-047-AC-8 | Given run results from quire-cli and quire-rs, the `quire` self-test output contains only its own case/run evidence and does not generate or duplicate the combined tool validation report or its engineering-assurance and filament-analyze report behavior. | Inspection |
| FR-047-AC-9 | On Linux, given a case script that attempts to create a file at a parent-relative path and at an absolute path outside `$CASE_TMP`, the OS-enforced sandbox denies both writes, leaves both target paths absent, and the retained run record marks the case `error` with each denied path even when the script handles both denials and exits 0. On macOS, Windows, or another unsupported operating system, the CLI fails closed before case execution and reports no case as passed. | Test |
| FR-047-AC-10 | Given no `--case` option, `quire selftest --run` runs all available quire-cli and quire-rs cases; given repeated `--case <id>` options, it runs only the named available cases once each in declaration order, and an unknown id is rejected before any case starts and named in the error. The initial case set includes at least one quire-rs-owned case whose trace marker names a quire-rs acceptance criterion; its source and trace remain in quire-rs, and listing or running it does not imply an accepted risk rating. | Test |
| FR-047-AC-11 | Given a selftest run, its anomaly set contains each authored anomaly document embedded by quire-cli and linked quire-rs with exact raw text and stable paths prefixed by the owning source repository; it contains no Quoin records, and its `producerVersion` exactly equals the run record's `toolVersion`. | Test |

## Dependencies

- **Upstream**: Quoin [FR-229](ix://agent-ix/quoin/FR-229) defines the shared
  self-test case conventions; Quoin [FR-230](ix://agent-ix/quoin/FR-230)
  defines canonical retained run records; Quoin [NFR-042](ix://agent-ix/quoin/NFR-042)
  and [NFR-043](ix://agent-ix/quoin/NFR-043) constrain offline execution and
  fixture ownership; quire-rs owns its embedded self-test cases.
- **Downstream**: Quoin [FR-217](ix://agent-ix/quoin/FR-217) consumes retained
  quire-cli and quire-rs run evidence for the combined validation report.
