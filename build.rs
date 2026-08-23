//! Resolve the `quire-rs` version this binary actually links, at build time.
//!
//! The parser lives in `src/lockfile.rs` and is `include!`d here rather than
//! written inline: `cargo test` does not run a build script's `#[cfg(test)]`
//! module, so tests written in this file would compile and never execute.
//! Shared with the library, they run on every `cargo test` — which for a parser
//! whose failure mode is silently reporting a plausible wrong number is the
//! difference between a gate and a decoration (agent-ix/quire-cli#68).

use std::path::PathBuf;

include!("src/lockfile.rs");

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let lock = manifest_dir.join("Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock.display());
    println!("cargo:rerun-if-changed=src/lockfile.rs");
    println!("cargo:rerun-if-changed=build.rs");

    // `unknown` rather than a fallback to the crate version. An unresolvable
    // engine is a fact a consumer must be able to see; substituting something
    // plausible is the confident-but-wrong claim this whole surface prevents.
    let resolved = std::fs::read_to_string(&lock)
        .ok()
        .and_then(|text| engine_version(&text))
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=QUIRE_ENGINE_VERSION={resolved}");
}
