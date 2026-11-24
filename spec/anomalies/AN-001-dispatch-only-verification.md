---
id: AN-001
title: "CI and release verification are dispatch-only"
type: Anomaly
status: identified
safety_impact: unassessed
---
# [AN-001] CI and release verification are dispatch-only

## Description

The 2026-11-23 regulated-standards review found that repository verification
workflows are dispatch-only and do not provide continuously executed evidence
for changes on `main`. In this repository, `.github/workflows/ci.yml` is
triggered only by `workflow_dispatch`; the release workflow is also
dispatch-only, as specified by FR-022-AC-5. The review reports that no CI run
had run on `main` since 2026-11-15 (lane-D.md, “Testing adequacy”). This record
does not claim a failure of a dispatched workflow or assign a safety impact or
severity; those require a person's assessment.
