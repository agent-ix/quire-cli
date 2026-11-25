<!-- reviewer-dispositions repo=agent-ix/quire-cli visibility=public quoin=quoin 0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-20261116045033 round=1 method=gap-analysis model=gpt-6.2-sol run=1d79bf95-e6a6-4217-9d38-605d8156a751 -->

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Direct trace binding added; matrix reports `tagged`. |
| FND-002 | fixed | Direct trace binding added; matrix reports `tagged`. |
| FND-003 | fixed | The active focused test is not ignored; matrix reports `tagged`. |
| FND-004 | fixed | The active focused test binds AC-3 and is not ignored; matrix reports `tagged`. |

+++ [reviewer data]

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: '    "FR-034-AC-19",'
- fnd: FND-002
  outcome: fixed
  after_excerpt: '    "FR-037-AC-6",'
- fnd: FND-003
  outcome: fixed
  after_excerpt: "#[trace(\n    \"FR-034-AC-19\",\n    \"FR-037-AC-6\",\n    \"NFR-011-AC-1\",\n    \"NFR-011-AC-3\",\n    \"NFR-011-AC-4\"\n)]"
- fnd: FND-004
  outcome: fixed
  after_excerpt: "#[trace(\n    \"FR-034-AC-19\",\n    \"FR-037-AC-6\",\n    \"NFR-011-AC-1\",\n    \"NFR-011-AC-3\",\n    \"NFR-011-AC-4\"\n)]"
scope:
- id: FR-034-AC-19
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: Two exports of the same committed source with the same template, Quire build, pandoc, typst, fonts, and inputs yield byte-identical output files, including when the source frontmatter contains `producer_identity`. Changing only the wall clock, time zone, host, working directory, capability-root path, or output path does not change output bytes.
- id: FR-037-AC-6
  path: spec/functional/FR-037-export-clock-free-output.md
  role: examined
  excerpt: With a producer identity present in committed input frontmatter, both PDF and DOCX title pages and metadata carry its exact `name` and `version`; changing the export process clock and time zone does not change the source commit Date, producer identity or any output byte.
- id: NFR-011-AC-1
  path: spec/non-functional/NFR-011-export-reproducibility.md
  role: examined
  excerpt: Exporting one committed fixture twice to each format with the same Quire build, real pandoc and typst, and the same fonts, once with the process clock and `TZ` changed between runs, yields byte-identical files, equal extracted text and equal extracted structure; the test fails naming the missing tool if a converter or extractor is unavailable.
- id: NFR-011-AC-3
  path: spec/non-functional/NFR-011-export-reproducibility.md
  role: examined
  excerpt: Exporting the same fixture from two different working directories and two different output directories yields byte-identical files, equal extracted text and equal extracted structure.
- id: tests/cli_export::real_exports_give_equal_text_and_structure_across_directories_and_zones
  path: tests/cli_export.rs
  role: examined
  excerpt: "#[trace(\n    \"FR-034-AC-19\",\n    \"FR-037-AC-6\",\n    \"NFR-011-AC-1\",\n    \"NFR-011-AC-3\",\n    \"NFR-011-AC-4\"\n)]"
matrix:
  FR-034-AC-19: tagged
  FR-037-AC-6: tagged
  NFR-011-AC-1: tagged
  NFR-011-AC-3: tagged
```

+++