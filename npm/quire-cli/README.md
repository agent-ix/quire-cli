# @agent-ix/quire-cli

Prebuilt distribution of [`quire`](https://github.com/agent-ix/quire-cli), a
static CLI over [`quire-rs`](https://github.com/agent-ix/quire-rs) for parsing,
extracting, looking up, editing, validating, and inspecting Markdown artifacts.

This npm package is a thin launcher. The actual binary ships in a per-platform
optional dependency (`@agent-ix/quire-cli-<os>-<arch>`); npm installs only the
one matching your machine. No source build and no `quire-rs` checkout are
required.

## Install

This package is published to the **public npm registry**, so no auth or registry
config is needed:

```bash
npm install -g @agent-ix/quire-cli
quire --help
```

Or run without installing:

```bash
npx @agent-ix/quire-cli parse README.md
```

## Supported platforms

| os / arch       | Rust target                     |
|-----------------|---------------------------------|
| linux-x64       | `x86_64-unknown-linux-musl`     |
| linux-arm64     | `aarch64-unknown-linux-musl`    |
| darwin-arm64    | `aarch64-apple-darwin`          |
| win32-x64       | `x86_64-pc-windows-msvc`        |

Linux x64 covers both Intel and AMD servers; win32-x64 covers both Intel and AMD
Windows machines. macOS support targets Apple Silicon. The Linux binaries are
statically linked against musl.

See the [main repository](https://github.com/agent-ix/quire-cli) for command
documentation.
