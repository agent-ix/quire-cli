// Read the resolved `quire-rs` version out of a Cargo lockfile.
//
// This file is `include!`d by `build.rs` as well as compiled into the library,
// so it carries NO `//!` inner doc comments — `include!` rejects them. The
// module's documentation sits on the `pub mod lockfile;` declaration in
// `lib.rs`, which is the one place both builds agree on.

/// Parse the `[[package]]` stanza named `quire-rs` out of a lockfile.
///
/// Hand-parsed rather than pulled through a TOML dependency: a build script's
/// dependencies are compiled for the host on every build, and this is a short
/// scan over a file whose shape Cargo guarantees.
///
/// Preference order within the stanza is deliberate. A git source's `?tag=` /
/// `?rev=` / `?branch=` fragment is what the operator pinned and what
/// `make check-engine` will compare against; the stanza's own `version` field
/// is the dependency's manifest value, which for this dependency is stale by
/// twelve minor releases. A path dependency has no locator, so the manifest
/// value is all there is — and it is correct there, because a path dependency
/// is the tree in front of you.
///
/// Returns `None` when the package is absent. Never guesses: an unresolvable
/// engine is a fact a caller must be able to see, and substituting something
/// plausible is the confident-but-wrong claim this file exists to prevent.
pub fn engine_version(lock: &str) -> Option<String> {
    let mut in_package = false;
    let mut manifest_version: Option<String> = None;

    for line in lock.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            // A new stanza begins. If the previous one was ours and carried no
            // git source, its manifest version is the answer.
            if in_package {
                return manifest_version;
            }
            in_package = false;
            manifest_version = None;
            continue;
        }
        if let Some(name) = strip_kv(line, "name") {
            in_package = name == "quire-rs";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(version) = strip_kv(line, "version") {
            manifest_version = Some(version.to_string());
            continue;
        }
        if let Some(source) = strip_kv(line, "source") {
            if let Some(pin) = git_pin(source) {
                return Some(pin);
            }
        }
    }
    // The stanza ran to end-of-file.
    in_package.then_some(manifest_version).flatten()
}

/// `key = "value"` → `value`, for the exact key.
fn strip_kv<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(key)?.trim_start().strip_prefix('=')?;
    rest.trim().strip_prefix('"')?.strip_suffix('"')
}

/// The pinned ref out of a git source locator, if it is one.
///
/// `git+https://github.com/agent-ix/quire-rs?tag=v0.45.0#99e97f0…` → `0.45.0`.
/// The leading `v` is dropped so the value reads as a version rather than a ref
/// name; everything else — `rev=`, `branch=`, a `-<n>-g<sha>` describe suffix —
/// travels **verbatim**, because the point of this field is to say what is
/// actually linked, not to normalise it into something familiar. Rounding a
/// describe suffix to its nearest tag is the original defect restated one layer
/// down.
fn git_pin(source: &str) -> Option<String> {
    let source = source.strip_prefix("git+")?;
    let query = source.split('#').next().unwrap_or(source);
    let (_, params) = query.split_once('?')?;
    for param in params.split('&') {
        for key in ["tag", "rev", "branch"] {
            if let Some(value) = param.strip_prefix(key).and_then(|r| r.strip_prefix('=')) {
                return Some(value.strip_prefix('v').unwrap_or(value).to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod lockfile_tests {
    use super::*;

    const LOCK: &str = r#"
[[package]]
name = "serde"
version = "1.0.0"

[[package]]
name = "quire-rs"
version = "0.33.0"
source = "git+https://github.com/agent-ix/quire-rs?tag=v0.45.0#99e97f013d858ff678b7c0783c1998703c268d71"
dependencies = [
 "globset",
]

[[package]]
name = "glob"
version = "0.3.0"
"#;

    // The whole point: this lockfile says BOTH 0.33.0 and v0.45.0, and 0.33.0
    // is the number a constant would have reported.
    #[test]
    fn the_git_tag_wins_over_the_stale_manifest_version() {
        assert_eq!(engine_version(LOCK).as_deref(), Some("0.45.0"));
    }

    #[test]
    fn a_path_dependency_reports_its_manifest_version() {
        let lock = "[[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n";
        assert_eq!(engine_version(lock).as_deref(), Some("0.33.0"));
    }

    #[test]
    fn a_describe_suffix_and_a_rev_survive_verbatim() {
        let described = "[[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n\
             source = \"git+https://x?tag=v0.45.0-3-g99e97f0#abc\"\n";
        assert_eq!(
            engine_version(described).as_deref(),
            Some("0.45.0-3-g99e97f0"),
        );

        let rev = "[[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n\
             source = \"git+https://x?rev=99e97f0#99e97f0\"\n";
        assert_eq!(engine_version(rev).as_deref(), Some("99e97f0"));
    }

    #[test]
    fn an_absent_dependency_is_absent_rather_than_guessed() {
        assert_eq!(
            engine_version("[[package]]\nname = \"serde\"\nversion = \"1\"\n"),
            None,
        );
    }

    // `in_package` must clear on the next `name =`, or the first git dependency
    // AFTER quire-rs would supply its pin — a mutation that leaves every other
    // case in this module green.
    #[test]
    fn another_packages_source_is_not_read_as_ours() {
        let lock = "[[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n\n\
             [[package]]\nname = \"other\"\nversion = \"9.9.9\"\n\
             source = \"git+https://x?tag=v9.9.9#abc\"\n";
        assert_eq!(engine_version(lock).as_deref(), Some("0.33.0"));
    }

    // This crate's OWN lockfile, read the way build.rs reads it. The fixtures
    // above pin the parser against strings; this pins it against the file that
    // actually decides what every payload reports.
    #[test]
    fn the_real_lockfile_resolves_to_a_git_pin() {
        let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
            .expect("Cargo.lock");
        let resolved = engine_version(&lock).expect("quire-rs is a dependency of this crate");
        assert_ne!(
            resolved,
            env!("CARGO_PKG_VERSION"),
            "the CLI version must never be reported as the engine version",
        );
        assert!(!resolved.is_empty());
    }
}
