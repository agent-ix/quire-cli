<!-- reviewer repo=agent-ix/quire-cli visibility=public quoin=0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-967175026302197768900474181935084793 method=code-review lang=rust pr=quire-cli#247 model=gpt-6.2-sol run=b3819e72-43a8-46b1-bf20-e61067ef23ba -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | soundness | The inventory parser rejects valid #trace: shell markers accepted by the declared grammar |
| FND-002 | high | coverage | The fixture adds a selftest entry path absent from the active module, hiding that real markers do not bind |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-047-AC-3, path: "spec/functional/FR-047-quire-selftest.md", role: examined, excerpt: "for each corpus member, strict mode checks every selftest/**/run.sh for at least one shell trace marker that names an acceptance criterion in the corpus, and reports a missing or invalid marker with the exact script path. A quire-cli selftest case with a valid shell trace marker binds its named criterion."}
  - {id: check_selftest_trace_markers, path: "src/commands/matrix.rs", role: examined, excerpt: "The strict inventory scans each run.sh and checks lines for a marker prefix before validating ids against matrix criteria."}
  - {id: selftest_shell_marker_binds_and_removal_names_the_case_script, path: "tests/cli_matrix.rs", role: examined, excerpt: "The test supplies a fixture module with a shell marker declaration and selftest/cases/**/*.sh entry path, then checks valid, missing, and unknown markers."}
findings:
  - fnd: FND-001
    severity: medium
    confidence: high
    method: code-review
    check: soundness
    artifact_id: FR-047-AC-3
    related_ids: []
    path: src/commands/matrix.rs
    lines: "312-315"
    excerpt: "for each corpus member, strict mode checks every selftest/**/run.sh for at least one shell trace marker that names an acceptance criterion in the corpus, and reports a missing or invalid marker with the exact script path."
    finding: "The active shell marker grammar is #\\s*trace:\\s*(.+)$, which accepts #trace: FR-047-AC-3, but this inventory recognizes only the exact # trace: prefix. A valid no-space marker is therefore treated as absent and strict mode fails on a valid case. Parse markers using the declared grammar or the shared parser, and add a regression test for an accepted alternate whitespace form."
  - fnd: FND-002
    severity: high
    confidence: high
    method: code-review
    check: coverage
    artifact_id: FR-047-AC-3
    related_ids: []
    path: tests/cli_matrix.rs
    lines: "82-91"
    excerpt: "A quire-cli selftest case with a valid shell trace marker binds its named criterion."
    finding: "The test injects entry_paths selftest/cases/**/*.sh into a synthetic module. The active module declares only scripts/audits/**/*.sh, so the real matrix does not discover selftest markers as binders; a valid case marker does not bind its criterion unless the companion module change lands. The test should exercise the actual module contract, or this PR must remain gated on that module change."
bindings:
  - {test_id: selftest_shell_marker_binds_and_removal_names_the_case_script, ac_id: FR-047-AC-3, trace: correct}
  - {test_id: local_trace_marker_is_retained_in_run_metadata, ac_id: FR-047-AC-3, trace: correct}
```
+++

