---
id: FR-022
title: "npm distribution through the shared publish action"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

# FR-022: npm distribution through the shared publish action

## Description

The `@agent-ix/quire-cli` npm package SHALL install the native `quire` binary on
every supported platform. The release workflow SHALL publish it through the
shared `agent-ix/nodejs-actions/publish-native-npm` action (ADR-0003), which
owns package layout, the launcher, and the `npm publish` calls. This repository
contains no npm packaging or launcher implementation.

Supported platforms are `linux-x64`, `linux-arm64`, `darwin-arm64`, and
`win32-x64`. The Linux binaries are fully static musl builds, so their packages
carry no libc restriction and install on glibc and musl hosts alike.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-022-AC-1 | `npm install -g @agent-ix/quire-cli@<version>` installs a `quire` that runs the native binary and reports `<version>` from `--version`, on each supported platform. | Inspection (release workflow post-publish smoke) |
| FR-022-AC-2 | Under an npm install, `quire update` exits non-zero and names `npm install -g @agent-ix/quire-cli@latest` instead of replacing the binary. | Inspection (release workflow post-publish smoke) |
| FR-022-AC-3 | The release workflow publishes only when dispatched with `publish` true, authenticates to public npm by OIDC Trusted Publishing, and passes the four built targets as `artifacts/<rust-target>/quire[.exe]`. | Inspection |
| FR-022-AC-4 | `quire-dist verify-binary` fails unless the built executable's `--version` equals the release version, and the release workflow runs it for each target before packaging. | Test |

## Dependencies

- **Upstream**: ADR-0003; the shared `publish-native-npm` action.
