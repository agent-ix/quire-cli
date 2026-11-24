---
id: SR-20261116045020
title: "code-review review of PLAT-1817 / quire-cli#235"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli; scripts/real_tool_test.py; spec/evidence/bindings.json; tests/cli_export.rs"
review_set: all
---

## Summary

This review is read-only at head ed76755423189b6334cbad621112bcbff6d6fe8c. No applicable AssuranceProfile was found. PR scope includes Makefile, scripts/real_tool_test.py, spec/evidence/bindings.json, the retained SUITE-004 run, and spec/evidence/suites.md. The 23-test focused locked real-tool run passed; no build was rerun. The separate pre-existing FR-022-AC-6/SUITE-001 release-evidence gap is outside this PR scope.

Verdict: FAIL.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | The new binding registry repeats test-to-criterion mappings already declared by #[trace] attributes in tests/cli_export.rs. The runner validates the duplicate mappings for consistency, but the duplicate remains two sources of truth and can drift. Remove one mapping source and have the evidence lane consume the authoritative mapping. | spec/evidence/bindings.json:1 | implementation-bug-despite-evidence |
