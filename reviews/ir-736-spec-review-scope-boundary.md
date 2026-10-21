---
id: SR-14408
title: spec-review/scope-boundary review of IR-736 check-lock contract
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-cli; IR-736; spec/functional/FR-027-check-lock-subcommand.md,
  US-008, StR-004 and index/master edits
review_set: subset
---

## Summary

Ticket: IR-736. Frozen head: `92475c7142d318352faecf3b87357735dd88247b`.

Owner: distributed quire-cli owns typed Cargo.lock intake, normalized agent-ix policy and existing shared diagnostic/output wiring. Existing engine remains owner of Markdown artifact behavior. Private quire-qualify remains repository-local. Maintained TOML/URL parsers are library dependencies; their contract is exercised by the planned adverse corpus. IR/CG/RT invoke an installed release later through IR-641. No network fetch, caller runtime edge, engine duplication, compatibility path or copied original checker is permitted. Installed Titan 0.36.2 exists but lacks this new command; no other host or new release is qualified.

## Examined Scope

```yaml
- id: FR-027-CON-1
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Coherence logic SHALL have one Rust owner in quire-cli; consumer Makefiles
    invoke the released binary without copied executable checker logic.
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
- id: FR-027-AC-1
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: A lock with two or more distinct package names from one agent-ix repository
    at the same resolved commit passes; empty and registry/path-only locks also pass.
- id: FR-027-AC-2
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Distinct package names from one canonical repository at different resolved
    commits fail with exit 2 and diagnostics naming the lock, repository, all affected
    names, and revisions.
- id: FR-027-AC-3
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Duplicate agent-ix Git package names fail even with identical sources or
    different repositories; legitimate repeated registry and non-agent-ix package
    names pass.
- id: FR-027-AC-4
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'Branch/tag/rev/no-query and terminal .git/slash variants normalize to
    one repository: aligned fragments pass and split fragments fail, including GitHub
    case and transport variations.'
- id: FR-027-AC-5
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Missing, empty, shortened, and non-hex agent-ix resolved fragments, malformed
    URLs, invalid package field types, and malformed TOML fail as invalid input rather
    than clean results.
- id: FR-027-AC-6
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Multiple independently aligned locks may use different revisions; any failing
    lock fails the invocation and its diagnostic names that lock. A missing or unreadable
    path cannot yield a successful report.
- id: FR-027-AC-7
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Success and failure streams, global diagnostic formatting, missing arguments,
    stdin rejection, path traversal, and typed I/O outcomes conform to the existing
    CLI contract; repeated invocations produce identical observations.
- id: FR-027-AC-8
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: A split-revision fixture passes the original consumer nine-line AWK checker
    and fails this command. Qualification executes the existing original checker externally
    without adding an executable copy to quire-cli.
- id: FR-027-AC-9
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Execution leaves all inputs unchanged and performs no network, Cargo, Git,
    or shell invocation.
- id: FR-027-AC-10
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: The normal quire-cli release/install route supplies this same command;
    installation and representative aligned/split invocation are qualified separately
    on each IR/CG/RT gate host, with unavailable hosts explicitly unqualified.
- id: FR-027:paragraph-3:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'The distributed `quire` executable SHALL expose `quire check-lock <Cargo.lock>...`

    as the single owned developer gate for agent-ix Git dependency coherence.

    The gate checks each lock independently; it does not compare independent locks

    with one another. IR-736 delivers this command; IR-641 separately adopts its

    released executable and removes the copied consumer scripts.'
- id: FR-027:paragraph-5:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'One or more explicit paths to regular Cargo.lock files. The command has
    no

    module argument, glob expansion, stdin mode, or network premise. A positional

    `-` SHALL be rejected as invalid input rather than consuming stdin. Input path

    safety and I/O classification follow [FR-005](./FR-005-path-safety.md) and

    [FR-007](./FR-007-exit-codes.md), including rejection of `..` components.'
- id: FR-027:paragraph-7:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'On complete success, the command SHALL emit one concise deterministic
    summary

    on stdout identifying the checked paths, with empty stderr and exit 0.

    On failure, the command SHALL leave stdout empty and emit diagnostics on stderr

    through the existing shared CLI diagnostic renderer, honouring global

    `--diagnostics-format` and `--color`. This command introduces no JSON result

    schema or alternate diagnostic format. `--pretty` has no effect on its text

    success summary.'
- id: FR-027:paragraph-9:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'The command SHALL parse each input as TOML using a maintained TOML parser
    and

    read the Cargo `[[package]]` records through a typed model. Missing package

    names, incorrectly typed names or sources, malformed TOML, and an incorrectly

    typed package collection SHALL be invalid input. A missing package collection

    or an empty package array is a valid zero-package lock. A missing source

    identifies a workspace/path package and is outside the Git rule.

    The command SHALL classify present sources lacking a recognized `registry+`,

    `sparse+`, or `git+` sour'
- id: FR-027:paragraph-9:part-550
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'ce prefix as invalid input. Registry sources and

    well-formed non-agent-ix Git repository sources are outside the coherence rule.'
- id: FR-027:paragraph-10:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'For this gate, an agent-ix repository is a Git URL with host `github.com`
    and

    owner path component `agent-ix`, compared case-insensitively. Supported Git

    transport URLs are `https`, `http`, `ssh`, and `git`, after removing Cargo''s

    `git+` source prefix. For an agent-ix repository, the command SHALL require

    exactly the owner and a nonempty

    repository path component, optionally ending in `.git` or `/`; credentials

    and a default transport port do not distinguish the repository. The canonical

    repository identity SHALL be `github.com/agent-ix/<reposi'
- id: FR-027:paragraph-10:part-550
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'tory>` with the

    repository name lowercased and a terminal `.git` and slash removed. A

    non-default port, empty repository, extra path component, encoded path separator,

    or malformed URL that cannot establish a valid excluded repository SHALL be

    invalid input, not silently ignored. URL parsing SHALL use a maintained URL

    parser rather than substring matching or a hand-written URL lexer.'
- id: FR-027:paragraph-11:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'For an agent-ix Git source, the command SHALL require a resolved fragment
    of

    exactly 40 ASCII hexadecimal digits and normalize those digits to lowercase.

    The fragment is the resolved commit. The command SHALL ignore `branch`, `tag`,

    and `rev` query selections when computing repository or resolved-commit identity.

    Thus changing a query cannot hide a split resolved revision.

    The command SHALL classify missing, empty, shortened, or non-hex agent-ix

    fragments as invalid input.'
- id: FR-027:paragraph-12:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'Within each lock, the command SHALL reject more than one package record
    with

    the same agent-ix Git package name, including identical revisions and records

    from different agent-ix repositories. It SHALL also reject more than one

    resolved revision for one canonical agent-ix repository, even when all affected

    package names differ. Aligned multi-crate repositories SHALL pass. Repeated

    registry or non-agent-ix package names SHALL not trigger this policy.'
- id: FR-027:paragraph-13:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'The command SHALL classify coherence violations as domain-policy refusals,

    exiting 2 under [FR-007](./FR-007-exit-codes.md).

    The command SHALL exit 3 for malformed input, absent paths, directories, or

    missing arguments.

    The command SHALL exit 2 for input permission refusal.

    The command SHALL exit 4 for unclassified I/O, serialization, or output failure.

    The command SHALL use typed conditions to classify these outcomes, independently

    of wording.'
- id: FR-027:paragraph-14:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'The command SHALL inspect all supplied locks before declaring success.
    On

    well-formed inputs, its diagnostic SHALL identify every failing lock, canonical

    repository, affected package name, and resolved revision.

    The command SHALL order diagnostics by supplied lock path, repository, package

    name, and revision, using lexicographic ordering within each group.

    Fatal input failures may stop the invocation at the first supplied failing path.
    The command SHALL identify that path in its diagnostic.

    The command SHALL NOT present a fatal input failure as'
- id: FR-027:paragraph-14:part-550
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: ' a clean coherence result. Consequently, one failing lock makes the whole
    invocation fail, while different revisions in two

    individually coherent locks do not.'
- id: FR-027:paragraph-15:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'The command SHALL read files without modifying them, fetching sources,
    invoking

    Cargo or Git, spawning shell checkers, or consulting an external revision

    catalog. Resolved-commit comparisons are derived solely from the supplied lock.'
- id: FR-027:paragraph-21:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'This specification precedes implementation. Its criteria are not qualified
    by

    this document. Once implemented, run the repository''s ordinary local gates and

    command integration tests with actual `#[test]` criterion bindings. Retain a

    split-revision fixture as data, then execute both the existing original

    IR/CG/RT `scripts/check_one_copy.awk` and the new native command against that

    same input. Record old exit 0 and new exit 2; do not copy the AWK program into

    this repository or make the released command depend on it.'
- id: FR-027:paragraph-22:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'Use the existing release-binary or npm installation route documented in
    the

    README. Record the actual host, installation route, executable invocation, and

    aligned/split verdicts on each gate host. A locally built candidate receipt is

    not an installed release receipt, and one host''s receipt does not qualify

    another. Release publication remains the existing explicit release operation;

    this specification neither changes a workflow nor claims publication.'
- id: FR-027:paragraph-24:part-0
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: "- [US-008](../usecase/US-008-check-git-lock-coherence.md) states the developer\
    \ need.\n- [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) confines\
    \ the developer-gate exception.\n- [FR-006](./FR-006-io-contract.md), [FR-007](./FR-007-exit-codes.md),\
    \ and\n  [NFR-005](../non-functional/NFR-005-diagnostic-format.md) govern process\
    \ I/O.\n- The normal release and installation paths are described in the repository\n\
    \  [README](../../README.md#install); this command requires no new installer.\n\
    - IR-641 owns later consumer Makefile adoption, rel"
- id: FR-027:paragraph-24:part-550
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: "ease selection, copied-script\n  removal, and consumer gate receipts.\
    \ No adoption or remote-host receipt is\n  asserted by this specification."
- id: US-008:paragraph-3:part-0
  path: spec/usecase/US-008-check-git-lock-coherence.md
  role: examined
  excerpt: '**As a** developer maintaining related Rust packages

    **I want** dependency gates to detect inconsistent resolutions from the same

    Git repository

    **So that** different crate names cannot hide a split dependency revision.'
- id: US-008:paragraph-5:part-0
  path: spec/usecase/US-008-check-git-lock-coherence.md
  role: examined
  excerpt: 'IR, CG, and RT currently copy a checker that detects repeated agent-ix
    crate

    names but accepts distinct crate names from one repository at different

    resolved commits. One distributed developer command avoids three diverging

    executable implementations. Different independently checked lockfiles can

    legitimately select different revisions.'
- id: US-008:paragraph-7:part-0
  path: spec/usecase/US-008-check-git-lock-coherence.md
  role: examined
  excerpt: '[FR-027](../functional/FR-027-check-lock-subcommand.md) defines the command.

    IR-641 owns subsequent adoption and removal of the copied consumer scripts.'
- id: StR-004-VC-1
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: '**StR-004-AC-1**: Each FR in `quire-cli/spec/functional/` that wraps a
    `quire-rs` API declares the upstream `quire-rs` FR ID in its frontmatter `relationships:`
    array (`type: implements`).'
- id: StR-004-VC-2
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: '**StR-004-AC-2**: `src/` contains no markdown parsing, template rendering,
    graph construction, assurance projection, or JSON Schema definition/validation
    logic — only argv/premise parsing, path-safety checks, stdin/stdout wiring, calls
    into `quire-rs`, and the explicitly owned FR-027 Cargo.lock developer gate.'
- id: StR-004-VC-3
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: '**StR-004-AC-3**: Code review checklist for this repo includes: "Does
    any new logic belong upstream in `quire-rs`?".'
- id: StR-004:paragraph-3:part-0
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: '`quire-rs` is the canonical engine. Any Markdown parser/renderer/validator
    behavior in this CLI that diverges from `quire-rs` creates the exact drift problem
    `quire-rs` was built to solve.'
- id: StR-004:paragraph-4:part-0
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: 'For artifact operations, the CLI SHALL be a **thin process boundary**
    over `quire-rs`. Every observable

    artifact behavior in `parse`/`extract`/`validate`/`assurance` SHALL trace to a

    `quire-rs` FR. If a feature request cannot be satisfied by adding flags or

    composing existing `quire-rs` APIs, the upstream FR is authored first and this

    CLI is updated to expose the new surface.'
- id: StR-004:paragraph-5:part-0
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: 'The distributed developer gate [FR-027](../functional/FR-027-check-lock-subcommand.md)

    is a narrow exception: quire-cli owns Cargo.lock TOML intake and agent-ix Git

    repository coherence, which are not Markdown artifact or engine semantics.

    This exception authorizes no document parser, engine model, graph, assurance,

    source-analysis, or general qualification-tool duplication. The existing

    private `quire-qualify` package remains repository-local under

    [FR-025](../functional/FR-025-rust-qualification-tooling.md).'
- id: StR-004:paragraph-7:part-0
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: Must-Have
- id: StR-004:paragraph-9:part-0
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: '`quire-rs` is the canonical engine; any parser/renderer/validator behavior

    duplicated in the CLI recreates the exact drift problem the engine was built to

    eliminate. Keeping the CLI a thin process boundary — argv parsing, path-safety,

    stream wiring, and calls into `quire-rs` — guarantees observable artifact behavior
    traces to

    an upstream FR and that new features are authored upstream first rather than

    forked into the CLI.'
- id: StR-004:paragraph-11:part-0
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: 'This need is considered satisfied when every CLI FR that wraps an engine
    API

    declares the upstream `quire-rs` FR in its relationships, the `src/` tree carries

    no parsing/rendering/validation, graph, or output-schema logic of its own, and
    the review checklist

    enforces the thin-boundary stance:'
- id: spec/spec.md
  path: spec/spec.md
  role: examined
  excerpt: '- The owned, offline Cargo.lock developer gate [FR-027](./functional/FR-027-check-lock-subcommand.md),
    distributed as `quire check-lock <Cargo.lock>...`; consumer adoption and copied-script
    removal remain IR-641.'
- id: spec/spec.md
  path: spec/spec.md
  role: examined
  excerpt: '- Artifact operations remain a **thin process boundary**: substantive
    Markdown parse/render/extract/validate logic belongs in `quire-rs`. The narrow
    developer-gate exception is [FR-027](./functional/FR-027-check-lock-subcommand.md),
    as scoped by [StR-004](./stakeholder/StR-004-thin-boundary-over-quire-rs.md);
    it owns no engine artifact behavior.'
- id: spec/functional/index.md
  path: spec/functional/index.md
  role: examined
  excerpt: '* [FR-027: Cargo.lock Git-revision coherence gate](./FR-027-check-lock-subcommand.md)'
- id: spec/usecase/index.md
  path: spec/usecase/index.md
  role: examined
  excerpt: '* [US-008: Developer checks Git lock coherence](./US-008-check-git-lock-coherence.md)'
- id: FR-005
  path: spec/functional/FR-005-path-safety.md
  role: context_only
  excerpt: 'Before any I/O, the CLI SHALL canonicalize and validate every user-supplied

    filesystem path argument, rejecting `..` segments and symlink escapes from the

    `--module` root, so that a malicious or malformed path argument cannot escape
    the

    intended module root. The behavioral surface is specified below.'
- id: FR-006
  path: spec/functional/FR-006-io-contract.md
  role: context_only
  excerpt: 'Every subcommand SHALL adhere to a uniform stdin/stdout/stderr contract:
    primary

    result on stdout, all diagnostics on stderr (never interleaved), and stdin used

    only for a positional `-`. This guarantees clean pipelines and clean diagnostic

    suppression. The behavioral surface is specified below.'
- id: FR-007
  path: spec/functional/FR-007-exit-codes.md
  role: context_only
  excerpt: 'The CLI SHALL use ix-cli-kit outcomes uniformly across subcommands, so
    callers

    can branch on typed outcomes without parsing diagnostics.'
- id: NFR-005
  path: spec/non-functional/NFR-005-diagnostic-format.md
  role: context_only
  excerpt: 'Every diagnostic the CLI emits to stderr — including CLI-originated diagnostics

    (`PathSafetyViolation`, `UnknownArchetype` lookup failure prior to engine call)
    —

    SHALL be expressible as a `quire-rs::Diagnostic`. The CLI SHALL NOT invent a

    parallel diagnostic format: human-readable rendering uses

    `quire-rs::format_violation`; JSON rendering uses `serde_json::to_string` against

    the `Diagnostic` type, gated by `--diagnostics-format=json`.'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. No native build, cargo, metadata, fmt or Kani execution was performed. This review does not qualify the not-yet-implemented command.
