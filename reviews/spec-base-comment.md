<!-- reviewer repo=agent-ix/quire-cli visibility=public quoin=0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-967175026302197768900474181935084795 method=spec-review/base lang=spec pr=quire-cli#247 model=gpt-6.2-sol run=b3819e72-43a8-46b1-bf20-e61067ef23ba -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | coverage | AC evidence uses a fixture-only entry path, so it does not prove the active module binds selftest markers |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-047-AC-3, path: "spec/functional/FR-047-quire-selftest.md", role: examined, excerpt: "A quire-cli selftest case with a valid shell trace marker binds its named criterion."}
  - {id: selftest_shell_marker_binds_and_removal_names_the_case_script, path: "tests/cli_matrix.rs", role: examined, excerpt: "The fixture adds selftest/cases/**/*.sh as a marker entry path and exercises valid, missing, and invalid criterion markers."}
findings:
  - fnd: FND-001
    severity: medium
    confidence: high
    method: spec-review/base
    check: coverage
    artifact_id: FR-047-AC-3
    related_ids: []
    path: tests/cli_matrix.rs
    lines: "82-91"
    excerpt: "A quire-cli selftest case with a valid shell trace marker binds its named criterion."
    finding: "The criterion is observable and verifiable, but the only new test supplies the needed selftest entry path in its own module fixture. The active module does not yet include selftest paths, so the evidence does not establish the stated binding behavior against the production configuration."
```
+++

