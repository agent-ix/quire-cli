# quire-dist

`quire-dist` is the non-published Rust tooling for `quire-cli` releases. It is a
workspace member but not an installed product binary; `cargo install --path .`
continues to install only `quire`.

The tool owns the native/npm target catalog, executable-format checks, package
assembly, and Cargo/npm version agreement:

```bash
make set-version VERSION=0.33.0
make dist-verify VERSION=0.33.0
make dist-package VERSION=0.33.0
make dist-test
```

`package-npm` expects one validated executable at
`artifacts/<rust-target>/quire[.exe]` for every catalog target and writes the
four platform packages under `npm/dist/`. It never publishes. The manually
dispatched release workflow remains responsible for invoking external build,
GitHub, and npm hosts.

This crate has no `quire-rs` dependency and treats executables as opaque
distribution artifacts. It does not parse documents or implement extraction,
formal-clause profiles, grammar, temporal/protocol behavior, or source
semantics.
