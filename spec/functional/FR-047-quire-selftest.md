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

The `quire` CLI SHALL let a user list and run the self-test cases shipped with
the installed binary, so the user can verify quire-cli and quire-rs behavior
without checking out either source repository. The command SHALL produce a
versioned JSON run result that can be retained as evidence. Quoin
[FR-216](ix://agent-ix/quoin/FR-216) owns canonical evidence-store retention,
and Quoin [FR-217](ix://agent-ix/quoin/FR-217) owns combined validation
reporting.

## Inputs

- The installed `quire` binary
- An optional case selection
- An optional `--out <file>` path for the run result

## Outputs

- The case list for `quire selftest --list`
- A run result for `quire selftest --run`, written as JSON to `--out <file>`
  when supplied or to standard output otherwise
- A non-zero exit status when any selected case fails or errors

## Behavior

- `quire selftest --list` SHALL identify the cases available from the installed
  binary.
- `quire selftest --run` SHALL run all available cases unless a case selection
  is supplied.
- The quire-cli repository SHALL store each quire-cli-owned case under
  `selftest/cases/<case-id>/` with `run.sh`, `input/`, and `expected/` entries.
- The quire-rs repository SHALL store and maintain each quire-rs-owned case.
  The linked quire-rs crate SHALL embed each such case for execution through
  the installed `quire` binary. Neither repository SHALL copy cases from the
  other repository or from qa-corpus.
- Each case script SHALL read the installed binary path from `$QUIRE`, create
  files only under `$CASE_TMP`, and include at least one trace marker naming an
  acceptance criterion it checks, such as `# FR-047-AC-1`.
- The `expected/` directory SHALL contain the expected exit code and stdout.
  The `expected/` directory MAY contain expected files.
  The runner SHALL compare the exit code, stdout bytes, and every listed
  expected file byte for byte.
- The runner SHALL label a case `pass` only when all expected outputs match,
  `fail` when an expected output does not match, and `error` when execution
  cannot complete. It SHALL retain a result for every case it attempted.
- The JSON run result SHALL identify its format version, tool name, exact
  `quire --version` output, operating system, architecture, suite, run
  timestamp, and each case's id, status, traced acceptance criteria, and
  failure or error detail when applicable.
- The JSON run result SHALL be suitable for retention as the quire-cli run
  record defined by Quoin [FR-216](ix://agent-ix/quoin/FR-216). The `quire`
  command SHALL NOT assign process-risk ratings or generate the combined
  validation report, whose behavior is defined by Quoin
  [FR-217](ix://agent-ix/quoin/FR-217), [FR-218](ix://agent-ix/quoin/FR-218),
  [FR-219](ix://agent-ix/quoin/FR-219), and
  [FR-220](ix://agent-ix/quoin/FR-220).
- The runner SHALL materialize cases from resources embedded in the installed
  binary and SHALL NOT require a source checkout or network access.
- While a case executes, the runner SHALL restrict files created by that case
  to its temporary case directory.
- When a required external tool is unavailable for a case, the runner SHALL
  mark that case `not-executed`, include the tool name and reason, retain the
  result, and SHALL NOT count the case as passing.
- The runner SHALL return a non-zero exit status when any attempted case is
  `fail`, `error`, or `not-executed`, while retaining results for all selected
  cases.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-047-AC-1 | Given `quire selftest --list`, the command lists quire-cli-owned cases and quire-rs-owned cases embedded by the linked crate; `quire selftest --run` executes a selected case through the installed binary and records its outcome. | Test |
| FR-047-AC-2 | Given an expected stdout file differing from actual stdout by one byte, the case is `fail`, its result names the case and traced criteria, and the command exits non-zero. | Test |
| FR-047-AC-3 | Given a case without a trace marker, `quire matrix --strict` over `selftest/` reports that case as untagged; a tagged case result preserves the exact criterion ids from its trace markers. | Test |
| FR-047-AC-4 | Given a run that passes, fails, or errors, the JSON result includes the format version, tool version output, OS, architecture, suite, timestamp, and result for every attempted case, and can be retained as a versioned run record. | Test |
| FR-047-AC-5 | Given a case that requires an unavailable external tool, the retained result marks it `not-executed`, names the tool and reason, does not present it as passing, and the command exits non-zero. | Test |
| FR-047-AC-6 | Given networking disabled and `HOME` set to an empty directory, the installed binary lists and runs its self-tests without making a network request or requiring a source checkout. | Test |
| FR-047-AC-7 | Inspection of the embedded case resources finds quire-cli cases sourced from this repository and quire-rs cases sourced from the quire-rs repository, with no cases copied between repositories or from qa-corpus. | Inspection |
| FR-047-AC-8 | Given run results from quire-cli and quire-rs, the `quire` self-test output contains only its own case/run evidence and does not generate or duplicate the combined tool validation report or its engineering-assurance and filament-analyze report behavior. | Inspection |

## Dependencies

- **Upstream**: Quoin [FR-215](ix://agent-ix/quoin/FR-215) defines the shared
  self-test case conventions; Quoin [FR-216](ix://agent-ix/quoin/FR-216)
  defines canonical retained run records; Quoin [NFR-042](ix://agent-ix/quoin/NFR-042)
  and [NFR-043](ix://agent-ix/quoin/NFR-043) constrain offline execution and
  fixture ownership; quire-rs owns its embedded self-test cases.
- **Downstream**: Quoin [FR-217](ix://agent-ix/quoin/FR-217) consumes retained
  quire-cli and quire-rs run evidence for the combined validation report.
