//! `quire update` end-to-end behavior at the process boundary.
//!
//! The test binary lives under `target/<profile>/` — neither a `node_modules`
//! tree nor `~/.cargo` — so the install-source detector resolves to `Unknown`.
//! On the Unknown path `update` must print manual upgrade instructions and exit
//! 0 WITHOUT shelling out to npm/cargo (so the test never touches the network).

mod common;

use common::quire;

// IT-083, FR-016-AC-1, FR-016-AC-2: `update --check` on an Unknown install
// source prints the npm recipe, the cargo recipe and the releases URL, exits 0,
// and performs no install or network call.
#[test]
fn update_check_on_unknown_source_prints_manual_instructions_and_exits_zero() {
    let out = quire()
        .args(["update", "--check"])
        .output()
        .expect("update runs");
    assert!(out.status.success(), "update --check should exit 0");
    let stdout = String::from_utf8(out.stdout).expect("stdout is UTF-8");
    assert!(
        stdout.contains("could not determine how this binary was installed"),
        "expected Unknown-source guidance, got:\n{stdout}"
    );
    assert!(
        stdout.contains("npm install -g @agent-ix/quire-cli@latest"),
        "expected npm upgrade recipe, got:\n{stdout}"
    );
    assert!(
        stdout.contains("cargo install --git"),
        "expected cargo upgrade recipe, got:\n{stdout}"
    );
}

// IT-084, FR-016-AC-2: even without `--check`, an Unknown source performs no
// install — it only emits instructions — so this stays network-free and exits 0.
#[test]
fn update_without_check_on_unknown_source_is_also_safe() {
    let out = quire().arg("update").output().expect("update runs");
    assert!(
        out.status.success(),
        "update should exit 0 on Unknown source"
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("could not determine how this binary was installed"));
}
