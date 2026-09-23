//! IT-145: public documentation and the exact engine pin agree on FR-020.

mod common;

use std::fs;

use common::quire;

// Trace: IT-145, FR-020-AC-9
#[test]
fn it_145_help_docs_capability_and_dependency_pin_agree() {
    let help = quire().arg("--help").output().expect("help");
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).expect("UTF-8 help");
    assert!(help.contains("assurance   Emit source-grounded assurance facts"));

    let root = env!("CARGO_MANIFEST_DIR");
    let read = |relative: &str| fs::read_to_string(format!("{root}/{relative}")).expect(relative);
    let manifest = read("Cargo.toml");
    let readme = read("README.md");
    let changelog = read("CHANGELOG.md");

    // The engine identity lives in Cargo.toml alone, pinned by rev with an
    // exact version; `cargo --locked` already checks the lock agrees with it.
    // No prose restates the SHA.
    let pin = manifest
        .lines()
        .find(|line| line.starts_with("quire-rs = "))
        .expect("Cargo.toml declares quire-rs");
    assert!(pin.contains("rev = \""), "quire-rs is pinned by rev: {pin}");
    assert!(
        pin.contains("version = \"="),
        "quire-rs is pinned to an exact version: {pin}"
    );

    assert!(readme.contains("quire assurance"));
    assert!(readme.contains("--expect-schema <MODULE/ARCHETYPE@SHA256>"));
    assert!(changelog.contains("assurance_export.v1"));
    assert!(quire_cli::engine::CAPABILITIES.contains(&"assurance_export.v1"));
}
