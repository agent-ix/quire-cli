---
id: ADR-0002
title: "Retain a minimal Node host for the cross-platform npm launcher"
type: ADR
---

# ADR 0002: Retain a minimal Node host for the cross-platform npm launcher

**Status**: proposed — owner disposition required before implementation
**Date**: 2026-09-09
**Decision authority**: repository owner

## Context

The public npm surface is one cross-platform meta-package,
`@agent-ix/quire-cli`, backed by four optional native packages. An npm `bin`
entry names an executable file inside the meta-package; npm does not select a
different `bin` path by operating system and architecture. Some host code must
therefore resolve the installed optional package before the native Rust process
can start.

The current Node file also duplicates a target allowlist. Separate MJS, shell,
Perl, embedded Node, and workflow fragments own generation and validation.
Those are avoidable first-party logic and violate the required Rust containment.

## Proposed Decision

Retain exactly one Node production file, `npm/quire-cli/bin/quire.js`, solely as
the npm executable host. Its complete permitted behavior is:

1. read the host platform and architecture;
2. derive the platform package name and verify that it occurs in the generated
   launcher's own `optionalDependencies`;
3. resolve that package's fixed `bin/quire` or `bin/quire.exe` member;
4. make one best-effort POSIX chmod call;
5. spawn the native executable with unchanged arguments and inherited standard
   streams;
6. mirror its normal exit status or terminating signal; and
7. emit deterministic unsupported-host, missing-package, or launch-failure
   diagnostics.

The file may not own a second target catalog, version policy, package
generation, binary validation, checksums, test assertions, document handling,
extraction, clause grammar, profile rules, or source semantics. One Rust target
catalog generates its optional dependencies, packages, and supported-set data.

The production npm path therefore continues to require Node, as npm itself
does. Qualification invokes exact Node/npm only as external hosts for local
pack, offline install, resolution, and launcher probes; `ix-trace-rs`-marked
Rust tests make every assertion. No hosted CI or publication is part of that
qualification.

## Consequences

- Existing `npm install -g @agent-ix/quire-cli` usability and all four supported
  targets remain intact.
- Consumers of the npm channel retain a Node runtime dependency; Cargo and
  release-binary consumers do not.
- The small launcher seam is behaviorally qualified, while package/version/
  validation logic moves to Rust and cannot drift into the shim.
- Any future expansion of the shim requires a new owner disposition and spec
  review.

## Alternative requiring owner choice

Remove the cross-platform meta-package and require consumers to install a
platform-specific package such as `@agent-ix/quire-cli-linux-x64` directly.
This eliminates the Node launcher but changes the public package name and makes
users or higher-level installers select the target. No compatibility migration
is required, but the installation experience is materially worse.

## Alternatives rejected by this proposal

- **npm lifecycle/postinstall script copies the binary**: still requires Node,
  adds install-time filesystem mutation, and is less transparent than a small
  launch-time host.
- **Duplicate launchers in every native package**: npm still cannot select one
  package from a shared cross-platform name, and the executable host would be
  copied four times.
- **Rust wrapper inside the meta-package**: the wrapper itself would need one
  preselected native target, which is the selection problem the optional
  packages solve.
