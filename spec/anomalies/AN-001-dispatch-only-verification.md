---
id: AN-001
title: "Main CI is dispatch-only; publication is manual"
type: Anomaly
status: identified
safety_impact: unassessed
---
# [AN-001] Main CI is dispatch-only; publication is manual

## Description

The 2026-11-23 regulated-standards review found that this repository does not
run its CI validation automatically for changes on `main`. The exact workflow
triggers are:

- `.github/workflows/ci.yml` runs only on `workflow_dispatch`; it does not run
  on pushes or pull requests.
- `.github/workflows/release.yml` runs only on `workflow_dispatch`; publication
  is manually started, as specified by FR-022-AC-5.
- `.github/workflows/tag-attestation.yml` runs on pushes of tags matching
  `v*`. This tag-triggered attestation is separate from CI validation of
  changes on `main`.

The review reports that no CI run had run on `main` since 2026-11-15 (lane-D.md,
“Testing adequacy”). This record does not claim a failure of a dispatched or
tag-triggered workflow or assign a safety impact or severity; those require a
person's assessment.
