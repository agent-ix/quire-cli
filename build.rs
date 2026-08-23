//! Resolve the `quire-rs` version this binary actually links, at build time.
//!
//! The parser lives in `src/lockfile.rs` and is `include!`d here rather than
//! written inline: `cargo test` does not run a build script's `#[cfg(test)]`
//! module, so tests written in this file would compile and never execute.
//! Shared with the library, they run on every `cargo test` — which for a parser
//! whose failure mode is silently reporting a plausible wrong number is the
//! difference between a gate and a decoration (agent-ix/quire-cli#68).

use std::path::{Path, PathBuf};

include!("src/lockfile.rs");

fn main() {
    println!("cargo:rerun-if-changed=src/lockfile.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let resolved = match find_lockfile(&manifest_dir) {
        Some(lock) => {
            println!("cargo:rerun-if-changed={}", lock.display());
            std::fs::read_to_string(&lock)
                .ok()
                .and_then(|text| engine_version(&text))
        }
        None => None,
    };

    // `unknown` rather than a fallback to the crate version. An unresolvable
    // engine is a fact a consumer must be able to see; substituting something
    // plausible is the confident-but-wrong claim this whole surface prevents.
    let resolved = resolved.unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=QUIRE_ENGINE_VERSION={resolved}");
}

/// The nearest `Cargo.lock` at or above the manifest directory.
///
/// Not just `CARGO_MANIFEST_DIR/Cargo.lock`: a workspace keeps one lockfile at
/// its root, so a build of `quire-cli` as a workspace member found no lockfile
/// and shipped a release binary reporting `engine unknown` — `cargo test` would
/// have caught it, `cargo build --release` alone would not.
fn find_lockfile(from: &Path) -> Option<PathBuf> {
    from.ancestors()
        .map(|dir| dir.join("Cargo.lock"))
        .find(|candidate| candidate.is_file())
}
