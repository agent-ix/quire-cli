# quire-dist

`quire-dist` is the non-published Rust release tooling for `quire-cli`. It is a
workspace member but not an installed product binary; `cargo install --path .`
continues to install only `quire`.

Its one command, `verify-binary --version X.Y.Z --binary <path>`, executes a
built `quire` and fails unless its `--version` reports exactly the release
version. The release workflow runs it for each target before publishing.

npm packaging is not done here. The release workflow publishes through the
shared `agent-ix/nodejs-actions/publish-native-npm` action (ADR-0003).

This crate has no `quire-rs` dependency and treats executables as opaque
artifacts.

```bash
make dist-test
```
