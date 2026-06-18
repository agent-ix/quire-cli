[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

<p align="center">
  <img src="logo.png" alt="Quire" width="100%" />
</p>

# quire-cli

`quire-cli` is a static command-line wrapper around
[`quire-rs`](https://github.com/agent-ix/quire-rs). It gives agents and
humans one fast binary for parsing, extracting, looking up, editing,
validating, and inspecting the input contract of Markdown artifacts.

The crate is intentionally a thin process boundary: Markdown parsing,
extraction, and structural validation live in `quire-rs`.

## Commands

```bash
quire parse <DOC|->
quire lookup <DOC|-> (--heading <TEXT> [--level <1..6>] | --id <ID> | --block-id <BLOCK_ID>) [--content]
quire edit <DOC|-> (--heading <TEXT> | --block-id <BLOCK_ID>) --content <FILE|-> [--out <PATH>]
quire extract <DOC|-> --module <PATH> [--archetype <NAME>]
quire validate <DOC|GLOB|->... [--scope <DIR>] [--module <PATH>] [--archetype <NAME>]
quire validate --okf <BUNDLE_DIR> [--scope <DIR>] [--module <PATH>]
quire schema <ARCHETYPE> --module <PATH>
```

Global flags:

| Flag | Default | Purpose |
|------|---------|---------|
| `--diagnostics-format <human\|json>` | `human` | stderr diagnostic encoding |
| `--pretty` | off | indented JSON output for `parse`, `lookup`, and `extract` |

Exit codes:

| Code | Meaning |
|------|---------|
| 0 | success |
| 1 | user error: parse failure, schema violation, unknown archetype, I/O error, lookup miss |
| 2 | argv error: missing required flag, unknown flag, invalid flag combination |
| 134 | panic, never expected |

## Install

### npm (prebuilt binary — recommended)

Published to the public npm registry as [`@agent-ix/quire-cli`](https://www.npmjs.com/package/@agent-ix/quire-cli).
A per-platform optional dependency carries the prebuilt binary, so no Rust
toolchain and no `quire-rs` checkout are needed — and because it resolves from
the default public registry, no auth or `.npmrc` config either:

```bash
npm install -g @agent-ix/quire-cli   # or: npx @agent-ix/quire-cli --help
quire --help
```

Prebuilt targets: linux-x64, linux-arm64 (musl/static), darwin-arm64, win32-x64.
Linux x64 covers Intel and AMD; win32-x64 covers Intel and AMD Windows.

### Prebuilt tarball

Each [release](https://github.com/agent-ix/quire-cli/releases) attaches a
`quire-<version>-<target>.tar.gz` (`.zip` on Windows) plus `SHA256SUMS.txt`.
Download, verify, and drop `quire` on your `PATH`.

### From source

`quire-cli` builds on [`quire-rs`](https://github.com/agent-ix/quire-rs), fetched
from GitHub at build time, so configure `cargo` with `net.git-fetch-with-cli = true`.

```bash
cargo install --git https://github.com/agent-ix/quire-cli
# or, from a checkout:
cargo build --release && target/release/quire --help
```

During development, `target/debug/quire` is fine for local testing.

## Usage Instructions

### Parse And Outline Markdown

Parse a document into a `QuireDocument` JSON envelope:

```bash
quire --pretty parse spec/functional/FR-001.md
```

Print a compact outline:

```bash
quire parse spec/functional/FR-001.md \
  | jq -r '.. | objects | select(has("heading") and has("level")) | "\(.level) \(.heading) id=\(.id) block_id=\(.block_id // "-") lines=\(.start_line)-\(.end_line)"'
```

Inspect frontmatter:

```bash
quire parse spec/functional/FR-001.md | jq '.frontmatter'
```

### Lookup One Section

Fetch one parsed section as JSON:

```bash
quire lookup spec/functional/FR-001.md --heading Behavior
quire lookup spec/functional/FR-001.md --heading "Document Title" --level 1
quire lookup spec/functional/FR-001.md --id behavior-L14
quire lookup spec/functional/FR-001.md --block-id blk-behavior
```

Fetch only section body bytes:

```bash
quire lookup spec/functional/FR-001.md --heading Acceptance --content
quire lookup spec/functional/FR-001.md --block-id blk-behavior --content
```

`--id` is generated as `<slug>-L<line>` and can change when lines move.
For stable machine addressing, author headings with Pandoc block IDs:

```markdown
## Behavior {#blk-behavior}
```

Then use:

```bash
quire lookup doc.md --block-id blk-behavior
```

### Edit One Section

Replace a single section's body (or a full block) without rewriting the rest of
the document — frontmatter and every untouched section stay byte-identical:

```bash
# Replace the Acceptance Criteria body in place (new content from stdin)
quire lookup FR-001.md --heading "Acceptance Criteria" --content   # read current
quire edit FR-001.md --heading "Acceptance Criteria" --content new-ac.md --out FR-001.md

# Replace a full stable block (heading line + body) from stdin
quire edit FR-001.md --block-id blk-behavior --content - < new-block.md
```

`--heading` content is the section BODY (everything after the heading line);
`--block-id` content is the FULL block (heading line, with its `{#blk-id}`
attribute, plus the body). Omit `--out` to write the updated document to stdout.

### Extract Records And Links

Run an archetype's `body_extraction` DSL and collect edges:

```bash
quire extract spec/objects/EX-001.md --module ./extract-mod
```

Override archetype inference when needed:

```bash
quire extract EX-001.md --module ./extract-mod --archetype ExtractSample
```

Show only harvested edges:

```bash
quire extract EX-001.md --module ./extract-mod | jq '.edges'
```

`extract` does not auto-validate. Run `validate` separately for context
JSON schema checks.

### Validate A Markdown Document

Structurally validate an authored document against its archetype. The archetype
is resolved from the frontmatter `type` unless `--archetype` overrides
it. Relative document globs are resolved under `--scope`; in scoped mode Quire
loads modules from that scope, `--scope ./.ix/modules` style plugin roots, and
`IX_SCHEMA_PATH`. quire-rs runs the archetype's `body_extraction` asserts (required-section
presence, non-placeholder content, table columns/rows, list items, id patterns)
plus frontmatter-schema and per-level heading uniqueness:

```bash
quire validate --scope . "spec/**/*.md"
quire validate --scope . "spec/functional/*.md" "spec/usecase/*.md"
quire validate ./spec/functional/FR-001.md --module ./iso
quire validate ./FR-001.md --module ./iso --archetype FR
cat FR-001.md | quire validate - --module ./iso --archetype FR
```

On success `validate` exits 0 with no output. On failure it exits 1 and writes
the line-numbered quire-rs diagnostics (naming the archetype, section/assert, and
reason: `missing`/`empty`/`placeholder`/`assert`/`frontmatter`/`duplicate-heading`)
to stderr — verbatim, the CLI adds no validation logic of its own.

Every document also satisfies the base **concept** contract before its archetype
runs: `type` is required and non-empty (the OKF discriminator), and the optional
OKF fields `description` (string) and `tags` (string array) are type-checked when
present.

### Validate An OKF Bundle (`--okf`)

`--okf` reads a *foreign* OKF bundle directory under a permissive posture for
portability. `type` is still required and non-empty, but unknown types, broken
`ix://` links, and `index.md` completeness gaps (every sibling artifact must be
listed; the bundle-root `index.md` must carry `okf_version`) are reported as
**warnings** (exit 0) rather than hard errors. An untyped document is still a
hard error.

```bash
quire validate --okf path/to/bundle --module ./iso
quire validate --okf --scope path/to/bundle      # scoped module discovery
```

Without `--okf`, bundle directories validated via `--scope "spec/**/*.md"` keep
the strict per-file posture (archetype conformance, resolvable references,
complete indexes are all hard requirements).

### Inspect An Archetype's Input Contract

Emit the archetype input contract — the frontmatter JSON Schema plus the
`body_extraction` asserts (required headings, table columns, id-patterns) that
`validate` enforces — as deterministic JSON. This is the same contract an
authoring agent fills; there is no template-variable list (templates were
removed):

```bash
quire schema FR --module ./iso
quire --pretty schema FR --module ./iso
```

Unknown archetypes exit 1 with `UnknownArchetype` on stderr.

## Agent Skills

This repository ships Codex-style agent skills under `skills/`. They are
intended to be packaged with or installed from `quire-cli` so agents can use
the CLI consistently without relearning command patterns.

Available skills:

| Skill | Slash-style name | Purpose |
|-------|------------------|---------|
| `explore-markdown` | `/explore-markdown` | Outline Markdown and fetch targeted sections with `parse` and `lookup`. |
| `write-markdown` | `/write-markdown` | Author Markdown artifacts against an archetype's input contract via `schema` + `validate`. |
| `validate-markdown` | `/validate-markdown` | Check authored Markdown structure with `validate`, `parse`, and `lookup`. |
| `link-markdown` | `/link-markdown` | Inspect relationships, `ix://` links, and blast radius with `extract`. |

Each skill has:

```text
skills/<name>/SKILL.md
skills/<name>/agents/openai.yaml
```

When adding a new shipped skill, keep it general, command-oriented, and
domain-neutral. Domain review guidance belongs in domain-specific skills,
not here.

Validate skills with:

```bash
python3 path/to/quick_validate.py skills/explore-markdown
python3 path/to/quick_validate.py skills/write-markdown
python3 path/to/quick_validate.py skills/validate-markdown
python3 path/to/quick_validate.py skills/link-markdown
```

## Safety

Path safety is part of the CLI contract:

- `--module`, `--out`, `--content`, and positional document paths reject `..`
  traversal where the command applies path safety.
- Symlink escapes are rejected.
- A positional `-` reads from stdin by design (path-safety-exempt).
- The CLI makes no network calls in normal operation; tests audit this with
  `strace`.

## Development

```bash
make build               # release build
make test                # cargo test, including integration tests and audits
make lint                # clippy -D warnings
make fmt-check           # rustfmt --check
make deny                # cargo deny check licenses
make deny-bans           # cargo deny check bans
make audit-unsafe        # every unsafe block carries a // SAFETY: comment
make audit-thin-boundary # src/ stays a thin wrapper over quire-rs
make refresh-fixtures    # re-sync tests/fixtures/iso from ../quire-rs
make ci                  # local CI gauntlet
```

The release binary is audited to link only baseline system libraries on
Linux. See `tests/audit_ldd.rs`.

## Spec And Plan

Requirements live in `spec/`; the implementation plan and task index live in
`plan/`. The traceability matrix in `spec/tests.md` maps acceptance criteria
to integration tests, benchmarks, or static audits.

## License

MIT
