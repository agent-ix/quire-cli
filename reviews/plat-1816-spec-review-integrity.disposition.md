<!-- reviewer-dispositions repo=agent-ix/quire-cli visibility=public quoin=quoin 0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-20261116045036 round=1 method=spec-review/integrity model=gpt-6.2-sol run=1d79bf95-e6a6-4217-9d38-605d8156a751 -->

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The broad input toolchain-provenance promise was narrowed to optional declared producer identity. |

+++ [reviewer data]

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: '`quire export` of a Markdown input SHALL assemble the six document identity fields below and SHALL hand them to the converter as data and as generated title-page text ([FR-035](./FR-035-default-export-template.md), [FR-036](./FR-036-organisation-export-template.md)). It SHALL also carry the current exporter''s Quire provenance and preserve an optional declared producer identity from the committed input frontmatter. The identity ties the exported file to one committed revision of its source. No field is taken from the wall clock, the host or the user''s environment.'
scope:
- id: FR-034
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: '`quire export` of a Markdown input SHALL assemble the six document identity fields below and SHALL hand them to the converter as data and as generated title-page text ([FR-035](./FR-035-default-export-template.md), [FR-036](./FR-036-organisation-export-template.md)). It SHALL also carry the current exporter''s Quire provenance and preserve an optional declared producer identity from the committed input frontmatter. The identity ties the exported file to one committed revision of its source. No field is taken from the wall clock, the host or the user''s environment.'
```

+++