use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_dist::{
    package_npm, set_version, verify_binary_version, verify_published, verify_release, PackageNpm,
    SetVersion, VerifyPublished, VerifyRelease, LAUNCHER_PACKAGE_NAME, TARGETS,
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
    // A build may cover any non-empty subset of the catalog (TC-822) — but
    // an empty artifacts directory is never a valid release, and is rejected
    // the same as any other malformed input.
    assert_refusal_preserves(|fixture| {
        for target in TARGETS {
            fs::remove_dir_all(fixture.artifacts.join(target.rust)).expect("remove target");
        }
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

#[trace("TC-822", "FR-023-AC-2", "FR-023-AC-4", "FR-023-AC-6")]
#[test]
fn tc822_partial_artifact_set_emits_only_the_built_platforms() {
    // This is the local publish path's real shape (PLAT-885): only
    // linux-x64 is actually built. The generator must emit exactly one
    // package and one launcher dependency for it, and nothing at all for
    // the three platforms it did not build -- not a dangling pin at a
    // version that was never published for them.
    let fixture = fixture();
    for target in TARGETS {
        if target != TARGETS[0] {
            fs::remove_dir_all(fixture.artifacts.join(target.rust)).expect("prune target");
        }
    }
    package(&fixture).expect("partial build must package");

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
    assert_eq!(
        actual_dirs,
        BTreeSet::from([format!(
            "quire-cli-{}-{}",
            TARGETS[0].platform, TARGETS[0].arch
        )])
    );

    let launcher = read_launcher(&fixture.launcher.join("package.json"));
    let dependencies = launcher["optionalDependencies"]
        .as_object()
        .expect("launcher optional dependencies");
    let built_name = package_name(TARGETS[0].platform, TARGETS[0].arch);
    assert_eq!(
        dependencies.keys().cloned().collect::<Vec<_>>(),
        vec![built_name.clone()]
    );
    assert_eq!(dependencies[&built_name], VERSION);
    for target in &TARGETS[1..] {
        assert!(!dependencies.contains_key(&package_name(target.platform, target.arch)));
    }
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

    // A launcher declaring a non-empty proper subset of the catalog (the
    // local publish path's real shape, PLAT-885) is a valid input: it is
    // bumped in place, and the platform it never declared stays absent.
    let partial = fixture();
    {
        let manifest = partial.launcher.join("package.json");
        let mut launcher = read_launcher(&manifest);
        let dependencies = launcher["optionalDependencies"]
            .as_object_mut()
            .expect("optional dependencies");
        dependencies.remove(&package_name(TARGETS[0].platform, TARGETS[0].arch));
        write_json(&manifest, &launcher);
    }
    set_version(&SetVersion {
        version: "2.5.0",
        cargo_manifest: &partial.cargo_manifest,
        launcher_manifest: &partial.launcher.join("package.json"),
    })
    .expect("set version over a non-empty subset");
    let launcher = read_launcher(&partial.launcher.join("package.json"));
    let dependencies = launcher["optionalDependencies"]
        .as_object()
        .expect("optional dependencies");
    assert_eq!(dependencies.len(), TARGETS.len() - 1);
    assert!(dependencies.values().all(|value| value == "2.5.0"));
    assert!(!dependencies.contains_key(&package_name(TARGETS[0].platform, TARGETS[0].arch)));

    for mutate in ["invalid-version", "empty-dependencies", "extra-dependency"] {
        let fixture = fixture();
        if mutate != "invalid-version" {
            let manifest = fixture.launcher.join("package.json");
            let mut launcher = read_launcher(&manifest);
            let dependencies = launcher["optionalDependencies"]
                .as_object_mut()
                .expect("optional dependencies");
            if mutate == "empty-dependencies" {
                dependencies.clear();
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

/// A fake `npm` that answers the two `npm view` shapes `verify_published`
/// actually issues:
///
/// - `view <spec> version <registry_arg>` (one platform package)
/// - `view <spec> --json <registry_arg>` (the launcher package, whose
///   response is read from a sibling `published-manifest.json` fixture so
///   each sub-case below can vary it)
///
/// `registry_arg` is asserted verbatim against `expected_registry_override`
/// and the stub fails, loudly, if it is wrong or absent — this is the fix
/// for the reviewer's finding: a stub that answers regardless of the
/// registry argument tests nothing about the registry argument. Deleting
/// `&registry_override` from either call site in `lib.rs` now fails every
/// "should succeed" case below immediately, not just a dedicated one, since
/// every call this test makes goes through the same assertion.
///
/// A version containing `missing` is treated as a 404 (exit 1, npm-shaped
/// stderr) in both modes -- measured directly against npm 10.9.2 and the
/// CI-pinned 11.6.2 on both npm.ix and public npm, a genuinely absent
/// version reliably exits non-zero with `E404`, never exit 0 with empty
/// stdout, on either registry. A version containing `garbled` echoes a
/// version that disagrees with what was asked for, exit 0 -- the shape
/// `verify_package_resolves`'s `resolved != version` guard exists for.
///
/// `#[cfg(unix)]`: this and `it165_...resolved_packages` below are the
/// only tests that exercise `verify_published`'s stub-backed hermetic path
/// through a `sh` script, so Windows carries zero coverage of it. Accepted
/// deliberately, not by omission: `tests/npm_host.rs` already takes the
/// same `#![cfg(unix)]` stance file-wide for the identical reason (a shell
/// stub host), and `make ci`/this repo's CI never run `cargo test` on
/// Windows at all -- the only Windows leg is `release.yml`'s
/// `x86_64-pc-windows-msvc` *build* matrix entry, which never runs the test
/// suite. The real-network `it165_unreachable_registry_...` case below is
/// not `#[cfg(unix)]` and does run cross-platform.
#[cfg(unix)]
fn write_stub_npm(directory: &Path, expected_registry_override: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = directory.join("npm");
    let script = format!(
        "#!/bin/sh\n\
         here=\"$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\"\n\
         spec=\"$2\"\n\
         mode=\"$3\"\n\
         registry_arg=\"$4\"\n\
         if [ \"$registry_arg\" != {expected_registry_override:?} ]; then\n\
         \x20\x20echo \"npm error: unexpected or missing registry override '$registry_arg'\" >&2\n\
         \x20\x20exit 1\n\
         fi\n\
         version=\"${{spec##*@}}\"\n\
         case \"$mode\" in\n\
         \x20\x20--json)\n\
         \x20\x20\x20\x20case \"$version\" in\n\
         \x20\x20\x20\x20\x20\x20*missing*) echo 'npm error code E404' >&2; exit 1 ;;\n\
         \x20\x20\x20\x20\x20\x20*) cat \"$here/published-manifest.json\" ;;\n\
         \x20\x20\x20\x20esac\n\
         \x20\x20\x20\x20;;\n\
         \x20\x20version)\n\
         \x20\x20\x20\x20case \"$version\" in\n\
         \x20\x20\x20\x20\x20\x20*missing*) echo 'npm error code E404' >&2; exit 1 ;;\n\
         \x20\x20\x20\x20\x20\x20*garbled*) echo 'not-the-requested-version' ;;\n\
         \x20\x20\x20\x20\x20\x20*) echo \"$version\" ;;\n\
         \x20\x20\x20\x20esac\n\
         \x20\x20\x20\x20;;\n\
         \x20\x20*)\n\
         \x20\x20\x20\x20echo \"npm error: unknown query mode '$mode'\" >&2\n\
         \x20\x20\x20\x20exit 1\n\
         \x20\x20\x20\x20;;\n\
         esac\n"
    );
    fs::write(&path, script).expect("write stub npm");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod stub npm");
    path
}

#[cfg(unix)]
#[trace("IT-165", "FR-023-AC-9")]
#[test]
fn it165_publish_resolvability_gate_fails_closed_and_passes_on_resolved_packages() {
    let registry = "http://npm.ix/";
    let expected_registry_override = "--@agent-ix:registry=http://npm.ix/";
    let stub_dir = tempfile::tempdir().expect("stub npm directory");
    let npm = write_stub_npm(stub_dir.path(), expected_registry_override);
    let manifest_fixture = stub_dir.path().join("published-manifest.json");

    let write_published_manifest = |version: &str, dependencies: Value| {
        write_json(
            &manifest_fixture,
            &json!({ "version": version, "optionalDependencies": dependencies }),
        );
    };
    let verify = |version: &str| {
        verify_published(&VerifyPublished {
            version,
            registry,
            npm_binary: npm.as_os_str(),
        })
    };

    // The launcher itself, and every declared platform package, resolves at
    // its pinned version: success. This is also the mutation-sensitivity
    // check for the registry argument: if `lib.rs` stopped passing
    // `--@agent-ix:registry=…`, the stub above would refuse every call the
    // launcher-manifest fetch makes, and this assertion would fail first.
    write_published_manifest(
        "0.32.3",
        json!({ "@agent-ix/quire-cli-linux-x64": "0.32.3" }),
    );
    verify("0.32.3").expect("launcher and platform package both resolve");

    // The launcher package itself does not resolve at the requested version
    // (its own `npm publish` failed after the platform packages already
    // succeeded, or nothing was published at all) -- must fail, naming the
    // launcher, never silently falling back to checking only the platform
    // packages a local file happened to list.
    let error = verify("0.32.3-missing").expect_err("unresolved launcher must fail");
    assert!(error
        .to_string()
        .contains(&format!("{LAUNCHER_PACKAGE_NAME}@0.32.3-missing")));

    // One declared platform package does not exist at its pinned version:
    // the exact PLAT-885 shape (three dangling platform pins alongside one
    // real one), and it must fail, naming the unresolved package.
    write_published_manifest(
        "0.32.3",
        json!({
            "@agent-ix/quire-cli-linux-x64": "0.32.3",
            "@agent-ix/quire-cli-darwin-arm64": "0.32.3-missing",
        }),
    );
    let error = verify("0.32.3").expect_err("unresolved platform package must fail");
    assert!(error
        .to_string()
        .contains("@agent-ix/quire-cli-darwin-arm64@0.32.3-missing"));

    // `npm view` can exit 0 while printing something other than what was
    // asked for -- defend against trusting a non-empty stdout as proof on
    // its own.
    write_published_manifest(
        "0.32.3",
        json!({ "@agent-ix/quire-cli-linux-x64": "0.32.3-garbled" }),
    );
    let error = verify("0.32.3").expect_err("a printed version that disagrees must fail");
    assert!(error
        .to_string()
        .contains("did not resolve to \"0.32.3-garbled\""));

    // Empty optionalDependencies (or the field absent entirely) must fail
    // outright, not pass by iterating zero entries -- that silent success
    // is the defect class this gate exists to catch (PLAT-885).
    write_published_manifest("0.32.3", json!({}));
    let error = verify("0.32.3").expect_err("empty dependency set must fail");
    assert!(error
        .to_string()
        .contains("must declare at least one platform package"));
}

/// The unreachable-registry half of FR-023-AC-9, against the real `npm`
/// binary rather than the hermetic stub: a host that cannot even resolve
/// must fail the gate. This is precisely the scenario finding 1 measured as
/// silently passing under the original `--registry` flag (npm resolved the
/// scope against this host's own default and never consulted the flag).
/// Network-dependent, so it is `#[ignore]`d out of the default offline
/// suite rather than skipping silently or flaking under a blocked host.
#[trace("IT-165", "FR-023-AC-9")]
#[test]
#[ignore = "hits the real npm binary and a non-resolving network address; run explicitly with `cargo test -- --ignored`"]
fn it165_unreachable_registry_fails_against_real_npm() {
    verify_published(&VerifyPublished {
        version: "0.32.3",
        registry: "http://does-not-exist.invalid/",
        npm_binary: OsStr::new("npm"),
    })
    .expect_err("an unreachable registry must fail the gate, not resolve against some default");
}
