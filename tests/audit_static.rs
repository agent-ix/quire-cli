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

// TC-090, TC-814, StR-004-AC-2, FR-004-AC-9, FR-014-AC-9, FR-015-AC-6,
// FR-017-AC-9, FR-020-AC-6, US-006-AC-4:
// `src/` is a
// thin process boundary — no markdown parsing, no structural-validation logic,
// no render/template code outside the documented dispatch sites.
#[test]
fn tc090_src_is_a_thin_boundary_over_quire_rs() {
    let (ok, output) = run_script("scripts/check_thin_boundary.sh");
    assert!(ok, "thin-boundary audit failed:\n{output}");
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
// a third against the same block with its `// SAFETY:` comment, in a tempdir:
// the script resolves `src/` and its baseline relative to the working
// directory, so neither run touches this repo.
#[test]
fn tc092_every_unsafe_block_is_documented_and_the_gate_catches_violations() {
    let script = repo_root().join("scripts/check_unsafe_comments.sh");

    // AC-1: this repository is clean.
    let (ok, output) = run_script("scripts/check_unsafe_comments.sh");
    assert!(ok, "unsafe-comment audit failed:\n{output}");

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
    let out = Command::new("bash")
        .arg(&script)
        .current_dir(dir.path())
        .output()
        .expect("run gate on violating tree");
    assert!(
        !out.status.success(),
        "the gate passed an undocumented `unsafe` block — it catches nothing"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("missing SAFETY comment"),
        "the refusal must name the missing comment; got: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // ...and accepts the same block once documented, so the refusal is about
    // the comment and not about the file merely containing `unsafe`.
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "pub fn f() {\n    // SAFETY: nothing is dereferenced here.\n    unsafe {\n        let _ = 1;\n    }\n}\n",
    )
    .unwrap();
    let out = Command::new("bash")
        .arg(&script)
        .current_dir(dir.path())
        .output()
        .expect("run gate on documented tree");
    assert!(
        out.status.success(),
        "the gate rejected a documented `unsafe` block: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Split `s` on commas at brace-nesting depth 0 (helper for
/// [`expanded_use_paths`]).
fn split_top_level(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

/// Recursively expand one `use` declaration body — possibly a brace group,
/// possibly aliased — into fully-qualified paths.
fn expand_use(decl: &str, prefix: &str, out: &mut Vec<String>) {
    let decl = decl.trim();
    if let Some(open) = decl.find('{') {
        let stem = decl[..open].trim().trim_end_matches("::").trim();
        let inner = decl[open + 1..]
            .rsplit_once('}')
            .map(|(i, _)| i)
            .unwrap_or(&decl[open + 1..]);
        let prefix = if stem.is_empty() {
            prefix.to_string()
        } else {
            format!("{prefix}{stem}::")
        };
        for part in split_top_level(inner) {
            if !part.trim().is_empty() {
                expand_use(part, &prefix, out);
            }
        }
    } else {
        // `path as alias` imports `path`; `self` names the group's stem.
        let path = decl.split(" as ").next().unwrap_or(decl).trim();
        if path == "self" {
            out.push(prefix.trim_end_matches("::").to_string());
        } else {
            out.push(format!("{prefix}{path}"));
        }
    }
}

/// Every path a file's `use` declarations import, with brace groups expanded
/// (`use a::{b, c::d}` → `a::b`, `a::c::d`) — so a grouped import cannot
/// evade a needle match (#56, SR-006 FND-003).
fn expanded_use_paths(source: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for stmt in source.split(';') {
        // Collapse whitespace (incl. newlines inside multi-line groups).
        let stmt = stmt.split_whitespace().collect::<Vec<_>>().join(" ");
        for lead in ["use ", "pub use ", "pub(crate) use "] {
            if let Some(rest) = stmt.trim_start().strip_prefix(lead) {
                expand_use(rest, "", &mut paths);
                break;
            }
        }
    }
    paths
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
    let root = repo_root();
    let engine = std::fs::read_to_string(root.join("src/self_update/mod.rs"))
        .expect("src/self_update/mod.rs");

    // Module paths the engine must not reach, however imported. `quire_cli::`
    // forms cover a self-import through the lib name.
    const FORBIDDEN_PREFIXES: &[&str] = &[
        "crate::io",
        "crate::commands",
        "crate::Ctx",
        "quire_cli::io",
        "quire_cli::commands",
    ];
    let forbidden = |path: &str| {
        FORBIDDEN_PREFIXES
            .iter()
            .any(|p| path == *p || path.starts_with(&format!("{p}::")))
            // A glob at the crate root imports `io` wholesale.
            || path == "crate::*"
            || path == "quire_cli::*"
    };

    for path in expanded_use_paths(&engine) {
        assert!(
            !forbidden(&path),
            "src/self_update/mod.rs imports `{path}`; the engine must stay \
             config-struct driven (FR-016-AC-6)"
        );
    }
    // Inline fully-qualified references need no `use` at all — check them on
    // whitespace-normalized text so formatting cannot hide one.
    let compact: String = engine.split_whitespace().collect();
    for needle in [
        "crate::io::",
        "crate::commands::",
        "crate::Ctx",
        "crate::ctx",
    ] {
        assert!(
            !compact.contains(needle),
            "src/self_update/mod.rs references `{needle}` inline; the engine \
             must stay config-struct driven (FR-016-AC-6)"
        );
    }

    // The hardening is proven to catch, not merely to pass (the TC-092
    // pattern): each evasion shape the substring gate missed is flagged.
    for evasion in [
        "use crate::{io, safety};",
        "use crate::{\n    io,\n    safety,\n};",
        "pub use crate::{commands::Ctx as C, safety};",
        "use quire_cli::{io::emit_diagnostic, safety};",
    ] {
        assert!(
            expanded_use_paths(evasion).iter().any(|p| forbidden(p)),
            "the gate did not catch the grouped-import evasion: {evasion}"
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
