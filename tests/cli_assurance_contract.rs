//! IT-145: public documentation and the exact engine pin agree on FR-020.

mod common;

use std::fs;

use common::quire;

const ENGINE_REVISION: &str = "523e47f61ca5532c3c86064ed4872a1c5de3ed02";

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
    let lock = read("Cargo.lock");
    let readme = read("README.md");
    let changelog = read("CHANGELOG.md");
    let spec = read("spec/functional/FR-020-assurance-export-subcommand.md");

    assert!(manifest.contains(&format!("rev = \"{ENGINE_REVISION}\"")));
    assert!(lock.contains(&format!("rev={ENGINE_REVISION}#{ENGINE_REVISION}")));
    assert!(readme.contains("quire assurance"));
    assert!(readme.contains("--expect-schema <MODULE/ARCHETYPE@SHA256>"));
    assert!(changelog.contains(ENGINE_REVISION));
    assert!(changelog.contains("assurance_export.v1"));
    // FR-020-CON-1 states the pin as a normative constraint, not just narrative
    // prose (the "CR followup" blockquotes above it) -- IT-145's job is exact
    // manifest/lock/spec/changelog/executable agreement, and a spec file naming
    // a stale rev with every other check green is exactly the drift this test
    // exists to catch.
    assert!(spec.contains(&format!("revision `{ENGINE_REVISION}` and use its owned")));
    assert!(quire_cli::engine::CAPABILITIES.contains(&"assurance_export.v1"));

    let resolved = quire_cli::lockfile::engine_source_revision(&lock).expect("engine revision");
    assert_eq!(resolved, ENGINE_REVISION);
    let version = quire_cli::lockfile::engine_manifest_version(&lock).expect("engine version");
    assert_eq!(version, "0.46.0");
}
