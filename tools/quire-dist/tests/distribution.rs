use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_dist::{
    package_npm, set_version, verify_binary_version, verify_release, PackageNpm, SetVersion,
    VerifyRelease, TARGETS,
};
use serde_json::{json, Value};
use tempfile::TempDir;

const VERSION: &str = "1.2.3";

struct Fixture {
    _temporary: TempDir,
    root: PathBuf,
    cargo_manifest: PathBuf,
    artifacts: PathBuf,
    output: PathBuf,
    launcher: PathBuf,
    license: PathBuf,
}

fn package_name(platform: &str, arch: &str) -> String {
    format!("@agent-ix/quire-cli-{platform}-{arch}")
}

fn launcher_json(version: &str) -> Value {
    let dependencies = TARGETS
        .iter()
        .map(|target| (package_name(target.platform, target.arch), json!(version)))
        .collect();
    json!({
        "name": "@agent-ix/quire-cli",
        "version": version,
        "description": "test launcher",
        "license": "AGPL-3.0-or-later",
        "type": "commonjs",
        "bin": { "quire": "bin/quire.js" },
        "files": ["bin/", "LICENSE"],
        "engines": { "node": ">=16" },
        "optionalDependencies": Value::Object(dependencies),
        "publishConfig": {
            "registry": "https://registry.npmjs.org/",
            "access": "public"
        }
    })
}

fn write_json(path: &Path, value: &Value) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize fixture JSON");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write fixture JSON");
}

fn binary_bytes(rust_target: &str) -> Vec<u8> {
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
        other => panic!("unhandled fixture target {other}"),
    }
}

fn fixture() -> Fixture {
    let temporary = tempfile::tempdir().expect("temporary fixture");
    let root = temporary.path().to_path_buf();
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
        fs::write(directory.join(target.binary), binary_bytes(target.rust))
            .expect("write artifact binary");
    }
    let launcher = root.join("npm/quire-cli");
    fs::create_dir_all(launcher.join("bin")).expect("create launcher directory");
    write_json(&launcher.join("package.json"), &launcher_json(VERSION));
    fs::write(launcher.join("bin/quire.js"), "// fixture launcher\n")
        .expect("write fixture launcher");
    fs::write(launcher.join("README.md"), "fixture launcher readme\n")
        .expect("write fixture README");
    let license = root.join("LICENSE");
    fs::write(&license, "fixture AGPL license\n").expect("write license");
    let output = root.join("npm/dist");
    Fixture {
        _temporary: temporary,
        root,
        cargo_manifest,
        artifacts,
        output,
        launcher,
        license,
    }
}

fn package(fixture: &Fixture) -> anyhow::Result<()> {
    package_npm(&PackageNpm {
        version: VERSION,
        cargo_manifest: &fixture.cargo_manifest,
        artifacts_dir: &fixture.artifacts,
        output_dir: &fixture.output,
        launcher_dir: &fixture.launcher,
        license: &fixture.license,
    })
}

fn tree_snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, Option<u32>)> {
    fn visit(
        base: &Path,
        directory: &Path,
        snapshot: &mut BTreeMap<PathBuf, (Vec<u8>, Option<u32>)>,
    ) {
        let mut entries: Vec<_> = fs::read_dir(directory)
            .expect("read snapshot directory")
            .map(Result::unwrap)
            .collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if entry.file_type().expect("snapshot file type").is_dir() {
                visit(base, &path, snapshot);
            } else {
                #[cfg(unix)]
                let mode = {
                    use std::os::unix::fs::PermissionsExt;
                    Some(
                        entry
                            .metadata()
                            .expect("snapshot metadata")
                            .permissions()
                            .mode()
                            & 0o777,
                    )
                };
                #[cfg(not(unix))]
                let mode = None;
                snapshot.insert(
                    path.strip_prefix(base)
                        .expect("relative snapshot")
                        .to_path_buf(),
                    (fs::read(path).expect("snapshot bytes"), mode),
                );
            }
        }
    }

    let mut snapshot = BTreeMap::new();
    visit(root, root, &mut snapshot);
    snapshot
}

#[trace("TC-815", "FR-023-AC-1", "FR-022-AC-1", "NFR-008-AC-1")]
#[test]
fn tc815_target_catalog_is_closed_unique_and_exact() {
    let observed: Vec<_> = TARGETS
        .iter()
        .map(|target| (target.rust, target.platform, target.arch, target.binary))
        .collect();
    assert_eq!(
        observed,
        vec![
            ("x86_64-unknown-linux-musl", "linux", "x64", "quire"),
            ("aarch64-unknown-linux-musl", "linux", "arm64", "quire"),
            ("aarch64-apple-darwin", "darwin", "arm64", "quire"),
            ("x86_64-pc-windows-msvc", "win32", "x64", "quire.exe"),
        ]
    );
    let rust: BTreeSet<_> = TARGETS.iter().map(|target| target.rust).collect();
    let hosts: BTreeSet<_> = TARGETS
        .iter()
        .map(|target| (target.platform, target.arch))
        .collect();
    assert_eq!(rust.len(), TARGETS.len());
    assert_eq!(hosts.len(), TARGETS.len());
}

#[trace("TC-816", "FR-023-AC-4", "FR-023-AC-5")]
#[trace("FR-023-AC-6", "FR-023-AC-7", "US-007-AC-4")]
#[test]
fn tc816_package_generation_is_complete_exact_and_deterministic() {
    let fixture = fixture();
    fs::create_dir_all(&fixture.output).expect("create stale output");
    fs::write(fixture.output.join("stale"), "remove me").expect("write stale output");
    package(&fixture).expect("generate packages");
    assert!(!fixture.output.join("stale").exists());

    let expected_dirs: BTreeSet<_> = TARGETS
        .iter()
        .map(|target| format!("quire-cli-{}-{}", target.platform, target.arch))
        .collect();
    let actual_dirs: BTreeSet<_> = fs::read_dir(&fixture.output)
        .expect("read output")
        .map(|entry| {
            entry
                .expect("output entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(actual_dirs, expected_dirs);

    for target in TARGETS {
        let directory = fixture
            .output
            .join(format!("quire-cli-{}-{}", target.platform, target.arch));
        let members: BTreeSet<_> = tree_snapshot(&directory).keys().cloned().collect();
        assert_eq!(
            members,
            BTreeSet::from([
                PathBuf::from("LICENSE"),
                PathBuf::from("package.json"),
                PathBuf::from("bin").join(target.binary),
            ])
        );
        assert_eq!(
            fs::read(directory.join("LICENSE")).expect("package license"),
            fs::read(&fixture.license).expect("root license")
        );
        let manifest: Value =
            serde_json::from_slice(&fs::read(directory.join("package.json")).expect("manifest"))
                .expect("parse manifest");
        assert_eq!(manifest["name"], package_name(target.platform, target.arch));
        assert_eq!(manifest["version"], VERSION);
        assert_eq!(manifest["license"], "AGPL-3.0-or-later");
        assert_eq!(manifest["os"], json!([target.platform]));
        assert_eq!(manifest["cpu"], json!([target.arch]));
        assert_eq!(manifest["publishConfig"]["access"], "public");
        #[cfg(unix)]
        if target.platform != "win32" {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(directory.join("bin").join(target.binary))
                .expect("binary metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o755);
        }
    }

    let first = tree_snapshot(&fixture.root.join("npm"));
    let launcher = read_launcher(&fixture.launcher.join("package.json"));
    assert_eq!(launcher["name"], "@agent-ix/quire-cli");
    assert_eq!(launcher["version"], VERSION);
    assert_eq!(launcher["license"], "AGPL-3.0-or-later");
    assert_eq!(launcher["bin"], json!({ "quire": "bin/quire.js" }));
    assert_eq!(launcher["files"], json!(["bin/", "LICENSE"]));
    let dependencies = launcher["optionalDependencies"]
        .as_object()
        .expect("launcher optional dependencies");
    assert_eq!(dependencies.len(), TARGETS.len());
    assert!(dependencies.values().all(|value| value == VERSION));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(fixture.launcher.join("package.json"))
                .expect("launcher manifest metadata")
                .permissions()
                .mode()
                & 0o777,
            0o644
        );
        assert_eq!(
            fs::metadata(fixture.launcher.join("LICENSE"))
                .expect("launcher license metadata")
                .permissions()
                .mode()
                & 0o777,
            0o644
        );
    }
    package(&fixture).expect("repeat generation");
    assert_eq!(tree_snapshot(&fixture.root.join("npm")), first);
}

fn assert_refusal_preserves(mutator: impl FnOnce(&Fixture)) {
    let fixture = fixture();
    fs::create_dir_all(&fixture.output).expect("create existing output");
    fs::write(fixture.output.join("sentinel"), "old output").expect("write sentinel");
    let before_output = tree_snapshot(&fixture.output);
    let before_launcher = fs::read(fixture.launcher.join("package.json")).expect("launcher before");
    mutator(&fixture);
    assert!(package(&fixture).is_err(), "mutant unexpectedly packaged");
    assert_eq!(tree_snapshot(&fixture.output), before_output);
    assert_eq!(
        fs::read(fixture.launcher.join("package.json")).expect("launcher after"),
        before_launcher
    );
}

#[trace("TC-817", "FR-023-AC-2", "FR-023-AC-3", "FR-023-AC-7")]
#[test]
fn tc817_artifact_inventory_and_identity_mutations_fail_before_output() {
    assert_refusal_preserves(|fixture| {
        fs::remove_dir_all(fixture.artifacts.join(TARGETS[0].rust)).expect("remove target");
    });
    assert_refusal_preserves(|fixture| {
        fs::create_dir(fixture.artifacts.join("unexpected-target")).expect("add target");
    });
    assert_refusal_preserves(|fixture| {
        let directory = fixture.artifacts.join(TARGETS[0].rust);
        fs::rename(directory.join("quire"), directory.join("wrong-name")).expect("rename binary");
    });
    assert_refusal_preserves(|fixture| {
        fs::write(
            fixture.artifacts.join(TARGETS[0].rust).join("quire"),
            b"not an executable",
        )
        .expect("write wrong format");
    });
    assert_refusal_preserves(|fixture| {
        let x64 =
            fs::read(fixture.artifacts.join(TARGETS[0].rust).join("quire")).expect("read x64");
        fs::write(fixture.artifacts.join(TARGETS[1].rust).join("quire"), x64)
            .expect("replace arm with x64");
    });
}

#[trace("TC-818", "FR-024-AC-1", "FR-024-AC-2", "FR-024-AC-3", "FR-024-AC-4")]
#[test]
fn tc818_version_contract_is_typed_complete_and_preflighted() {
    let updated = fixture();
    set_version(&SetVersion {
        version: "2.3.4-beta.1+build.7",
        cargo_manifest: &updated.cargo_manifest,
        launcher_manifest: &updated.launcher.join("package.json"),
    })
    .expect("set valid version");
    let cargo = fs::read_to_string(&updated.cargo_manifest).expect("updated Cargo");
    assert!(cargo.contains("version = \"2.3.4-beta.1+build.7\""));
    let launcher = read_launcher(&updated.launcher.join("package.json"));
    assert_eq!(launcher["version"], "2.3.4-beta.1+build.7");
    assert!(launcher["optionalDependencies"]
        .as_object()
        .expect("optional dependencies")
        .values()
        .all(|value| value == "2.3.4-beta.1+build.7"));

    for mutate in ["invalid-version", "missing-dependency", "extra-dependency"] {
        let fixture = fixture();
        if mutate != "invalid-version" {
            let manifest = fixture.launcher.join("package.json");
            let mut launcher = read_launcher(&manifest);
            let dependencies = launcher["optionalDependencies"]
                .as_object_mut()
                .expect("optional dependencies");
            if mutate == "missing-dependency" {
                dependencies.remove(&package_name(TARGETS[0].platform, TARGETS[0].arch));
            } else {
                dependencies.insert("@agent-ix/quire-cli-extra".to_owned(), json!(VERSION));
            }
            write_json(&manifest, &launcher);
        }
        let cargo_before = fs::read(&fixture.cargo_manifest).expect("Cargo before");
        let launcher_before =
            fs::read(fixture.launcher.join("package.json")).expect("launcher before");
        let requested = if mutate == "invalid-version" {
            "not-semver"
        } else {
            "2.0.0"
        };
        assert!(set_version(&SetVersion {
            version: requested,
            cargo_manifest: &fixture.cargo_manifest,
            launcher_manifest: &fixture.launcher.join("package.json"),
        })
        .is_err());
        assert_eq!(
            fs::read(&fixture.cargo_manifest).expect("Cargo after"),
            cargo_before
        );
        assert_eq!(
            fs::read(fixture.launcher.join("package.json")).expect("launcher after"),
            launcher_before
        );
    }

    let fixture = fixture();
    assert!(verify_release(&VerifyRelease {
        version: "9.9.9",
        cargo_manifest: &fixture.cargo_manifest,
        launcher_manifest: &fixture.launcher.join("package.json"),
    })
    .is_err());
}

fn read_launcher(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read launcher")).expect("parse launcher")
}

#[trace("IT-164", "FR-024-AC-4")]
#[test]
fn it164_native_binary_version_mismatch_fails() {
    let probe = Path::new(env!("CARGO_BIN_EXE_quire-dist-probe"));
    verify_binary_version(VERSION, probe).expect("matching version");
    let error = verify_binary_version("1.2.4", probe).expect_err("mismatch must fail");
    assert!(error.to_string().contains("reports version 1.2.3"));
}
