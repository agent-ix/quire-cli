---
id: FR-004
title: "quire validate subcommand"
artifact_type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-003"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-017"
    type: "consumes"
    cardinality: "1:1"
---

## Behavior

The CLI SHALL expose a `validate` subcommand:

```
quire validate <ARCHETYPE> --module <PATH> --data <FILE|->
```

Behavior:
1. Path-safety on `--module` and `--data` (when not `-`).
2. Load Registry; resolve archetype by name. Unknown archetype → exit 1 with `UnknownArchetype` diagnostic.
3. Read context JSON.
4. Dispatch to `quire_rs::validate(&compiled_archetype, &data)` (upstream FR-002).
5. On success: exit 0, no stdout.
6. On schema violation: write structured violation list to stderr per upstream FR-017, exit 1.

`validate` SHALL NOT render or write any artifact body. Used as a fast CI gate.

## Acceptance

- **FR-004-AC-1**: `quire validate FR --module $ISO --data valid.json` exits 0 with no output.
- **FR-004-AC-2**: `quire validate FR --module $ISO --data invalid.json` (missing required `id`) exits 1; stderr contains a violation naming `/id` and the `required` constraint.
- **FR-004-AC-3**: `quire validate NONEXISTENT --module $ISO --data x.json` exits 1 with `UnknownArchetype` on stderr.
- **FR-004-AC-4**: For each ISO archetype, a hand-crafted valid context exits 0 and a hand-crafted invalid context exits 1 (parametric test across all 8).
