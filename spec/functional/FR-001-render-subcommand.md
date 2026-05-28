---
id: FR-001
title: "quire render subcommand"
artifact_type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-014"
    type: "consumes"
    cardinality: "1:1"
---

## Behavior

The CLI SHALL expose a `render` subcommand with the following surface:

```
quire render <ARCHETYPE> --module <PATH> --data <FILE|->  [--out <PATH>]
```

Required arguments:
- `<ARCHETYPE>` — positional, archetype name as known to the loaded `Registry` (e.g. `FR`, `NFR`, `ADR`).
- `--module <PATH>` — filesystem path to a directory containing a Filament Module conformant to `filament-core/FR-035` (manifest + templates + schemas).
- `--data <FILE|->` — path to a JSON file containing the render context, or `-` to read from stdin.

Optional:
- `--out <PATH>` — write rendered output to PATH instead of stdout. PATH MUST NOT contain `..`.

Behavior:
1. Argument parsing via `clap` derive. `--help` lists all flags. `--version` prints the crate version.
2. Path-safety check (per **FR-005**) on `--module`, `--data` (when not `-`), and `--out`.
3. Load the module: `quire_rs::Registry::load_from(module_path)` (consumer of upstream FR-014). Module load errors are propagated per **FR-007**.
4. Look up the archetype by name. Unknown archetype → exit 1 with `UnknownArchetype` diagnostic.
5. Read context JSON (from file or stdin), deserialize as `serde_json::Value`.
6. Dispatch to `quire_rs::render_by_name(&registry, archetype, &data)` (consumer of upstream FR-001 + FR-002).
7. On success: write rendered markdown to stdout or `--out` PATH.
8. On error: write structured diagnostic(s) to stderr per upstream FR-017, exit 1.

The CLI SHALL NOT implement rendering or schema validation logic — the call delegates entirely to `quire-rs`.

## Acceptance

- **FR-001-AC-1**: `quire render FR --module $ISO --data ctx.json` against the canonical ISO FR archetype produces byte-identical output to `minijinja-cli templates/fr.md.j2 ctx.json` for the same context.
- **FR-001-AC-2**: `quire render FR --module $ISO --data -` reads context from stdin and produces equivalent output.
- **FR-001-AC-3**: `quire render NONEXISTENT --module $ISO --data ctx.json` exits 1; stderr begins with a `UnknownArchetype` diagnostic; no stdout output.
- **FR-001-AC-4**: `quire render FR --module $ISO --data invalid.json` (failing schema) exits 1; stderr lists violations per FR-017; stdout is empty.
- **FR-001-AC-5**: `quire render FR --module $ISO --data ctx.json --out /tmp/fr.md` writes the file and produces no stdout.
- **FR-001-AC-6**: All 8 ISO archetypes (FR, NFR, StR, US, IT, TC, AC, CON) render byte-identically through this subcommand vs. the Python Jinja2 reference (parity matrix vendored from `quire-rs/tests/render_parity/`).
