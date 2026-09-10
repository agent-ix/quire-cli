//! Static-audit gates as test symbols.
//!
//! `make ci` runs the native thin-boundary and unsafe-comment gates plus
//! `cargo deny check bans`. These tests do not replace those gates. They exist
//! because the matrix rows they back declare
//! `Type: Static`, which is NOT in the module's `no_source_symbol` exemption
//! list, so a row whose only evidence is a shell script can never be backed
//! however it is tagged (agent-ix/quire-cli#43).
//!
//! Trace ids sit on the tests, not in this header — a `//!` block attaches to
//! the file and binds to no symbol.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// Trace: TC-090, TC-814, StR-004-VC-2, StR-004-VC-3, FR-004-AC-9,
// FR-014-AC-9, FR-015-AC-6, FR-017-AC-9, FR-020-AC-6
// `src/` is a
// thin process boundary — no markdown parsing, no structural-validation logic,
// no render/template code outside the documented dispatch sites.
#[test]
fn tc090_src_is_a_thin_boundary_over_quire_rs() {
    quire_qualify::source::check_thin_boundary(&repo_root()).expect("native thin-boundary audit");
}

// TC-091, NFR-004-AC-1, NFR-004-AC-3: no HTTP/RPC client crate reaches the
// build. `cargo
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

// TC-092, NFR-003-AC-1, NFR-003-AC-2: every `unsafe {` block in `src/` carries
// a `// SAFETY:` comment or sits in the reviewed baseline (AC-1), AND the gate
// actually fails on a violation (AC-2).
//
// The second half is the point. Asserting only that the script exits 0 today
// proves the tree is clean, not that the gate would catch anything — a script
// that unconditionally returned 0 would pass it. So the check is run a second
// time against a synthetic tree containing an undocumented `unsafe` block, and
// a third against the same block with its `// SAFETY:` comment, in a tempdir,
// so neither run touches this repo.
#[test]
fn tc092_every_unsafe_block_is_documented_and_the_gate_catches_violations() {
    // AC-1: this repository is clean.
    quire_qualify::unsafe_comments::check(&repo_root()).expect("native unsafe-comment audit");

    // AC-2: the gate refuses an undocumented `unsafe` block.
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::create_dir_all(dir.path().join("scripts")).unwrap();
    std::fs::write(dir.path().join("scripts/unsafe_comment_baseline.txt"), "").unwrap();
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "pub fn f() {\n    unsafe {\n        let _ = 1;\n    }\n}\n",
    )
    .unwrap();
    let error = quire_qualify::unsafe_comments::check(dir.path())
        .expect_err("the gate must reject an undocumented unsafe block");
    assert!(matches!(
        error,
        quire_qualify::Error::UnsafeBaseline { unreviewed, stale }
            if unreviewed == ["src/lib.rs:2"] && stale.is_empty()
    ));

    // ...and accepts the same block once documented, so the refusal is about
    // the comment and not about the file merely containing `unsafe`.
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "pub fn f() {\n    // SAFETY: nothing is dereferenced here.\n    unsafe {\n        let _ = 1;\n    }\n}\n",
    )
    .unwrap();
    quire_qualify::unsafe_comments::check(dir.path())
        .expect("the gate must accept the documented block");
}

// TC-093, FR-016-AC-5, FR-016-AC-6, StR-004-AC-2: the `self_update` engine is
// package-agnostic — it is driven by a config struct and imports nothing from
// quire's `io` or command context, so `commands/update.rs` stays the only
// quire-specific glue. Source inspection is the only way to reach this: no
// runtime path can observe an import that isn't there.
//
// The gate matches expanded module paths, not bare substrings (#56, SR-006
// FND-003): the previous `contains("crate::io")` check was evaded by any
// grouped import (`use crate::{io, …}`), which contains no needle at all.
#[test]
fn tc093_self_update_engine_is_package_agnostic() {
    quire_qualify::source::check_thin_boundary(&repo_root())
        .expect("AST-backed self-update and CLI boundary audit");
}
