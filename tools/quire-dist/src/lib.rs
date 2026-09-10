//! Native release and npm distribution tooling for `quire-cli`.
//!
//! This crate deliberately has no dependency on `quire-rs`: distribution
//! treats every native executable as an opaque, target-identified artifact.

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use semver::Version;
use serde_json::{json, Map, Value};
use tempfile::{Builder, NamedTempFile};
use toml_edit::{value, DocumentMut};

/// One admitted native/npm distribution target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DistributionTarget {
    pub rust: &'static str,
    pub platform: &'static str,
    pub arch: &'static str,
    pub binary: &'static str,
    format: BinaryFormat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BinaryFormat {
    Elf { machine: u16 },
    MachO { cpu_type: u32 },
    Pe { machine: u16 },
}

/// The sole production catalog for native release and npm package targets.
pub const TARGETS: [DistributionTarget; 4] = [
    DistributionTarget {
        rust: "x86_64-unknown-linux-musl",
        platform: "linux",
        arch: "x64",
        binary: "quire",
        format: BinaryFormat::Elf { machine: 62 },
    },
    DistributionTarget {
        rust: "aarch64-unknown-linux-musl",
        platform: "linux",
        arch: "arm64",
        binary: "quire",
        format: BinaryFormat::Elf { machine: 183 },
    },
    DistributionTarget {
        rust: "aarch64-apple-darwin",
        platform: "darwin",
        arch: "arm64",
        binary: "quire",
        format: BinaryFormat::MachO {
            cpu_type: 0x0100_000c,
        },
    },
    DistributionTarget {
        rust: "x86_64-pc-windows-msvc",
        platform: "win32",
        arch: "x64",
        binary: "quire.exe",
        format: BinaryFormat::Pe { machine: 0x8664 },
    },
];

#[derive(Debug)]
pub struct PackageNpm<'a> {
    pub version: &'a str,
    pub cargo_manifest: &'a Path,
    pub artifacts_dir: &'a Path,
    pub output_dir: &'a Path,
    pub launcher_dir: &'a Path,
    pub license: &'a Path,
}

#[derive(Debug)]
pub struct SetVersion<'a> {
    pub version: &'a str,
    pub cargo_manifest: &'a Path,
    pub launcher_manifest: &'a Path,
}

#[derive(Debug)]
pub struct VerifyRelease<'a> {
    pub version: &'a str,
    pub cargo_manifest: &'a Path,
    pub launcher_manifest: &'a Path,
}

fn checked_version(raw: &str) -> Result<Version> {
    Version::parse(raw).with_context(|| format!("invalid SemVer release version {raw:?}"))
}

fn read_cargo_manifest(path: &Path) -> Result<DocumentMut> {
    fs::read_to_string(path)
        .with_context(|| format!("read Cargo manifest {}", path.display()))?
        .parse::<DocumentMut>()
        .with_context(|| format!("parse Cargo manifest {}", path.display()))
}

fn cargo_package_version(document: &DocumentMut, path: &Path) -> Result<String> {
    document["package"]["version"]
        .as_str()
        .map(str::to_owned)
        .with_context(|| format!("{} has no string package.version", path.display()))
}

fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(
        &fs::read(path).with_context(|| format!("read JSON {}", path.display()))?,
    )
    .with_context(|| format!("parse JSON {}", path.display()))
}

fn object_mut<'a>(value: &'a mut Value, path: &Path) -> Result<&'a mut Map<String, Value>> {
    value
        .as_object_mut()
        .with_context(|| format!("{} must contain a JSON object", path.display()))
}

fn package_name(target: &DistributionTarget) -> String {
    format!("@agent-ix/quire-cli-{}-{}", target.platform, target.arch)
}

fn expected_package_names() -> BTreeSet<String> {
    TARGETS.iter().map(package_name).collect()
}

fn launcher_dependencies<'a>(
    object: &'a mut Map<String, Value>,
    path: &Path,
) -> Result<&'a mut Map<String, Value>> {
    let dependencies = object
        .get_mut("optionalDependencies")
        .with_context(|| format!("{} has no optionalDependencies", path.display()))?;
    let dependencies = dependencies
        .as_object_mut()
        .with_context(|| format!("{}.optionalDependencies must be an object", path.display()))?;
    let observed: BTreeSet<_> = dependencies.keys().cloned().collect();
    let expected = expected_package_names();
    if observed != expected {
        bail!(
            "{}.optionalDependencies keys differ from the target catalog: expected {expected:?}, observed {observed:?}",
            path.display()
        );
    }
    Ok(dependencies)
}

fn require_json_value(
    object: &Map<String, Value>,
    key: &str,
    expected: &Value,
    path: &Path,
) -> Result<()> {
    let observed = object
        .get(key)
        .with_context(|| format!("{} has no {key}", path.display()))?;
    if observed != expected {
        bail!(
            "{}.{key} must be {expected}, observed {observed}",
            path.display()
        );
    }
    Ok(())
}

fn validate_launcher_contract(object: &Map<String, Value>, path: &Path) -> Result<()> {
    require_json_value(object, "name", &json!("@agent-ix/quire-cli"), path)?;
    require_json_value(object, "license", &json!("AGPL-3.0-or-later"), path)?;
    require_json_value(object, "type", &json!("commonjs"), path)?;
    require_json_value(object, "bin", &json!({ "quire": "bin/quire.js" }), path)?;
    require_json_value(object, "files", &json!(["bin/", "LICENSE"]), path)?;
    require_json_value(
        object,
        "publishConfig",
        &json!({
            "registry": "https://registry.npmjs.org/",
            "access": "public"
        }),
        path,
    )?;
    if object
        .get("dependencies")
        .and_then(Value::as_object)
        .is_some_and(|dependencies| !dependencies.is_empty())
    {
        bail!("{}.dependencies must be absent or empty", path.display());
    }
    Ok(())
}

fn json_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).context("serialize package JSON")?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .with_context(|| format!("create temporary file beside {}", path.display()))?;
    temporary
        .write_all(bytes)
        .with_context(|| format!("write temporary file for {}", path.display()))?;
    temporary
        .flush()
        .with_context(|| format!("flush temporary file for {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o644))
            .with_context(|| format!("set metadata mode for {}", path.display()))?;
    }
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}

/// Set the root crate and generated launcher dependency versions in lockstep.
pub fn set_version(request: &SetVersion<'_>) -> Result<()> {
    let version = checked_version(request.version)?.to_string();
    let mut cargo = read_cargo_manifest(request.cargo_manifest)?;
    let current_cargo_version = cargo_package_version(&cargo, request.cargo_manifest)?;
    checked_version(&current_cargo_version).with_context(|| {
        format!(
            "{}.package.version is not valid SemVer",
            request.cargo_manifest.display()
        )
    })?;
    cargo["package"]["version"] = value(&version);

    let mut launcher = read_json(request.launcher_manifest)?;
    let launcher_object = object_mut(&mut launcher, request.launcher_manifest)?;
    if !launcher_object.get("version").is_some_and(Value::is_string) {
        bail!(
            "{}.version must be a string",
            request.launcher_manifest.display()
        );
    }
    let dependencies = launcher_dependencies(launcher_object, request.launcher_manifest)?;
    for dependency_version in dependencies.values_mut() {
        if !dependency_version.is_string() {
            bail!(
                "{}.optionalDependencies values must be strings",
                request.launcher_manifest.display()
            );
        }
        *dependency_version = Value::String(version.clone());
    }
    launcher_object.insert("version".to_owned(), Value::String(version));

    let cargo_bytes = cargo.to_string().into_bytes();
    let launcher_bytes = json_bytes(&launcher)?;
    write_atomic(request.cargo_manifest, &cargo_bytes)?;
    write_atomic(request.launcher_manifest, &launcher_bytes)?;
    Ok(())
}

fn verify_manifest_versions(
    version: &str,
    cargo_manifest: &Path,
    launcher_manifest: &Path,
) -> Result<()> {
    let expected = checked_version(version)?.to_string();
    let cargo = read_cargo_manifest(cargo_manifest)?;
    let observed = cargo_package_version(&cargo, cargo_manifest)?;
    if observed != expected {
        bail!("Cargo package version {observed} does not match release version {expected}");
    }

    let mut launcher = read_json(launcher_manifest)?;
    let object = object_mut(&mut launcher, launcher_manifest)?;
    validate_launcher_contract(object, launcher_manifest)?;
    let launcher_version = object
        .get("version")
        .and_then(Value::as_str)
        .with_context(|| format!("{}.version must be a string", launcher_manifest.display()))?;
    if launcher_version != expected {
        bail!("launcher version {launcher_version} does not match release version {expected}");
    }
    let dependencies = launcher_dependencies(object, launcher_manifest)?;
    for (name, value) in dependencies {
        let observed = value.as_str().with_context(|| {
            format!(
                "{}.optionalDependencies[{name:?}] must be a string",
                launcher_manifest.display()
            )
        })?;
        if observed != expected {
            bail!(
                "launcher dependency {name} version {observed} does not match release version {expected}"
            );
        }
    }
    Ok(())
}

/// Verify all committed release-version declarations before packaging.
pub fn verify_release(request: &VerifyRelease<'_>) -> Result<()> {
    verify_manifest_versions(
        request.version,
        request.cargo_manifest,
        request.launcher_manifest,
    )
}

/// Execute one native artifact's version surface and compare it exactly.
pub fn verify_binary_version(version: &str, binary: &Path) -> Result<()> {
    let expected = checked_version(version)?.to_string();
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .with_context(|| format!("execute {} --version", binary.display()))?;
    if !output.status.success() {
        bail!(
            "{} --version failed with {}: {}",
            binary.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = std::str::from_utf8(&output.stdout)
        .with_context(|| format!("{} --version stdout is not UTF-8", binary.display()))?;
    let observed = stdout
        .split_whitespace()
        .nth(1)
        .with_context(|| format!("{} --version has no version field", binary.display()))?;
    if observed != expected {
        bail!(
            "binary {} reports version {observed}, expected {expected}",
            binary.display()
        );
    }
    Ok(())
}

fn u16_le(bytes: &[u8], start: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(start..start + 2)?.try_into().ok()?,
    ))
}

fn u32_le(bytes: &[u8], start: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(start..start + 4)?.try_into().ok()?,
    ))
}

fn verify_binary_format(path: &Path, target: &DistributionTarget) -> Result<()> {
    let mut file =
        fs::File::open(path).with_context(|| format!("open binary {}", path.display()))?;
    let mut bytes = Vec::with_capacity(4096);
    Read::by_ref(&mut file)
        .take(4096)
        .read_to_end(&mut bytes)
        .with_context(|| format!("read binary header {}", path.display()))?;
    let matches = match target.format {
        BinaryFormat::Elf { machine } => {
            bytes.get(0..6) == Some(b"\x7fELF\x02\x01") && u16_le(&bytes, 18) == Some(machine)
        }
        BinaryFormat::MachO { cpu_type } => {
            bytes.get(0..4) == Some(&[0xcf, 0xfa, 0xed, 0xfe])
                && u32_le(&bytes, 4) == Some(cpu_type)
        }
        BinaryFormat::Pe { machine } => {
            let pe_offset = u32_le(&bytes, 0x3c).map(|offset| offset as usize);
            bytes.get(0..2) == Some(b"MZ")
                && pe_offset.is_some_and(|offset| {
                    bytes.get(offset..offset + 4) == Some(b"PE\0\0")
                        && u16_le(&bytes, offset + 4) == Some(machine)
                })
        }
    };
    if !matches {
        bail!(
            "binary {} does not match target {} ({}/{})",
            path.display(),
            target.rust,
            target.platform,
            target.arch
        );
    }
    Ok(())
}

fn artifact_paths(root: &Path) -> Result<Vec<(DistributionTarget, PathBuf)>> {
    let expected_dirs: BTreeSet<_> = TARGETS
        .iter()
        .map(|target| target.rust.to_owned())
        .collect();
    let observed_dirs: BTreeSet<_> = fs::read_dir(root)
        .with_context(|| format!("read artifacts directory {}", root.display()))?
        .map(|entry| {
            let entry = entry.with_context(|| format!("read entry in {}", root.display()))?;
            let file_type = entry
                .file_type()
                .with_context(|| format!("read type for {}", entry.path().display()))?;
            if !file_type.is_dir() {
                bail!(
                    "unexpected non-directory artifact {}",
                    entry.path().display()
                );
            }
            entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("artifact directory name is not UTF-8"))
        })
        .collect::<Result<_>>()?;
    if observed_dirs != expected_dirs {
        bail!(
            "artifact targets differ from catalog: expected {expected_dirs:?}, observed {observed_dirs:?}"
        );
    }

    TARGETS
        .iter()
        .copied()
        .map(|target| {
            let directory = root.join(target.rust);
            let entries: Vec<_> = fs::read_dir(&directory)
                .with_context(|| format!("read artifact target {}", directory.display()))?
                .collect::<std::io::Result<_>>()?;
            if entries.len() != 1 || entries[0].file_name() != target.binary {
                let names: Vec<_> = entries
                    .iter()
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect();
                bail!(
                    "artifact target {} must contain only {:?}, observed {names:?}",
                    target.rust,
                    target.binary
                );
            }
            let path = entries[0].path();
            if !entries[0]
                .file_type()
                .with_context(|| format!("read artifact type {}", path.display()))?
                .is_file()
            {
                bail!("artifact {} must be a regular file", path.display());
            }
            verify_binary_format(&path, &target)?;
            Ok((target, path))
        })
        .collect()
}

fn platform_manifest(target: &DistributionTarget, version: &str) -> Value {
    json!({
        "name": package_name(target),
        "version": version,
        "description": format!("Prebuilt quire binary for {}-{}.", target.platform, target.arch),
        "homepage": "https://github.com/agent-ix/quire-cli#readme",
        "repository": {
            "type": "git",
            "url": "git+https://github.com/agent-ix/quire-cli.git"
        },
        "license": "AGPL-3.0-or-later",
        "os": [target.platform],
        "cpu": [target.arch],
        "files": ["bin/", "LICENSE"],
        "publishConfig": {
            "registry": "https://registry.npmjs.org/",
            "access": "public"
        }
    })
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .with_context(|| format!("set executable mode on {}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

fn replace_output_tree(staged: &Path, output: &Path, staging: tempfile::TempDir) -> Result<()> {
    let backup = staging.path().join("previous-output");
    if output.exists() {
        fs::rename(output, &backup)
            .with_context(|| format!("stage existing output {}", output.display()))?;
    }
    if let Err(error) = fs::rename(staged, output) {
        if backup.exists() {
            if let Err(restore_error) = fs::rename(&backup, output) {
                let recovery = staging.keep();
                bail!(
                    "replace output {} failed ({error}); restoring the previous output also failed ({restore_error}); recovery tree preserved at {}",
                    output.display(),
                    recovery.display()
                );
            }
        }
        return Err(error).with_context(|| format!("replace output {}", output.display()));
    }
    Ok(())
}

/// Generate the closed npm package set from validated native artifacts.
pub fn package_npm(request: &PackageNpm<'_>) -> Result<()> {
    let version = checked_version(request.version)?.to_string();
    let launcher_manifest = request.launcher_dir.join("package.json");
    verify_manifest_versions(&version, request.cargo_manifest, &launcher_manifest)?;
    for relative in ["bin/quire.js", "README.md"] {
        let path = request.launcher_dir.join(relative);
        if !fs::symlink_metadata(&path)
            .with_context(|| format!("read launcher member {}", path.display()))?
            .file_type()
            .is_file()
        {
            bail!("launcher member {} must be a regular file", path.display());
        }
    }
    let artifacts = artifact_paths(request.artifacts_dir)?;
    let license = fs::read(request.license)
        .with_context(|| format!("read license {}", request.license.display()))?;

    let output_parent = request
        .output_dir
        .parent()
        .with_context(|| format!("{} has no parent", request.output_dir.display()))?;
    fs::create_dir_all(output_parent)
        .with_context(|| format!("create output parent {}", output_parent.display()))?;
    let staging = Builder::new()
        .prefix(".quire-dist-")
        .tempdir_in(output_parent)
        .with_context(|| format!("create staging directory in {}", output_parent.display()))?;
    let staged_output = staging.path().join("dist");
    fs::create_dir(&staged_output).context("create staged npm output")?;

    for (target, source) in artifacts {
        let directory =
            staged_output.join(format!("quire-cli-{}-{}", target.platform, target.arch));
        let bin_directory = directory.join("bin");
        fs::create_dir_all(&bin_directory)
            .with_context(|| format!("create {}", bin_directory.display()))?;
        let destination = bin_directory.join(target.binary);
        fs::copy(&source, &destination)
            .with_context(|| format!("copy {} to {}", source.display(), destination.display()))?;
        if target.platform != "win32" {
            make_executable(&destination)?;
        }
        fs::write(directory.join("LICENSE"), &license)
            .with_context(|| format!("write license in {}", directory.display()))?;
        fs::write(
            directory.join("package.json"),
            json_bytes(&platform_manifest(&target, &version))?,
        )
        .with_context(|| format!("write package manifest in {}", directory.display()))?;
    }

    replace_output_tree(&staged_output, request.output_dir, staging)?;
    write_atomic(&request.launcher_dir.join("LICENSE"), &license)?;
    Ok(())
}
