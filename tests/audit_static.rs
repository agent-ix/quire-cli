//! Static-audit gates as test symbols.
//!
//! `make ci` still runs `scripts/check_thin_boundary.sh`, `cargo deny check
//! bans` and `scripts/check_unsafe_comments.sh` — these tests do not replace
//! those gates. They exist because the matrix rows they back declare
//! `Type: Static`, which is NOT in the module's `no_source_symbol` exemption
//! list, so a row whose only evidence is a shell script can never be backed
//! however it is tagged (agent-ix/quire-cli#43).
//!
//! Trace ids sit on the tests, not in this header — a `//!` block attaches to
//! the file and binds to no symbol.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run a repo script under `bash` and return (success, combined output).
fn run_script(rel: &str) -> (bool, String) {
    let root = repo_root();
    let out = Command::new("bash")
        .arg(root.join(rel))
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| panic!("failed to launch {rel}: {e}"));
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), combined)
}

// TC-090 (StR-004-AC-2, FR-004-AC-9, FR-014-AC-9, FR-015-AC-6): `src/` is a
// thin process boundary — no markdown parsing, no structural-validation logic,
// no render/template code outside the documented dispatch sites.
#[test]
fn tc090_src_is_a_thin_boundary_over_quire_rs() {
    let (ok, output) = run_script("scripts/check_thin_boundary.sh");
    assert!(ok, "thin-boundary audit failed:\n{output}");
}

// TC-091 (NFR-004-AC-1): no HTTP/RPC client crate reaches the build. `cargo
// deny check bans` is the CI gate; this asserts the same property directly
// against `deny.toml` and `Cargo.lock` so the row is backed without depending
// on cargo-deny being installed, and so a ban silently dropped from deny.toml
// is caught by the second half rather than passing an empty check.
#[test]
fn tc091_no_http_client_crate_is_banned_or_linked() {
    let root = repo_root();
    let deny = std::fs::read_to_string(root.join("deny.toml")).expect("deny.toml");
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).expect("Cargo.lock");

    const CLIENTS: &[&str] = &["reqwest", "hyper", "tonic", "surf", "ureq"];
    for client in CLIENTS {
        assert!(
            deny.contains(&format!("name = \"{client}\"")),
            "deny.toml no longer bans `{client}` (NFR-004-AC-1)"
        );
        assert!(
            !lock
                .lines()
                .any(|l| l.trim() == format!("name = \"{client}\"")),
            "`{client}` is in Cargo.lock — the process can leave the sandbox"
        );
    }
}

// TC-092 (NFR-003-AC-1): every `unsafe {` block in src/ and tests/ carries a
// `// SAFETY:` comment or sits in the reviewed baseline.
#[test]
fn tc092_every_unsafe_block_is_documented() {
    let (ok, output) = run_script("scripts/check_unsafe_comments.sh");
    assert!(ok, "unsafe-comment audit failed:\n{output}");
}

// TC-093 (FR-016-AC-5, FR-016-AC-6, StR-004-AC-2): the `self_update` engine is
// package-agnostic — it is driven by a config struct and imports nothing from
// quire's `io` or command context, so `commands/update.rs` stays the only
// quire-specific glue. Source inspection is the only way to reach this: no
// runtime path can observe an import that isn't there.
#[test]
fn tc093_self_update_engine_is_package_agnostic() {
    let root = repo_root();
    let engine = std::fs::read_to_string(root.join("src/self_update/mod.rs"))
        .expect("src/self_update/mod.rs");

    const FORBIDDEN: &[&str] = &[
        "crate::io",
        "crate::ctx",
        "crate::Ctx",
        "use crate::commands",
    ];
    for needle in FORBIDDEN {
        assert!(
            !engine.contains(needle),
            "src/self_update/mod.rs reaches into `{needle}`; the engine must stay \
             config-struct driven (FR-016-AC-6)"
        );
    }

    // And the glue that does know about quire carries no parser/validator work.
    let glue = std::fs::read_to_string(root.join("src/commands/update.rs"))
        .expect("src/commands/update.rs");
    for needle in [
        "quire_rs::parse_document",
        "quire_rs::validate",
        "quire_rs::extract",
    ] {
        assert!(
            !glue.contains(needle),
            "src/commands/update.rs carries `{needle}`; `update` is not a document command"
        );
    }
}
