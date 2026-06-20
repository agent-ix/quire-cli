//! Self-update for a CLI distributed as a static binary.
//!
//! **This module is deliberately package-agnostic.** It has no dependency on
//! `quire`'s I/O layer, command context, or package identity — everything a
//! concrete CLI needs to supply arrives through [`SelfUpdateConfig`]. The
//! intent is that when the IX CLIs are ported to Rust behind a shared CLI kit
//! crate, this whole file moves there verbatim and only the thin command
//! wrapper (`commands/update.rs`) is rewritten per binary.
//!
//! The binary can arrive through more than one install channel, so a generic
//! self-update must first work out *how it was installed* and then drive the
//! matching package manager. We never diff the running binary's version
//! against the registry: a CLI's npm wrapper and its Cargo crate version are
//! independently numbered, so a naive compare is meaningless. Instead we hand
//! idempotency to npm/cargo, which already no-op when up to date.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

/// How the running binary was placed on disk. Determines which package manager
/// (if any) can upgrade it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallSource {
    /// Installed via npm (prebuilt-binary wrapper); the executable lives under
    /// a `node_modules` tree.
    Npm,
    /// Installed via `cargo install`; the executable lives under `~/.cargo`.
    Cargo,
    /// Some other placement (prebuilt tarball dropped on `$PATH`, a copied
    /// binary, a dev build). We cannot safely guess an upgrade command.
    Unknown,
}

/// The only CLI-specific data the generic engine needs. A consuming binary
/// supplies these as constants.
pub struct SelfUpdateConfig {
    /// npm package name, e.g. `@agent-ix/quire-cli`.
    pub npm_package: &'static str,
    /// Git repository for `cargo install --git`, e.g.
    /// `https://github.com/agent-ix/quire-cli`.
    pub cargo_git: &'static str,
    /// Releases page surfaced in the manual/Unknown path.
    pub releases_url: &'static str,
}

/// Runtime options parsed from the command line.
pub struct SelfUpdateOpts {
    /// Report availability without installing.
    pub check: bool,
    /// Override the npm registry (npm channel only). When `None`, npm resolves
    /// the package via the ambient config — i.e. however it was installed.
    pub registry: Option<String>,
}

/// npm flags that force `registry` for a scoped/unscoped package. A plain
/// `--registry` is silently ignored for a scoped package when the user's npmrc
/// pins a `@scope:registry`; the scope-specific override is the one npm
/// honors. Returns an empty vec when no override is requested (ambient config
/// resolves the package).
fn registry_args(npm_package: &str, registry: Option<&str>) -> Vec<String> {
    let Some(registry) = registry else {
        return Vec::new();
    };
    if let Some((scope, _)) = npm_package
        .split_once('/')
        .filter(|_| npm_package.starts_with('@'))
    {
        vec![format!("--{scope}:registry={registry}")]
    } else {
        vec!["--registry".to_string(), registry.to_string()]
    }
}

/// What [`run_self_update`] decided/did. Reporting is the caller's job — this
/// keeps the engine free of any I/O-format policy (kit-ready).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `--check` only: no changes made. `latest` carries the npm-reported
    /// version when the source is npm; `None` for the cargo channel (which
    /// tracks a git branch and has no single "latest" to report).
    Checked { latest: Option<String> },
    /// An upgrade command ran to completion.
    Installed,
    /// The install source was unknown; the report carries manual instructions.
    Manual,
}

/// The outcome of a self-update attempt: the detected source, the action
/// taken, and human-readable summary lines for the caller to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfUpdateReport {
    pub source: InstallSource,
    pub action: Action,
    pub messages: Vec<String>,
}

/// Classify an install from the binary's path. Pure and I/O-free so it is unit
/// testable without touching the real executable. Symlinked global bins (npm,
/// pnpm) resolve through `current_exe()` into the store, so a `node_modules`
/// component is still present after resolution. Matching whole path
/// *components* (not substrings) avoids false positives like `.cargo-backup`
/// or a project literally named `node_modules-tools`. npm wins if both appear.
pub fn detect_source(exe_path: &Path) -> InstallSource {
    let mut npm = false;
    let mut cargo = false;
    for component in exe_path.components() {
        if let std::path::Component::Normal(c) = component {
            if c == "node_modules" {
                npm = true;
            } else if c == ".cargo" {
                cargo = true;
            }
        }
    }
    if npm {
        InstallSource::Npm
    } else if cargo {
        InstallSource::Cargo
    } else {
        InstallSource::Unknown
    }
}

/// Detect the install source from the running executable and dispatch the
/// matching upgrade path. Shelling out to npm/cargo (with inherited stdio so
/// their progress is visible) is the one side effect, and it is gated behind
/// `opts.check`.
pub fn run_self_update(cfg: &SelfUpdateConfig, opts: &SelfUpdateOpts) -> Result<SelfUpdateReport> {
    let exe = std::env::current_exe().context("resolving the running executable path")?;
    run_for_source(cfg, opts, detect_source(&exe))
}

/// Source-injected core, split out so tests can exercise every channel without
/// manipulating the real `current_exe()`.
pub fn run_for_source(
    cfg: &SelfUpdateConfig,
    opts: &SelfUpdateOpts,
    source: InstallSource,
) -> Result<SelfUpdateReport> {
    match source {
        InstallSource::Npm => npm_update(cfg, opts),
        InstallSource::Cargo => cargo_update(cfg, opts),
        InstallSource::Unknown => Ok(manual_report(cfg)),
    }
}

fn npm_update(cfg: &SelfUpdateConfig, opts: &SelfUpdateOpts) -> Result<SelfUpdateReport> {
    let reg_args = registry_args(cfg.npm_package, opts.registry.as_deref());

    if opts.check {
        let mut args = vec!["view", cfg.npm_package, "version"];
        args.extend(reg_args.iter().map(String::as_str));
        let latest = run_capture("npm", &args)
            .context("querying the npm registry for the latest version")?;
        let messages = vec![
            format!("installed via npm ({})", cfg.npm_package),
            format!("latest published version: {latest}"),
            "run `quire update` to upgrade".to_string(),
        ];
        return Ok(SelfUpdateReport {
            source: InstallSource::Npm,
            action: Action::Checked {
                latest: Some(latest),
            },
            messages,
        });
    }

    let spec = format!("{}@latest", cfg.npm_package);
    let mut args = vec!["install", "-g", spec.as_str()];
    args.extend(reg_args.iter().map(String::as_str));
    run_inherited("npm", &args).context("running npm install -g to upgrade")?;
    Ok(SelfUpdateReport {
        source: InstallSource::Npm,
        action: Action::Installed,
        messages: vec![format!("upgraded {} via npm", cfg.npm_package)],
    })
}

fn cargo_update(cfg: &SelfUpdateConfig, opts: &SelfUpdateOpts) -> Result<SelfUpdateReport> {
    if opts.check {
        let messages = vec![
            "installed via cargo".to_string(),
            format!(
                "cargo installs track the git default branch ({}); there is no single \
                 published version to compare against",
                cfg.cargo_git
            ),
            "run `quire update` to rebuild from the latest source".to_string(),
        ];
        return Ok(SelfUpdateReport {
            source: InstallSource::Cargo,
            action: Action::Checked { latest: None },
            messages,
        });
    }

    run_inherited("cargo", &["install", "--git", cfg.cargo_git, "--force"])
        .context("running cargo install --git to upgrade")?;
    Ok(SelfUpdateReport {
        source: InstallSource::Cargo,
        action: Action::Installed,
        messages: vec![format!("upgraded from {} via cargo", cfg.cargo_git)],
    })
}

/// Unknown install source: never guess and clobber a binary we did not place.
/// Print how to upgrade and exit successfully.
fn manual_report(cfg: &SelfUpdateConfig) -> SelfUpdateReport {
    let messages = vec![
        "could not determine how this binary was installed".to_string(),
        format!(
            "if installed via npm:   npm install -g {}@latest",
            cfg.npm_package
        ),
        format!(
            "if installed via cargo: cargo install --git {} --force",
            cfg.cargo_git
        ),
        format!("or download the latest release: {}", cfg.releases_url),
    ];
    SelfUpdateReport {
        source: InstallSource::Unknown,
        action: Action::Manual,
        messages,
    }
}

/// Run a command with inherited stdio; non-zero exit is an error.
fn run_inherited(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("spawning `{cmd}` (is it installed and on PATH?)"))?;
    if !status.success() {
        bail!("`{cmd}` exited with {status}");
    }
    Ok(())
}

/// Run a command and capture trimmed stdout; non-zero exit is an error.
fn run_capture(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("spawning `{cmd}` (is it installed and on PATH?)"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`{cmd}` failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CFG: SelfUpdateConfig = SelfUpdateConfig {
        npm_package: "@agent-ix/quire-cli",
        cargo_git: "https://github.com/agent-ix/quire-cli",
        releases_url: "https://github.com/agent-ix/quire-cli/releases",
    };

    #[test]
    fn registry_args_omitted_when_no_override() {
        assert!(registry_args("@agent-ix/quire-cli", None).is_empty());
    }

    #[test]
    fn registry_args_uses_scope_form_for_scoped_package() {
        // A plain --registry is ignored for scoped packages when an npmrc pins
        // a scope registry; the @scope:registry form is what npm honors.
        assert_eq!(
            registry_args("@agent-ix/quire-cli", Some("http://npm.ix/")),
            vec!["--@agent-ix:registry=http://npm.ix/".to_string()]
        );
    }

    #[test]
    fn registry_args_uses_plain_registry_for_unscoped_package() {
        assert_eq!(
            registry_args("some-cli", Some("https://registry.npmjs.org/")),
            vec![
                "--registry".to_string(),
                "https://registry.npmjs.org/".to_string()
            ]
        );
    }

    #[test]
    fn detect_npm_from_node_modules_path() {
        let p =
            Path::new("/home/u/.npm-global/lib/node_modules/@agent-ix/quire-cli-linux-x64/quire");
        assert_eq!(detect_source(p), InstallSource::Npm);
    }

    #[test]
    fn detect_cargo_from_cargo_bin_path() {
        let p = Path::new("/home/u/.cargo/bin/quire");
        assert_eq!(detect_source(p), InstallSource::Cargo);
    }

    #[test]
    fn detect_unknown_from_bare_path() {
        let p = Path::new("/usr/local/bin/quire");
        assert_eq!(detect_source(p), InstallSource::Unknown);
    }

    #[test]
    fn detect_ignores_lookalike_path_components() {
        // Substring matching would misclassify these; component matching does not.
        assert_eq!(
            detect_source(Path::new("/home/u/.cargo-backup/bin/quire")),
            InstallSource::Unknown
        );
        assert_eq!(
            detect_source(Path::new("/opt/node_modules-tools/quire")),
            InstallSource::Unknown
        );
    }

    #[test]
    fn unknown_source_emits_manual_instructions_without_installing() {
        let opts = SelfUpdateOpts {
            check: false,
            registry: None,
        };
        let report = run_for_source(&CFG, &opts, InstallSource::Unknown).unwrap();
        assert_eq!(report.action, Action::Manual);
        assert_eq!(report.source, InstallSource::Unknown);
        // Carries both upgrade recipes and the releases URL.
        let joined = report.messages.join("\n");
        assert!(joined.contains("npm install -g @agent-ix/quire-cli@latest"));
        assert!(joined.contains("cargo install --git"));
        assert!(joined.contains("releases"));
    }

    #[test]
    fn cargo_check_reports_branch_tracking_without_a_version() {
        let opts = SelfUpdateOpts {
            check: true,
            registry: None,
        };
        let report = run_for_source(&CFG, &opts, InstallSource::Cargo).unwrap();
        assert_eq!(report.action, Action::Checked { latest: None });
    }
}
