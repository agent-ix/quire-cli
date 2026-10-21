---
id: FR-027
title: "Cargo.lock Git-revision coherence gate"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/US-008"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/StR-004"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/FR-007"
    type: "consumes"
---
# FR-027: Cargo.lock Git-revision coherence gate

## Description

The distributed `quire` executable SHALL expose `quire check-lock <Cargo.lock>...`
as the single owned developer gate for agent-ix Git dependency coherence.
The gate checks each lock independently; it does not compare independent locks
with one another. IR-736 delivers this command; IR-641 separately adopts its
released executable and removes the copied consumer scripts.

## Inputs

One or more explicit paths to regular Cargo.lock files. The command has no
module argument, glob expansion, stdin mode, or network premise. A positional
`-` SHALL be rejected as invalid input rather than consuming stdin. Input path
safety and I/O classification follow [FR-005](./FR-005-path-safety.md) and
[FR-007](./FR-007-exit-codes.md), including rejection of `..` components.

## Outputs

On complete success, the command SHALL emit one concise deterministic summary
on stdout identifying the checked paths, with empty stderr and exit 0.
On failure, the command SHALL leave stdout empty and emit diagnostics on stderr
through the existing shared CLI diagnostic renderer, honouring global
`--diagnostics-format` and `--color`. This command introduces no JSON result
schema or alternate diagnostic format. `--pretty` has no effect on its text
success summary.

## Behavior

The command SHALL parse each input as TOML using a maintained TOML parser and
read the Cargo `[[package]]` records through a typed model. Missing package
names, incorrectly typed names or sources, malformed TOML, and an incorrectly
typed package collection SHALL be invalid input. A missing package collection
or an empty package array is a valid zero-package lock. A missing source
identifies a workspace/path package and is outside the Git rule.
The command SHALL classify present sources lacking a recognized `registry+`,
`sparse+`, or `git+` source prefix as invalid input. Registry sources and
well-formed non-agent-ix Git repository sources are outside the coherence rule.

For this gate, an agent-ix repository is a Git URL with host `github.com` and
owner path component `agent-ix`, compared case-insensitively. Supported Git
transport URLs are `https`, `http`, `ssh`, and `git`, after removing Cargo's
`git+` source prefix. For an agent-ix repository, the command SHALL require
exactly the owner and a nonempty
repository path component, optionally ending in `.git` or `/`; credentials
and a default transport port do not distinguish the repository. The canonical
repository identity SHALL be `github.com/agent-ix/<repository>` with the
repository name lowercased and a terminal `.git` and slash removed. A
non-default port, empty repository, extra path component, encoded path separator,
or malformed URL that cannot establish a valid excluded repository SHALL be
invalid input, not silently ignored. URL parsing SHALL use a maintained URL
parser rather than substring matching or a hand-written URL lexer.

For an agent-ix Git source, the command SHALL require a resolved fragment of
exactly 40 ASCII hexadecimal digits and normalize those digits to lowercase.
The fragment is the resolved commit. The command SHALL ignore `branch`, `tag`,
and `rev` query selections when computing repository or resolved-commit identity.
Thus changing a query cannot hide a split resolved revision.
The command SHALL classify missing, empty, shortened, or non-hex agent-ix
fragments as invalid input.

Within each lock, the command SHALL reject more than one package record with
the same agent-ix Git package name, including identical revisions and records
from different agent-ix repositories. It SHALL also reject more than one
resolved revision for one canonical agent-ix repository, even when all affected
package names differ. Aligned multi-crate repositories SHALL pass. Repeated
registry or non-agent-ix package names SHALL not trigger this policy.

The command SHALL classify coherence violations as domain-policy refusals,
exiting 2 under [FR-007](./FR-007-exit-codes.md).
The command SHALL exit 3 for malformed input, absent paths, directories, or
missing arguments.
The command SHALL exit 2 for input permission refusal.
The command SHALL exit 4 for unclassified I/O, serialization, or output failure.
The command SHALL use typed conditions to classify these outcomes, independently
of wording.

The command SHALL inspect all supplied locks before declaring success. On
well-formed inputs, its diagnostic SHALL identify every failing lock, canonical
repository, affected package name, and resolved revision.
The command SHALL order diagnostics by supplied lock path, repository, package
name, and revision, using lexicographic ordering within each group.
Fatal input failures may stop the invocation at the first supplied failing path. The command SHALL identify that path in its diagnostic.
The command SHALL NOT present a fatal input failure as a clean coherence result. Consequently, one failing lock makes the whole invocation fail, while different revisions in two
individually coherent locks do not.

The command SHALL read files without modifying them, fetching sources, invoking
Cargo or Git, spawning shell checkers, or consulting an external revision
catalog. Resolved-commit comparisons are derived solely from the supplied lock.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-027-CON-1 | Coherence logic SHALL have one Rust owner in quire-cli; consumer Makefiles invoke the released binary without copied executable checker logic. | Ownership | Inspection |
| FR-027-CON-2 | The command SHALL remain a developer gate without introducing runtime dependency edges into IR, CG, or RT. | Architecture | Inspection |
| FR-027-CON-3 | Local qualification SHALL NOT publish a release or claim remote-host installation from a local receipt. | Evidence | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-027-AC-1 | A lock with two or more distinct package names from one agent-ix repository at the same resolved commit passes; empty and registry/path-only locks also pass. | Test |
| FR-027-AC-2 | Distinct package names from one canonical repository at different resolved commits fail with exit 2 and diagnostics naming the lock, repository, all affected names, and revisions. | Test |
| FR-027-AC-3 | Duplicate agent-ix Git package names fail even with identical sources or different repositories; legitimate repeated registry and non-agent-ix package names pass. | Test |
| FR-027-AC-4 | Branch/tag/rev/no-query and terminal .git/slash variants normalize to one repository: aligned fragments pass and split fragments fail, including GitHub case and transport variations. | Test |
| FR-027-AC-5 | Missing, empty, shortened, and non-hex agent-ix resolved fragments, malformed URLs, invalid package field types, and malformed TOML fail as invalid input rather than clean results. | Test |
| FR-027-AC-6 | Multiple independently aligned locks may use different revisions; any failing lock fails the invocation and its diagnostic names that lock. A missing or unreadable path cannot yield a successful report. | Test |
| FR-027-AC-7 | Success and failure streams, global diagnostic formatting, missing arguments, stdin rejection, path traversal, and typed I/O outcomes conform to the existing CLI contract; repeated invocations produce identical observations. | Test |
| FR-027-AC-8 | A split-revision fixture passes the original consumer nine-line AWK checker and fails this command. Qualification executes the existing original checker externally without adding an executable copy to quire-cli. | Demonstration |
| FR-027-AC-9 | Execution leaves all inputs unchanged and performs no network, Cargo, Git, or shell invocation. | Test |
| FR-027-AC-10 | The normal quire-cli release/install route supplies this same command; installation and representative aligned/split invocation are qualified separately on each IR/CG/RT gate host, with unavailable hosts explicitly unqualified. | Demonstration |

## Qualification Procedure

This specification precedes implementation. Its criteria are not qualified by
this document. Once implemented, run the repository's ordinary local gates and
command integration tests with actual `#[test]` criterion bindings. Retain a
split-revision fixture as data, then execute both the existing original
IR/CG/RT `scripts/check_one_copy.awk` and the new native command against that
same input. Record old exit 0 and new exit 2; do not copy the AWK program into
this repository or make the released command depend on it.

Use the existing release-binary or npm installation route documented in the
README. Record the actual host, installation route, executable invocation, and
aligned/split verdicts on each gate host. A locally built candidate receipt is
not an installed release receipt, and one host's receipt does not qualify
another. Release publication remains the existing explicit release operation;
this specification neither changes a workflow nor claims publication.

## Dependencies

- [US-008](../usecase/US-008-check-git-lock-coherence.md) states the developer need.
- [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) confines the developer-gate exception.
- [FR-006](./FR-006-io-contract.md), [FR-007](./FR-007-exit-codes.md), and
  [NFR-005](../non-functional/NFR-005-diagnostic-format.md) govern process I/O.
- The normal release and installation paths are described in the repository
  [README](../../README.md#install); this command requires no new installer.
- IR-641 owns later consumer Makefile adoption, release selection, copied-script
  removal, and consumer gate receipts. No adoption or remote-host receipt is
  asserted by this specification.
