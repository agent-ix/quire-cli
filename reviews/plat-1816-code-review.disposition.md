<!-- reviewer-dispositions repo=agent-ix/quire-cli visibility=public quoin=quoin 0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-20261116045032 round=1 method=code-review model=gpt-6.2-sol run=1d79bf95-e6a6-4217-9d38-605d8156a751 -->

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Direct trace binding added. |
| FND-002 | fixed | Direct trace binding added. |
| FND-003 | new finding | The host variation is not exercised by the equality oracle. |

+++ [reviewer data]

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: '    "FR-034-AC-19",'
- fnd: FND-002
  outcome: fixed
  after_excerpt: '    "FR-037-AC-6",'
new_findings:
- fnd: FND-003
  severity: medium
  confidence: high
  method: code-review
  check: test-oracle
  artifact_id: FR-034-AC-19
  related_ids: []
  path: tests/cli_export.rs
  lines: 3838-3840
  excerpt: Two exports of the same committed source with the same template, Quire build, pandoc, typst, fonts, and inputs yield byte-identical output files, including when the source frontmatter contains `producer_identity`. Changing only the wall clock, time zone, host, working directory, capability-root path, or output path does not change output bytes.
  finding: The byte-equality test runs both exports on the same host. It varies clock, time zone, working/output/capability paths, but does not vary host identity; host-dependent bytes could therefore match in both outputs and pass the assertion.
scope:
- id: FR-034-AC-19
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: Two exports of the same committed source with the same template, Quire build, pandoc, typst, fonts, and inputs yield byte-identical output files, including when the source frontmatter contains `producer_identity`. Changing only the wall clock, time zone, host, working directory, capability-root path, or output path does not change output bytes.
- id: FR-037-AC-6
  path: spec/functional/FR-037-export-clock-free-output.md
  role: examined
  excerpt: With a producer identity present in committed input frontmatter, both PDF and DOCX title pages and metadata carry its exact `name` and `version`; changing the export process clock and time zone does not change the source commit Date, producer identity or any output byte.
- id: tests/cli_export::real_exports_give_equal_text_and_structure_across_directories_and_zones
  path: tests/cli_export.rs
  role: examined
  excerpt: "#[trace(\n    \"FR-034-AC-19\",\n    \"FR-037-AC-6\",\n    \"NFR-011-AC-1\",\n    \"NFR-011-AC-3\",\n    \"NFR-011-AC-4\"\n)]"
bindings:
- test_id: real_exports_give_equal_text_and_structure_across_directories_and_zones
  ac_id: FR-034-AC-19
  trace: correct
- test_id: real_exports_give_equal_text_and_structure_across_directories_and_zones
  ac_id: FR-037-AC-6
  trace: correct
- test_id: real_exports_give_equal_text_and_structure_across_directories_and_zones
  ac_id: NFR-011-AC-1
  trace: correct
- test_id: real_exports_give_equal_text_and_structure_across_directories_and_zones
  ac_id: NFR-011-AC-3
  trace: correct
```

+++