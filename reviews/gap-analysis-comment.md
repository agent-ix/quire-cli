<!-- reviewer repo=agent-ix/quire-cli visibility=public quoin=0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-967175026302197768900474181935084794 method=gap-analysis lang=rust pr=quire-cli#247 model=gpt-6.2-sol run=b3819e72-43a8-46b1-bf20-e61067ef23ba -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | coverage | The test's synthetic module configuration hides the absent selftest marker entry path in the active module |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-047-AC-3, path: "spec/functional/FR-047-quire-selftest.md", role: examined, excerpt: "A quire-cli selftest case with a valid shell trace marker binds its named criterion."}
  - {id: selftest_shell_marker_binds_and_removal_names_the_case_script, path: "tests/cli_matrix.rs", role: examined, excerpt: "The test fixture adds a shell-trace-comment declaration with selftest/cases/**/*.sh as entry_paths and checks valid, removed, and unknown markers."}
  - {id: shell-trace-comment, path: "active spec-artifacts-process module manifest.yaml", role: examined, excerpt: "The declared marker pattern is #\\s*trace:\\s*(.+)$; its only entry path is scripts/audits/**/*.sh."}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-047-AC-3
    related_ids: []
    path: tests/cli_matrix.rs
    lines: "82-91"
    excerpt: "A quire-cli selftest case with a valid shell trace marker binds its named criterion."
    finding: "The new matrix test only proves behavior after it locally adds a selftest entry path. The active module excludes those scripts, so a valid case marker is not discovered as a binder and the criterion remains unbound. The separate module change must land before this acceptance criterion is satisfied."
bindings:
  - {test_id: selftest_shell_marker_binds_and_removal_names_the_case_script, ac_id: FR-047-AC-3, trace: correct}
```
+++

