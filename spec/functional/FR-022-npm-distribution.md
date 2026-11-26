---
id: FR-022
title: "npm distribution through the shared publish action"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/engineering-assurance/FR-070"
    type: "references"
  - target: "ix://agent-ix/quoin/FR-030"
    type: "references"
  - target: "ix://agent-ix/quoin/FR-185"
    type: "references"
---

# FR-022: npm distribution through the shared publish action

## Description

The `@agent-ix/quire-cli` npm package SHALL install the native `quire` binary on
every supported platform. The release workflow SHALL publish it through the
shared `agent-ix/nodejs-actions/publish-native-npm` action (ADR-0003), which
owns package layout, the launcher, and the `npm publish` calls. This repository
contains no npm packaging or launcher implementation.

Release-test evidence is prepared locally for each release candidate. The
release owner runs the documented full test suite at the candidate revision,
records the suite results, requirement coverage, uncovered requirements, known
defects, and Quire/Quoin versions in a report, then attaches that report to the
release manually. This repository does not automate report attachment or
release attestation.

Supported platforms are `linux-x64`, `linux-arm64`, `darwin-arm64`, and
`win32-x64`. The Linux binaries are fully static musl builds, so their packages
carry no libc restriction and install on glibc and musl hosts alike.

## Behavior

- For each release candidate, the release owner SHALL run `make test` at the
  exact candidate revision and prepare a report containing test results,
  requirement coverage, uncovered requirements, known defects, and the exact
  Quire and Quoin versions used.
- When the release owner prepares a release, the release owner SHALL attach the
  report manually before publishing. The attachment is a human release step,
  not an automated workflow result.
- The tag-attestation workflow SHALL fail closed when retained release
  evidence is unavailable; it SHALL NOT write a release attestation or passing
  Test Result from absent evidence.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-022-AC-1 | `npm install -g @agent-ix/quire-cli@<version>` installs a `quire` that runs the native binary and reports `<version>` from `--version`, on each supported platform. | Inspection (release workflow post-publish smoke) |
| FR-022-AC-2 | Under an npm install, `quire update` exits non-zero and names `npm install -g @agent-ix/quire-cli@latest` instead of replacing the binary. | Inspection (release workflow post-publish smoke) |
| FR-022-AC-3 | The release workflow publishes only when dispatched with `publish` true, authenticates to public npm by OIDC Trusted Publishing, and passes the four built targets as `artifacts/<rust-target>/quire[.exe]`. | Inspection |
| FR-022-AC-4 | `quire-dist verify-binary` fails unless the built executable's `--version` equals the release version, and the release workflow runs it for each target before packaging. | Test |
| FR-022-AC-5 | A workflow-parsing integration test confirms `.github/workflows/release.yml` remains `workflow_dispatch`-only and both GitHub Release creation and npm publication remain gated by `inputs.publish`; the additive `.github/workflows/tag-attestation.yml` runs on `v*` tag pushes and invokes the Engineering Assurance reusable attestation workflow independently of the manual release workflow. | Test |
| FR-022-AC-6 | At the exact release candidate revision, the release owner runs `make test` and prepares a report containing test results, requirement coverage, uncovered requirements, known defects, and the Quire and Quoin versions used. | Demonstration |
| FR-022-AC-7 | The release owner attaches the completed report to the release manually before publishing; no workflow attachment or attestation is claimed. | Inspection |
| FR-022-AC-8 | If retained release evidence is unavailable, the tag-attestation workflow fails closed without writing a release attestation or passing Test Result. | Test |
| FR-022-AC-9 | (WITHDRAWN by owner decision on 2026-11-26; retained for history and never reused.) | Withdrawn |

> The full-suite report and release attachment are handled by the release owner;
> neither step is provided by GitHub Actions in this repository.

## Dependencies

- **Upstream**: ADR-0003; the shared `publish-native-npm` action.
