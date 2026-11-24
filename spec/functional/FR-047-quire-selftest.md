---
id: FR-047
title: "Run quire self-tests against the installed binary"
type: FR
relationships:
  - target: "ix://agent-ix/quoin/FR-215"
    type: "references"
  - target: "ix://agent-ix/quoin/FR-216"
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
[FR-215](ix://agent-ix/quoin/FR-215) defines shared case layout, execution,
comparison, status, and trace-marker behavior. Quoin
[FR-216](ix://agent-ix/quoin/FR-216) defines the versioned run-record format
and canonical retention. This requirement defines the quire-specific entry
point, case ownership, and embedding boundary. Quoin [FR-217](ix://agent-ix/quoin/FR-217)
owns combined validation reporting.

## Inputs

- The installed `quire` binary
- Case selection using the options defined by Quoin [FR-215](ix://agent-ix/quoin/FR-215)
- An optional `--out <file>` path for the Quoin [FR-216](ix://agent-ix/quoin/FR-216)
  run record

## Outputs

- The quire-cli and quire-rs case list for `quire selftest --list`
- A Quoin FR-216 run record from `quire selftest --run`, written to `--out <file>`
  when supplied or to standard output otherwise

## Behavior

- `quire selftest --list` SHALL identify the cases available from the installed
  binary.
- `quire selftest --run` SHALL run all available cases unless a case selection
  is supplied.
- The quire-cli repository SHALL own and embed its self-test cases. The linked
  quire-rs crate SHALL expose quire-rs-owned cases, authored and maintained in
  the quire-rs repository, to the installed `quire` binary.
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
  Quoin FR-216 run record. The runner SHALL NOT count the case as passing.
- If every declared tool is available, the runner SHALL run the case. The
  runner SHALL report execution failures according to Quoin
  [FR-215](ix://agent-ix/quoin/FR-215).
- The CLI SHALL implement the quire-cli case execution boundary required by
  Quoin [FR-215](ix://agent-ix/quoin/FR-215): each case process SHALL run under
  an operating-system-enforced write sandbox that permits writes beneath its
  `$CASE_TMP` directory and denies writes elsewhere. The sandbox SHALL expose
  denied write attempts to the runner so the run record marks that case
  `error` and identifies the attempted path, even if the case script handles
  the denial and exits successfully.
- `quire selftest --run` SHALL write each run record in the format and with
  the retention behavior defined by Quoin
  [FR-216](ix://agent-ix/quoin/FR-216). The `quire` command SHALL NOT assign
  process-risk ratings or generate the combined
  validation report, whose behavior is defined by Quoin
  [FR-217](ix://agent-ix/quoin/FR-217), [FR-218](ix://agent-ix/quoin/FR-218),
  [FR-219](ix://agent-ix/quoin/FR-219), and
  [FR-220](ix://agent-ix/quoin/FR-220).
- The runner SHALL materialize cases from resources embedded in the installed
  binary. The runner SHALL NOT require a source checkout or network access,
  as specified by Quoin [NFR-042](ix://agent-ix/quoin/NFR-042).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-047-AC-1 | Given `quire selftest --list`, the command lists quire-cli-owned cases and quire-rs-owned cases exposed by the linked crate; `quire selftest --run` delegates shared case execution and result semantics to Quoin FR-215 and writes the resulting run record as defined by Quoin FR-216. | Test |
| FR-047-AC-2 | Given an installed `quire` binary and an empty `HOME`, `quire selftest --list` and `quire selftest --run` use only embedded quire-cli and quire-rs cases and require no source checkout or network access. | Test |
| FR-047-AC-3 | Given one untagged case in each owning repository, `quire matrix --repo agent-ix/quire-cli=<quire-cli-root> --repo agent-ix/quire-rs=<quire-rs-root> --strict` reports both cases as untagged with their source repository identities; trace tags remain in each repository's own selftest sources, and no case is copied between repositories. | Test |
| FR-047-AC-4 | Given a quire self-test run, `quire selftest --run` emits a run record conforming to Quoin FR-216 and accepts `--out <file>` as its destination. | Test |
| FR-047-AC-5 | Given a case declaring an executable absent from `PATH`, `quire selftest --run` skips its script and records `not-executed` with the missing executable name and reason; given all declared executables available, the runner executes the case under Quoin FR-215 semantics. | Test |
| FR-047-AC-6 | Given networking disabled and `HOME` set to an empty directory, the installed binary lists and runs its self-tests without making a network request or requiring a source checkout. | Test |
| FR-047-AC-7 | Inspection of the embedded case resources finds quire-cli cases sourced from this repository and quire-rs cases sourced from the quire-rs repository, with no cases copied between repositories or from qa-corpus. | Inspection |
| FR-047-AC-8 | Given run results from quire-cli and quire-rs, the `quire` self-test output contains only its own case/run evidence and does not generate or duplicate the combined tool validation report or its engineering-assurance and filament-analyze report behavior. | Inspection |
| FR-047-AC-9 | Given a case script that attempts to create a file at a parent-relative path and at an absolute path outside `$CASE_TMP`, the OS-enforced sandbox denies both writes, leaves both target paths absent, and the retained run record marks the case `error` with each denied path even when the script handles both denials and exits 0. | Test |

## Dependencies

- **Upstream**: Quoin [FR-215](ix://agent-ix/quoin/FR-215) defines the shared
  self-test case conventions; Quoin [FR-216](ix://agent-ix/quoin/FR-216)
  defines canonical retained run records; Quoin [NFR-042](ix://agent-ix/quoin/NFR-042)
  and [NFR-043](ix://agent-ix/quoin/NFR-043) constrain offline execution and
  fixture ownership; quire-rs owns its embedded self-test cases.
- **Downstream**: Quoin [FR-217](ix://agent-ix/quoin/FR-217) consumes retained
  quire-cli and quire-rs run evidence for the combined validation report.
