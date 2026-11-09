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

/// Process-boundary tests for the npm channel. A copy of the real `quire`
/// binary is placed under a `node_modules` tree so the detector classifies it
/// as npm, and `npm` on `PATH` is a copy of a stock coreutils program: `echo`
/// reports exactly the argument vector `update` passed, `false` fails. No
/// network is touched and no shell runs.
#[cfg(unix)]
mod npm_channel {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};

    use ix_trace_rs::trace;
    use tempfile::TempDir;

    fn stock_program(name: &str) -> PathBuf {
        ["/bin", "/usr/bin"]
            .iter()
            .map(|dir| Path::new(dir).join(name))
            .find(|path| path.is_file())
            .unwrap_or_else(|| panic!("stock `{name}` is installed"))
    }

    /// Run `quire update <args>` from an npm-classified install whose `npm`
    /// is the stock program `npm_stand_in`, or absent from `PATH` when `None`.
    fn update_with_npm(npm_stand_in: Option<&str>, args: &[&str]) -> Output {
        let dir = TempDir::new().expect("tempdir");
        let bin_dir = dir.path().join("node_modules/@agent-ix/quire-cli/bin");
        fs::create_dir_all(&bin_dir).expect("npm tree");
        let quire = bin_dir.join("quire");
        fs::copy(env!("CARGO_BIN_EXE_quire"), &quire).expect("copy quire");
        fs::set_permissions(&quire, fs::Permissions::from_mode(0o755)).expect("chmod");

        let path_dir = dir.path().join("path");
        fs::create_dir_all(&path_dir).expect("path dir");
        if let Some(name) = npm_stand_in {
            fs::copy(stock_program(name), path_dir.join("npm")).expect("copy npm stand-in");
        }

        Command::new(&quire)
            .arg("update")
            .args(args)
            .env("PATH", &path_dir)
            .output()
            .expect("update runs")
    }

    #[test]
    #[trace("FR-016-AC-3", "FR-016-AC-4")]
    fn check_queries_the_scoped_package_version_and_reports_without_installing() {
        let out = update_with_npm(Some("echo"), &["--check"]);
        assert!(out.status.success(), "{out:?}");
        let stdout = String::from_utf8(out.stdout).expect("UTF-8");
        assert!(
            stdout.contains("latest published version: view @agent-ix/quire-cli version\n"),
            "no registry flag without an override, no install verb: {stdout}"
        );
        assert!(
            !stdout.contains("install -g"),
            "--check must not install: {stdout}"
        );
    }

    #[test]
    #[trace("FR-016-AC-4")]
    fn registry_override_uses_the_scope_form_not_a_bare_registry_flag() {
        let out = update_with_npm(
            Some("echo"),
            &["--check", "--registry", "https://r.example/npm"],
        );
        assert!(out.status.success(), "{out:?}");
        let stdout = String::from_utf8(out.stdout).expect("UTF-8");
        assert!(
            stdout.contains(
                "latest published version: view @agent-ix/quire-cli version \
                 --@agent-ix:registry=https://r.example/npm\n"
            ),
            "{stdout}"
        );
        assert!(!stdout.contains("--registry"), "{stdout}");
    }

    #[test]
    #[trace("FR-016-AC-7")]
    fn failing_npm_exits_four_for_check_and_for_install() {
        for args in [&["--check"][..], &[][..]] {
            let out = update_with_npm(Some("false"), args);
            assert_eq!(out.status.code(), Some(4), "{args:?}: {out:?}");
            assert!(out.stdout.is_empty(), "{args:?}: {out:?}");
            assert!(!out.stderr.is_empty(), "{args:?}: {out:?}");
        }
    }

    #[test]
    #[trace("FR-016-AC-7")]
    fn unreachable_npm_exits_four() {
        let out = update_with_npm(None, &["--check"]);
        assert_eq!(out.status.code(), Some(4), "{out:?}");
        assert!(out.stdout.is_empty(), "{out:?}");
    }
}
