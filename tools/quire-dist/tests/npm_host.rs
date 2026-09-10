#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ix_trace_rs::trace;
use quire_dist::{package_npm, DistributionTarget, PackageNpm, TARGETS};
use serde_json::{json, Map, Value};

const VERSION: &str = "1.2.3";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("quire-cli repository root")
        .to_path_buf()
}

fn host_target() -> Option<DistributionTarget> {
    let (platform, arch) = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => ("linux", "x64"),
        ("linux", "aarch64") => ("linux", "arm64"),
        ("macos", "aarch64") => ("darwin", "arm64"),
        _ => return None,
    };
    TARGETS
        .iter()
        .copied()
        .find(|target| target.platform == platform && target.arch == arch)
}

fn package_name(target: &DistributionTarget) -> String {
    format!("@agent-ix/quire-cli-{}-{}", target.platform, target.arch)
}

fn valid_binary_bytes(rust_target: &str) -> Vec<u8> {
    match rust_target {
        "x86_64-unknown-linux-musl" => {
            let mut bytes = vec![0; 64];
            bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
            bytes[18..20].copy_from_slice(&62_u16.to_le_bytes());
            bytes
        }
        "aarch64-unknown-linux-musl" => {
            let mut bytes = vec![0; 64];
            bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
            bytes[18..20].copy_from_slice(&183_u16.to_le_bytes());
            bytes
        }
        "aarch64-apple-darwin" => {
            let mut bytes = vec![0; 32];
            bytes[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
            bytes[4..8].copy_from_slice(&0x0100_000c_u32.to_le_bytes());
            bytes
        }
        "x86_64-pc-windows-msvc" => {
            let mut bytes = vec![0; 128];
            bytes[..2].copy_from_slice(b"MZ");
            bytes[0x3c..0x40].copy_from_slice(&64_u32.to_le_bytes());
            bytes[64..68].copy_from_slice(b"PE\0\0");
            bytes[68..70].copy_from_slice(&0x8664_u16.to_le_bytes());
            bytes
        }
        other => panic!("unhandled target {other}"),
    }
}

fn launcher_manifest(targets: impl IntoIterator<Item = DistributionTarget>) -> Value {
    let dependencies: Map<_, _> = targets
        .into_iter()
        .map(|target| (package_name(&target), json!(VERSION)))
        .collect();
    json!({
        "name": "@agent-ix/quire-cli",
        "version": VERSION,
        "description": "native launcher fixture",
        "license": "AGPL-3.0-or-later",
        "type": "commonjs",
        "bin": { "quire": "bin/quire.js" },
        "files": ["bin/", "LICENSE"],
        "engines": { "node": ">=16" },
        "optionalDependencies": dependencies,
        "publishConfig": {
            "registry": "https://registry.npmjs.org/",
            "access": "public"
        }
    })
}

fn write_json(path: &Path, value: &Value) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize JSON");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write JSON");
}

fn prepare_packages() -> (tempfile::TempDir, PathBuf, DistributionTarget) {
    let host = host_target().expect("local host must be an admitted npm target");
    let temporary = tempfile::tempdir().expect("temporary package fixture");
    let root = temporary.path();
    let cargo_manifest = root.join("Cargo.toml");
    fs::write(
        &cargo_manifest,
        format!("[package]\nname = \"fixture\"\nversion = \"{VERSION}\"\n"),
    )
    .expect("write Cargo manifest");
    let artifacts = root.join("artifacts");
    for target in TARGETS {
        let directory = artifacts.join(target.rust);
        fs::create_dir_all(&directory).expect("create artifact directory");
        if target == host {
            fs::copy(
                env!("CARGO_BIN_EXE_quire-dist-probe"),
                directory.join(target.binary),
            )
            .expect("copy native probe");
        } else {
            fs::write(
                directory.join(target.binary),
                valid_binary_bytes(target.rust),
            )
            .expect("write synthetic cross-target binary");
        }
    }
    let launcher = root.join("npm/quire-cli");
    fs::create_dir_all(launcher.join("bin")).expect("create launcher");
    fs::copy(
        repo_root().join("npm/quire-cli/bin/quire.js"),
        launcher.join("bin/quire.js"),
    )
    .expect("copy launcher host");
    fs::copy(
        repo_root().join("npm/quire-cli/README.md"),
        launcher.join("README.md"),
    )
    .expect("copy launcher README");
    write_json(&launcher.join("package.json"), &launcher_manifest(TARGETS));
    let license = root.join("LICENSE");
    fs::write(&license, "fixture AGPL license\n").expect("write license");
    let output = root.join("npm/dist");
    package_npm(&PackageNpm {
        version: VERSION,
        cargo_manifest: &cargo_manifest,
        artifacts_dir: &artifacts,
        output_dir: &output,
        launcher_dir: &launcher,
        license: &license,
    })
    .expect("generate npm packages");
    (temporary, output, host)
}

fn npm_pack(directory: &Path, destination: &Path) -> PathBuf {
    let output = Command::new("npm")
        .args([
            "pack",
            "--offline",
            "--ignore-scripts",
            "--json",
            "--pack-destination",
        ])
        .arg(destination)
        .current_dir(directory)
        .output()
        .expect("run npm pack");
    assert!(
        output.status.success(),
        "npm pack failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("parse npm pack report");
    let filename = report[0]["filename"].as_str().expect("pack filename");
    destination.join(filename)
}

#[trace("IT-159", "US-007-AC-1", "US-007-AC-2")]
#[trace("FR-022-AC-1", "FR-022-AC-2")]
#[trace("FR-023-AC-8", "NFR-008-AC-5")]
#[test]
fn it_159_clean_offline_install_runs_the_matching_native_binary_transparently() {
    let (temporary, output, host) = prepare_packages();
    let tarballs = temporary.path().join("tarballs");
    fs::create_dir(&tarballs).expect("create tarball directory");
    let platform = TARGETS
        .iter()
        .map(|target| {
            (
                *target,
                npm_pack(
                    &output.join(format!("quire-cli-{}-{}", target.platform, target.arch)),
                    &tarballs,
                ),
            )
        })
        .find_map(|(target, tarball)| (target == host).then_some(tarball))
        .expect("matching host tarball");
    let launcher = npm_pack(&temporary.path().join("npm/quire-cli"), &tarballs);
    let consumer = temporary.path().join("consumer");
    fs::create_dir(&consumer).expect("create consumer");
    let install = Command::new("npm")
        .args([
            "install",
            "--offline",
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
            "--package-lock=false",
        ])
        .arg(&launcher)
        .arg(&platform)
        .current_dir(&consumer)
        .output()
        .expect("run offline npm install");
    assert!(
        install.status.success(),
        "offline npm install failed: {}\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    let mut child = Command::new(consumer.join("node_modules/.bin/quire"))
        .args(["alpha", "two words", "--exit", "23"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn installed launcher");
    child
        .stdin
        .take()
        .expect("launcher stdin")
        .write_all(b"consumer-input")
        .expect("write launcher stdin");
    let output = child.wait_with_output().expect("wait for launcher");
    assert_eq!(output.status.code(), Some(23));
    let stdout = String::from_utf8(output.stdout).expect("probe stdout");
    assert!(stdout.contains("args=[\"alpha\",\"two words\",\"--exit\",\"23\"]"));
    assert!(stdout.contains("stdin=consumer-input"));
    assert_eq!(
        String::from_utf8(output.stderr).expect("probe stderr"),
        "probe-stderr\n"
    );
}

#[trace("IT-163", "FR-023-AC-8", "US-007-AC-4", "NFR-008-AC-5", "NFR-008-AC-6")]
#[test]
fn it163_npm_pack_reports_exact_members_and_licenses_offline() {
    let (temporary, output, _) = prepare_packages();
    let mut packages = vec![temporary.path().join("npm/quire-cli")];
    packages.extend(
        TARGETS
            .iter()
            .map(|target| output.join(format!("quire-cli-{}-{}", target.platform, target.arch))),
    );

    for package in packages {
        let output = Command::new("npm")
            .args([
                "pack",
                "--offline",
                "--ignore-scripts",
                "--json",
                "--dry-run",
            ])
            .current_dir(&package)
            .output()
            .expect("run npm pack dry-run");
        assert!(output.status.success(), "npm pack dry-run failed");
        let report: Value = serde_json::from_slice(&output.stdout).expect("parse pack report");
        let files: BTreeMap<_, _> = report[0]["files"]
            .as_array()
            .expect("pack files")
            .iter()
            .map(|file| {
                (
                    file["path"].as_str().expect("file path"),
                    file["size"].as_u64().expect("file size"),
                )
            })
            .collect();
        assert_eq!(
            files.keys().copied().collect::<Vec<_>>(),
            if package.ends_with("quire-cli") {
                vec!["LICENSE", "README.md", "bin/quire.js", "package.json"]
            } else if package.ends_with("quire-cli-win32-x64") {
                vec!["LICENSE", "bin/quire.exe", "package.json"]
            } else {
                vec!["LICENSE", "bin/quire", "package.json"]
            }
        );
        assert_eq!(
            fs::read(package.join("LICENSE")).expect("package license"),
            b"fixture AGPL license\n"
        );
    }
}

fn launcher_tree(targets: impl IntoIterator<Item = DistributionTarget>) -> tempfile::TempDir {
    let temporary = tempfile::tempdir().expect("temporary launcher tree");
    let launcher = temporary.path().join("node_modules/@agent-ix/quire-cli");
    fs::create_dir_all(launcher.join("bin")).expect("create launcher tree");
    fs::copy(
        repo_root().join("npm/quire-cli/bin/quire.js"),
        launcher.join("bin/quire.js"),
    )
    .expect("copy launcher");
    write_json(&launcher.join("package.json"), &launcher_manifest(targets));
    temporary
}

fn run_launcher(tree: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new("node")
        .arg(tree.join("node_modules/@agent-ix/quire-cli/bin/quire.js"))
        .args(arguments)
        .output()
        .expect("run Node launcher")
}

#[trace("IT-160", "US-007-AC-3", "FR-022-AC-4", "FR-022-AC-5")]
#[test]
fn it160_unsupported_and_missing_packages_fail_without_substitution() {
    let host = host_target().expect("admitted host");
    let unsupported = launcher_tree(TARGETS.into_iter().filter(|target| *target != host));
    let output = run_launcher(unsupported.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("unsupported stderr");
    assert!(stderr.contains("unsupported platform"));
    assert!(stderr.contains("Build from source instead"));
    assert!(!stderr.contains(&format!("{}-{}.", host.platform, host.arch)));

    let missing = launcher_tree(TARGETS);
    let output = run_launcher(missing.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("missing stderr");
    assert!(stderr.contains(&package_name(&host)));
    assert!(stderr.contains("optional dependency"));
}

fn install_host_link(tree: &Path, target: &DistributionTarget, source: &Path) -> PathBuf {
    let binary = tree
        .join("node_modules")
        .join(package_name(target))
        .join("bin")
        .join(target.binary);
    fs::create_dir_all(binary.parent().expect("binary parent")).expect("create host package");
    symlink(source, &binary).expect("link host binary");
    binary
}

#[trace("IT-161", "FR-022-AC-6")]
#[test]
fn it161_chmod_refusal_is_tolerated_and_spawn_failure_is_named() {
    let host = host_target().expect("admitted host");
    let success = launcher_tree(TARGETS);
    install_host_link(success.path(), &host, Path::new("/bin/true"));
    let output = run_launcher(success.path(), &[]);
    assert!(
        output.status.success(),
        "root-owned executable must still launch"
    );

    let failure = launcher_tree(TARGETS);
    install_host_link(failure.path(), &host, Path::new("/etc/hosts"));
    let output = run_launcher(failure.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stderr)
        .expect("launch failure stderr")
        .contains("failed to launch binary"));
}

#[trace("IT-162", "US-007-AC-2", "FR-022-AC-3")]
#[test]
fn it162_child_signal_is_mirrored_by_the_launcher() {
    let host = host_target().expect("admitted host");
    let fixture = launcher_tree(TARGETS);
    install_host_link(fixture.path(), &host, Path::new("/bin/sh"));
    let output = run_launcher(fixture.path(), &["-c", "kill -ABRT $$"]);
    assert_eq!(output.status.signal(), Some(6));
}
