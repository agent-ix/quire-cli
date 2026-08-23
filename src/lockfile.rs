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
    // Every `[[package]]` stanza named `quire-rs`, in file order. Collected
    // rather than short-circuited on the first, because a lockfile can carry
    // more than one — `deny.toml` sets `multiple-versions = "allow"`, and Cargo
    // sorts by name then version, so returning the first would deterministically
    // report the LOWEST version present.
    let mut candidates: Vec<Resolved> = Vec::new();
    let mut current: Option<Resolved> = None;
    // Only a `[[package]]` table describes a resolved dependency. Cargo also
    // writes `[[patch.unused]]` stanzas carrying `name`/`version`/`source` — a
    // patch it explicitly DISCARDED — and `[metadata]` tables can carry the
    // same keys. Tracking the enclosing table, rather than only resetting on
    // the literal `[[package]]` line, is what keeps a discarded patch from
    // being reported as the linked engine.
    let mut in_package_table = false;

    for line in lock.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if let Some(finished) = current.take() {
                candidates.push(finished);
            }
            in_package_table = line == "[[package]]";
            continue;
        }
        if !in_package_table {
            continue;
        }
        if let Some(name) = strip_kv(line, "name") {
            current = (name == "quire-rs").then(Resolved::default);
            continue;
        }
        let Some(entry) = current.as_mut() else {
            continue;
        };
        if let Some(version) = strip_kv(line, "version") {
            entry.manifest_version = Some(version.to_string());
        } else if let Some(source) = strip_kv(line, "source") {
            entry.source = Some(source.to_string());
        }
    }
    if let Some(finished) = current.take() {
        candidates.push(finished);
    }

    // A git-sourced entry is the one this crate pins; prefer it over a registry
    // or path entry that a transitive dependency happened to pull in.
    candidates
        .iter()
        .find(|c| c.source.as_deref().is_some_and(|s| s.starts_with("git+")))
        .or_else(|| candidates.first())
        .and_then(Resolved::version)
}

/// One `[[package]]` stanza named `quire-rs`.
#[derive(Default)]
struct Resolved {
    manifest_version: Option<String>,
    source: Option<String>,
}

impl Resolved {
    /// The version to report for this stanza.
    ///
    /// For a git source: the `?tag=`/`?rev=`/`?branch=` locator, else the
    /// resolved commit sha from the `#` fragment. **Never the `version`
    /// field** — for a git dependency that is the dependency's own manifest
    /// constant, which for `quire-rs` has read `0.33.0` at every tag from
    /// v0.42.0 through v0.45.0. Falling back to it would report a number
    /// twelve minor releases stale: the exact defect this module exists to
    /// end, restated one layer down.
    ///
    /// For a registry or path source there is no locator, and the manifest
    /// version is both all there is and correct — a path dependency is the
    /// tree in front of you.
    fn version(&self) -> Option<String> {
        match self.source.as_deref() {
            Some(source) if source.starts_with("git+") => {
                git_pin(source).or_else(|| git_sha(source))
            }
            _ => self.manifest_version.clone(),
        }
    }
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
    let (_, params) = source.split('#').next()?.split_once('?')?;
    // Keys outer, params inner: the documented preference is tag → rev →
    // branch, and iterating params first would instead let whichever appeared
    // leftmost win. `?branch=main&tag=v0.45.0` reported `main`.
    for key in ["tag", "rev", "branch"] {
        for param in params.split('&') {
            if let Some(value) = param.strip_prefix(key).and_then(|r| r.strip_prefix('=')) {
                let value = value.strip_prefix('v').unwrap_or(value);
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// The resolved commit sha from a git locator's `#` fragment.
///
/// The fallback for a dependency pinned to a default branch — `git = "..."`
/// with no `tag`/`rev`/`branch`. Cargo still records the exact commit it
/// resolved, so there is a true answer available; without this the entry would
/// fall through to the stale manifest version and confidently report `0.33.0`.
fn git_sha(source: &str) -> Option<String> {
    let (_, sha) = source.split_once('#')?;
    (!sha.is_empty()).then(|| sha.to_string())
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

    // Review finding: Cargo writes `[[patch.unused]]` stanzas for patches it
    // DISCARDED, carrying the same name/version/source keys. Resetting only on
    // the literal `[[package]]` line let one hijack the parse — with no real
    // stanza at all it reported the discarded patch's version as the linked
    // engine.
    #[test]
    fn a_discarded_patch_stanza_is_not_the_linked_engine() {
        let lock = "[[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n\
             source = \"git+https://x?tag=v0.45.0#abc\"\n\n\
             [[patch.unused]]\nname = \"quire-rs\"\nversion = \"0.9.9\"\n\
             source = \"git+https://x?tag=v0.9.9#def\"\n";
        assert_eq!(engine_version(lock).as_deref(), Some("0.45.0"));

        // ...and with no real stanza, a discarded patch is not an answer.
        let only_patch = "[[package]]\nname = \"serde\"\nversion = \"1\"\n\n\
             [[patch.unused]]\nname = \"quire-rs\"\nversion = \"0.9.9\"\n\
             source = \"git+https://x?tag=v0.9.9#def\"\n";
        assert_eq!(engine_version(only_patch), None);
    }

    // Review finding: `deny.toml` allows multiple versions, and Cargo sorts
    // stanzas by name then version — so taking the first deterministically
    // reported the LOWEST. The git-sourced entry is the one this crate pins.
    #[test]
    fn the_git_sourced_stanza_wins_over_a_second_registry_copy() {
        let lock = "[[package]]\nname = \"quire-rs\"\nversion = \"0.10.0\"\n\
             source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\n\
             [[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n\
             source = \"git+https://x?tag=v0.45.0#abc\"\n";
        assert_eq!(engine_version(lock).as_deref(), Some("0.45.0"));
    }

    // Review finding: a git dep with no tag/rev/branch fell through to the
    // manifest version — reporting `0.33.0`, the stale constant this module
    // exists to avoid. Cargo still records the resolved commit, so there IS a
    // true answer.
    #[test]
    fn a_default_branch_git_dep_reports_its_resolved_sha_not_the_stale_manifest() {
        let lock = "[[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n\
             source = \"git+https://github.com/agent-ix/quire-rs#99e97f013d858ff678b7c0783c1998703c268d71\"\n";
        assert_eq!(
            engine_version(lock).as_deref(),
            Some("99e97f013d858ff678b7c0783c1998703c268d71"),
        );
    }

    // Review finding: params were iterated outer and keys inner, so the
    // leftmost PARAM won rather than the documented tag → rev → branch
    // preference. `?branch=main&tag=v0.45.0` reported `main`.
    #[test]
    fn the_tag_is_preferred_over_a_branch_regardless_of_order() {
        let lock = "[[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n\
             source = \"git+https://x?branch=main&tag=v0.45.0#abc\"\n";
        assert_eq!(engine_version(lock).as_deref(), Some("0.45.0"));
    }

    // An empty locator (`?tag=#sha`) must not yield an empty version — an
    // empty string satisfies `contains()` unconditionally downstream, which is
    // how a "names both versions" assertion becomes decorative.
    #[test]
    fn an_empty_locator_falls_through_rather_than_reporting_nothing() {
        let lock = "[[package]]\nname = \"quire-rs\"\nversion = \"0.33.0\"\n\
             source = \"git+https://x?tag=#99e97f0\"\n";
        assert_eq!(engine_version(lock).as_deref(), Some("99e97f0"));
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
