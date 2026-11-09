//! `quire update` end-to-end behavior at the process boundary.
//!
//! The test binary lives under `target/<profile>/` — neither a `node_modules`
//! tree nor `~/.cargo` — so the install-source detector resolves to `Unknown`.
//! On the Unknown path `update` must print manual upgrade instructions and exit
//! 0 WITHOUT shelling out to npm/cargo (so the test never touches the network).

mod common;

use common::quire;
use ix_trace_rs::trace;

// IT-083, FR-016-AC-1, FR-016-AC-2: `update --check` on an Unknown install
// source prints the npm recipe, the cargo recipe and the releases URL, exits 0,
// and performs no install or network call.
#[trace("IT-083", "FR-016-AC-1", "FR-016-AC-2")]
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
#[trace("IT-084", "FR-016-AC-2")]
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

/// Process-boundary tests for the npm and cargo channels. A copy of the real
/// `quire` binary is placed under a `node_modules` (npm) or `.cargo` (cargo)
/// tree so the detector classifies it, and the package manager on `PATH` is a
/// tiny `/bin/sh` script that either prints its arguments or fails. No network
/// is touched.
#[cfg(unix)]
mod package_manager_channels {
    use std::fs;
    use std::io::ErrorKind;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::{Command, Output};
    use std::sync::Mutex;
    use std::thread::sleep;
    use std::time::Duration;

    use ix_trace_rs::trace;
    use tempfile::TempDir;

    /// What the stand-in package manager does.
    enum Stub {
        /// Print its arguments on one line, exit 0.
        Echo,
        /// Exit non-zero.
        Fail,
        /// Not on `PATH` at all.
        Absent,
    }

    /// Writing an executable while sibling test threads fork can leave the
    /// file open in a child at `execve` time (rust-lang/rust#114554), so file
    /// creation and the run that follows are serialized.
    static EXEC_LOCK: Mutex<()> = Mutex::new(());

    fn write_executable(path: &Path, contents: &[u8]) {
        fs::write(path, contents).expect("write executable");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    fn spawn_retrying(command: &mut Command) -> Output {
        for _ in 0..50 {
            match command.output() {
                Err(error) if error.kind() == ErrorKind::ExecutableFileBusy => {
                    sleep(Duration::from_millis(20));
                }
                result => return result.expect("update runs"),
            }
        }
        panic!("executable stayed busy");
    }

    /// Run `quire update <args>` from an install under `<channel_dir>/…`
    /// (`node_modules` or `.cargo`) whose package manager is `stub`.
    fn update_with(channel_dir: &str, manager: &str, stub: &Stub, args: &[&str]) -> Output {
        let _serialized = EXEC_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = TempDir::new().expect("tempdir");
        let bin_dir = dir.path().join(channel_dir).join("bin");
        fs::create_dir_all(&bin_dir).expect("install tree");
        let quire = bin_dir.join("quire");
        write_executable(
            &quire,
            &fs::read(env!("CARGO_BIN_EXE_quire")).expect("read quire"),
        );

        let path_dir = dir.path().join("path");
        fs::create_dir_all(&path_dir).expect("path dir");
        match stub {
            Stub::Echo => write_executable(&path_dir.join(manager), b"#!/bin/sh\necho \"$@\"\n"),
            Stub::Fail => write_executable(&path_dir.join(manager), b"#!/bin/sh\nexit 1\n"),
            Stub::Absent => {}
        }

        spawn_retrying(
            Command::new(&quire)
                .arg("update")
                .args(args)
                .env("PATH", &path_dir),
        )
    }

    fn update_with_npm(stub: &Stub, args: &[&str]) -> Output {
        update_with("node_modules/@agent-ix/quire-cli", "npm", stub, args)
    }

    #[test]
    #[trace("FR-016-AC-3", "FR-016-AC-4")]
    fn check_queries_the_scoped_package_version_and_reports_without_installing() {
        let out = update_with_npm(&Stub::Echo, &["--check"]);
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
            &Stub::Echo,
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
            let out = update_with_npm(&Stub::Fail, args);
            assert_eq!(out.status.code(), Some(4), "{args:?}: {out:?}");
            assert!(out.stdout.is_empty(), "{args:?}: {out:?}");
            assert!(!out.stderr.is_empty(), "{args:?}: {out:?}");
        }
    }

    #[test]
    #[trace("FR-016-AC-7")]
    fn unreachable_npm_exits_four() {
        let out = update_with_npm(&Stub::Absent, &["--check"]);
        assert_eq!(out.status.code(), Some(4), "{out:?}");
        assert!(out.stdout.is_empty(), "{out:?}");
    }

    #[test]
    #[trace("FR-016-AC-7")]
    fn failing_or_missing_cargo_exits_four() {
        for stub in [Stub::Fail, Stub::Absent] {
            let out = update_with(".cargo", "cargo", &stub, &[]);
            assert_eq!(out.status.code(), Some(4), "{out:?}");
            assert!(out.stdout.is_empty(), "{out:?}");
            assert!(!out.stderr.is_empty(), "{out:?}");
        }
    }
}
