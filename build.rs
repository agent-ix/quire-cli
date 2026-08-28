//! Resolve the `quire-rs` version this binary actually links, at build time.
//!
//! The parser lives in `src/lockfile.rs` and is `include!`d here rather than
//! written inline: `cargo test` does not run a build script's `#[cfg(test)]`
//! module, so tests written in this file would compile and never execute.
//! Shared with the library, they run on every `cargo test` — which for a parser
//! whose failure mode is silently reporting a plausible wrong number is the
//! difference between a gate and a decoration (agent-ix/quire-cli#68).

use std::path::{Path, PathBuf};
use std::process::Command;

include!("src/lockfile.rs");

fn main() {
    println!("cargo:rerun-if-changed=src/lockfile.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let lock = find_lockfile(&manifest_dir);
    let lock_text = lock.as_ref().and_then(|lock| {
        println!("cargo:rerun-if-changed={}", lock.display());
        std::fs::read_to_string(lock).ok()
    });
    let resolved = match lock_text.as_deref() {
        Some(text) => engine_version(text),
        None => None,
    };

    // `unknown` rather than a fallback to the crate version. An unresolvable
    // engine is a fact a consumer must be able to see; substituting something
    // plausible is the confident-but-wrong claim this whole surface prevents.
    let resolved = resolved.unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=QUIRE_ENGINE_VERSION={resolved}");
    println!(
        "cargo:rustc-env=QUIRE_ENGINE_MANIFEST_VERSION={}",
        lock_text
            .as_deref()
            .and_then(engine_manifest_version)
            .unwrap_or_else(|| "unknown".to_string())
    );
    println!(
        "cargo:rustc-env=QUIRE_ENGINE_SOURCE_REVISION={}",
        lock_text
            .as_deref()
            .and_then(engine_source_revision)
            .unwrap_or_else(|| "unknown".to_string())
    );
    let engine_revision = lock_text
        .as_deref()
        .and_then(engine_source_revision)
        .unwrap_or_else(|| "unknown".to_string());
    println!(
        "cargo:rustc-env=QUIRE_ENGINE_SOURCE_SHORT={}",
        short_revision(&engine_revision)
    );

    let (revision, state) = source_identity(&manifest_dir);
    println!("cargo:rustc-env=QUIRE_CLI_SOURCE_REVISION={revision}");
    println!(
        "cargo:rustc-env=QUIRE_CLI_SOURCE_SHORT={}",
        short_revision(&revision)
    );
    println!("cargo:rustc-env=QUIRE_CLI_SOURCE_STATE={state}");
}

fn short_revision(revision: &str) -> &str {
    revision.get(..8).unwrap_or(revision)
}

fn source_identity(repo: &Path) -> (String, &'static str) {
    let output = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .ok()
            .filter(|output| output.status.success())
    };
    let Some(head) = output(&["rev-parse", "HEAD"]) else {
        return ("unknown".to_string(), "unknown");
    };
    let revision = String::from_utf8_lossy(&head.stdout).trim().to_string();
    if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return ("unknown".to_string(), "unknown");
    }

    if let Some(files) = output(&["ls-files"]) {
        for file in String::from_utf8_lossy(&files.stdout).lines() {
            println!("cargo:rerun-if-changed={}", repo.join(file).display());
        }
    }
    if let Some(git_dir) = output(&["rev-parse", "--git-dir"]) {
        let git_dir = String::from_utf8_lossy(&git_dir.stdout).trim().to_string();
        let git_dir = if Path::new(&git_dir).is_absolute() {
            PathBuf::from(git_dir)
        } else {
            repo.join(git_dir)
        };
        println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
        println!("cargo:rerun-if-changed={}", git_dir.join("index").display());
    }

    match output(&["status", "--porcelain", "--untracked-files=normal"]) {
        Some(status) if status.stdout.is_empty() => (revision, "clean"),
        Some(_) => (revision, "dirty"),
        None => (revision, "unknown"),
    }
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
