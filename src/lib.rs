//! Shared library surface for the `quire` binary.
//!
//! This crate is primarily a binary (`src/main.rs`); the library exists
//! so integration tests can import the path-safety guards and I/O
//! helpers without going through the process boundary.

pub mod engine;
pub mod io;
/// Read the resolved `quire-rs` version out of a Cargo lockfile (#68).
///
/// **Why the lockfile and not a constant.** `quire-rs`'s own `Cargo.toml` says
/// `version = "0.33.0"` while its released tags run to `v0.45.0` — that crate
/// derives its version from the tag and does not keep the manifest in step, so
/// `quire_rs::VERSION` (were it to exist) would report a number nobody ships.
/// The lockfile records what Cargo *resolved*: the tag and the exact commit.
/// A constant records what somebody wrote; a lockfile records what was built,
/// and this surface exists because those two disagreed silently.
///
/// **Why the source is shared with `build.rs`.** The build script `include!`s
/// this file. `cargo test` does not run a build script's `#[cfg(test)]` module,
/// so tests written there compile and never execute; shared here they run on
/// every `cargo test` — which, for a parser whose failure mode is silently
/// reporting a plausible wrong number, is the difference between a gate and a
/// decoration.
pub mod lockfile;
pub mod safety;
pub mod self_update;
