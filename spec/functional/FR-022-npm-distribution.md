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

The additive tag-attestation workflow also supplies this repository's retained
release-test evidence to the shared Engineering Assurance workflow. Quoin owns
the evidence-store format, run-record validation, and Test Result export; this
requirement owns the release-suite declaration for `quire-cli` and the workflow
handoff of that store.

Supported platforms are `linux-x64`, `linux-arm64`, `darwin-arm64`, and
`win32-x64`. The Linux binaries are fully static musl builds, so their packages
carry no libc restriction and install on glibc and musl hosts alike.

## Behavior

- The repository SHALL maintain `spec/evidence/suites.md` with one declared
  release-verification suite, `SUITE-001`, whose command is `make evidence`
  and whose test run is `cargo nextest run --workspace --locked --all-features
  --profile evidence`.
- The repository SHALL keep its `spec/evidence/suites.md` declaration and
  `spec/evidence/bindings.json` in source control as the registry for this
  suite.
- The `SUITE-001` run SHALL record the test outcomes and trace bindings in a
  Quoin evidence root under `spec/evidence`; the run record is generated for
  the checked-out tag and is not committed as historical state.
- When a tag push matches `v*`, the tag-attestation build job SHALL create that evidence
  root for the checked-out tag and upload the complete root as the
  `release-evidence` workflow artifact.
- The tag-attestation job that has `id-token: write` SHALL execute no
  repository code and SHALL pass the `release-evidence` artifact to the
  Engineering Assurance reusable workflow as the `evidence_artifact` input.
- If the evidence root is absent or invalid, or the run is failed or stale, the
  workflow SHALL write no release attestation.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-022-AC-1 | `npm install -g @agent-ix/quire-cli@<version>` installs a `quire` that runs the native binary and reports `<version>` from `--version`, on each supported platform. | Inspection (release workflow post-publish smoke) |
| FR-022-AC-2 | Under an npm install, `quire update` exits non-zero and names `npm install -g @agent-ix/quire-cli@latest` instead of replacing the binary. | Inspection (release workflow post-publish smoke) |
| FR-022-AC-3 | The release workflow publishes only when dispatched with `publish` true, authenticates to public npm by OIDC Trusted Publishing, and passes the four built targets as `artifacts/<rust-target>/quire[.exe]`. | Inspection |
| FR-022-AC-4 | `quire-dist verify-binary` fails unless the built executable's `--version` equals the release version, and the release workflow runs it for each target before packaging. | Test |
| FR-022-AC-5 | A workflow-parsing integration test confirms `.github/workflows/release.yml` remains `workflow_dispatch`-only and both GitHub Release creation and npm publication remain gated by `inputs.publish`; the additive `.github/workflows/tag-attestation.yml` runs on `v*` tag pushes and invokes the Engineering Assurance reusable attestation workflow independently of the manual release workflow. | Test |
| FR-022-AC-6 | `spec/evidence/suites.md` declares `SUITE-001` as `make evidence`, running `cargo nextest run --workspace --locked --all-features --profile evidence`; Quoin records the run outcomes and trace bindings in `spec/evidence/bindings.json` and `spec/evidence/runs/` for the checked-out tag. | Test |
| FR-022-AC-7 | When a tag push matches `v*`, the tag-attestation build job creates the `SUITE-001` evidence root for the checked-out tag and uploads it as `release-evidence`; the id-token-bearing attestation job passes that artifact as `evidence_artifact` to the Engineering Assurance reusable workflow and contains no checkout or repository-code execution. | Test |
| FR-022-AC-8 | An absent or invalid evidence root, or a failed or stale `SUITE-001` run, results in a failed tag-attestation workflow with no release attestation written; the run is never exported as a passing Test Result. | Test |

> **EA-122 scope note:** The repository-specific release-suite declaration and
> artifact handoff are part of this distribution requirement. Quoin continues
> to own the store schema, recording semantics, and export behavior. The
> workflow does not accept a caller-built Test Result. The suite declaration
> and binding registry are committed; each tag run record is retained only in
> its `release-evidence` artifact and is not committed as historical state.

## Dependencies

- **Upstream**: ADR-0003; the shared `publish-native-npm` action.
