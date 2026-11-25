<!-- reviewer repo=agent-ix/quire-cli visibility=public quoin=0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-967175026302197768900474181935084796 method=spec-review/ears-conformance lang=spec pr=quire-cli#247 model=gpt-6.2-sol run=b3819e72-43a8-46b1-bf20-e61067ef23ba -->

+++ [reviewer data]

```yaml
clean: true
scope:
  - {id: FR-047-AC-3, path: "spec/functional/FR-047-quire-selftest.md", role: examined, excerpt: "Given one untagged case in each owning repository, quire matrix --repo agent-ix/quire-cli=<quire-cli-root> --repo agent-ix/quire-rs=<quire-rs-root> --strict reports both cases as untagged with their source repository identities; for each corpus member, strict mode checks every selftest/**/run.sh for at least one shell trace marker that names an acceptance criterion in the corpus, and reports a missing or invalid marker with the exact script path. A quire-cli selftest case with a valid shell trace marker binds its named criterion. Trace tags remain in each repository's own selftest sources, and no case is copied between repositories."}
findings: []
```
+++

