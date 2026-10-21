//! Process-boundary conformance for the distributed Git lock gate (FR-027).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use ix_trace_rs::trace;
use tempfile::TempDir;

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn package(name: &str, source: &str) -> String {
    format!("[[package]]\nname = {name:?}\nversion = \"1.0.0\"\nsource = {source:?}\n\n")
}

fn git(name: &str, revision: &str) -> String {
    package(
        name,
        &format!("git+https://github.com/agent-ix/fixture#{revision}"),
    )
}

fn lock(dir: &Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, text).unwrap();
    path.to_str().unwrap().to_owned()
}

fn run(paths: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quire"))
        .arg("--color=never")
        .arg("check-lock")
        .args(paths)
        .output()
        .unwrap()
}

fn assert_code(output: &Output, code: i32) {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    if code == 0 {
        assert!(output.stderr.is_empty(), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Git lock coherence checked:"));
    } else {
        assert!(output.stdout.is_empty(), "{output:?}");
        assert!(!output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
#[trace("FR-027-AC-1", "FR-027-AC-3")]
fn aligned_empty_path_registry_and_other_owner_locks_pass() {
    let dir = TempDir::new().unwrap();
    for text in [
        String::new(),
        "package = []".into(),
        "[[package]]\nname = \"workspace\"".into(),
        git("core", A) + &git("derive", A),
        package(
            "repeat",
            "registry+https://github.com/rust-lang/crates.io-index",
        ) + &package("repeat", "sparse+https://index.crates.io/"),
        package("repeat", "git+https://github.com/other/fixture#short")
            + &package("repeat", "git+ssh://git@example.test/other/fixture"),
    ] {
        let path = lock(dir.path(), "Cargo.lock", &text);
        assert_code(&run(&[&path]), 0);
    }
}

#[test]
#[trace("FR-027-AC-2", "FR-027-AC-6", "FR-027-AC-7")]
fn every_split_lock_and_affected_package_is_reported_in_order() {
    let dir = TempDir::new().unwrap();
    let z = lock(dir.path(), "z.lock", &(git("zeta", B) + &git("alpha", A)));
    let a = lock(dir.path(), "a.lock", &(git("beta", B) + &git("alpha", A)));
    let output = run(&[&z, &a]);
    assert_code(&output, 2);
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(
        stderr.find(&a).unwrap() < stderr.find(&z).unwrap(),
        "{stderr}"
    );
    for path in [&a, &z] {
        for revision in [A, B] {
            assert!(stderr
                .lines()
                .any(|line| line.contains(path) && line.contains(revision)));
        }
    }
    for name in ["alpha", "beta", "zeta"] {
        assert!(
            stderr.contains(&format!("package {name} revision")),
            "{stderr}"
        );
    }
    assert!(stderr.contains("github.com/agent-ix/fixture"));
    let repeated = run(&[&z, &a]);
    assert_eq!(output.stdout, repeated.stdout);
    assert_eq!(output.stderr, repeated.stderr);
}

#[test]
#[trace("FR-027-AC-3")]
fn duplicate_agent_names_are_refused_even_with_identical_or_different_repositories() {
    let dir = TempDir::new().unwrap();
    for second in [
        git("same", A),
        package(
            "same",
            &format!("git+https://github.com/agent-ix/other#{A}"),
        ),
    ] {
        let path = lock(dir.path(), "Cargo.lock", &(git("same", A) + &second));
        let output = run(&[&path]);
        assert_code(&output, 2);
        assert!(String::from_utf8_lossy(&output.stderr).contains("package same revision"));
    }
}

#[test]
#[trace("FR-027-AC-4")]
fn transport_case_query_and_terminal_variants_share_repository_identity() {
    let dir = TempDir::new().unwrap();
    let variants = [
        "https://github.com/agent-ix/fixture",
        "https://GITHUB.COM/AGENT-IX/FIXTURE.git?branch=main",
        "http://user:password@github.com:80/agent-ix/fixture/?tag=v1",
        "ssh://git@github.com:22/agent-ix/fixture.git/?rev=ignored",
        "git://github.com:9418/agent-ix/FIXTURE",
    ];
    let mut aligned = String::new();
    for (index, url) in variants.iter().enumerate() {
        aligned += &package(
            &format!("crate-{index}"),
            &format!("git+{url}#{}", A.to_uppercase()),
        );
    }
    let path = lock(dir.path(), "Cargo.lock", &aligned);
    assert_code(&run(&[&path]), 0);
    for url in variants {
        let text = git("base", A) + &package("other", &format!("git+{url}#{B}"));
        fs::write(&path, text).unwrap();
        assert_code(&run(&[&path]), 2);
    }
}

#[test]
#[trace("FR-027-AC-5")]
fn malformed_fragments_urls_toml_and_typed_package_fields_are_invalid() {
    let dir = TempDir::new().unwrap();
    for fragment in ["", "#", "#abc", "#gggggggggggggggggggggggggggggggggggggggg"] {
        let text = package(
            "bad",
            &format!("git+https://github.com/agent-ix/fixture{fragment}"),
        );
        let path = lock(dir.path(), "Cargo.lock", &text);
        assert_code(&run(&[&path]), 3);
    }
    for url in [
        "not-a-url",
        "https://",
        "file:///agent-ix/fixture",
        "https://github.com:123/agent-ix/fixture",
        "https://github.com/agent-ix/",
        "https://github.com/agent-ix/.git",
        "https://github.com/agent-ix/fixture/extra",
        "https://github.com/agent-ix/fixture%2fextra",
        "https://github.com/agent-ix/fixture%5cextra",
        "https://github.com/agent-ix%2ffixture",
    ] {
        let path = lock(
            dir.path(),
            "Cargo.lock",
            &package("bad", &format!("git+{url}#{A}")),
        );
        assert_code(&run(&[&path]), 3);
    }
    for text in [
        "not toml",
        "package = 3",
        "package = {}",
        "[[package]]\nversion = '1'",
        "[[package]]\nname = 2",
        "[[package]]\nname = 'a'\nsource = 3",
        "[[package]]\nname = 'a'\nsource = 'unknown+https://example.test'",
    ] {
        let path = lock(dir.path(), "Cargo.lock", text);
        assert_code(&run(&[&path]), 3);
    }
}

#[test]
#[trace("FR-027-AC-6", "FR-027-AC-7")]
fn independent_locks_are_not_compared_and_fatal_paths_never_report_success() {
    let dir = TempDir::new().unwrap();
    let a = lock(dir.path(), "a.lock", &git("one", A));
    let b = lock(dir.path(), "b.lock", &git("two", B));
    assert_code(&run(&[&a, &b]), 0);
    let missing = dir.path().join("missing.lock");
    for path in [missing.to_str().unwrap(), dir.path().to_str().unwrap()] {
        let output = run(&[&a, path]);
        assert_code(&output, 3);
        assert!(String::from_utf8_lossy(&output.stderr).contains(path));
    }
    assert_code(&run(&[]), 3);
    assert_code(&run(&["-"]), 3);
    assert_code(&run(&["../Cargo.lock"]), 2);
}

#[test]
#[trace("FR-027-AC-7")]
fn global_format_color_and_pretty_use_existing_stream_contract() {
    let dir = TempDir::new().unwrap();
    let path = lock(dir.path(), "Cargo.lock", &(git("one", A) + &git("two", B)));
    let output = Command::new(env!("CARGO_BIN_EXE_quire"))
        .args([
            "--diagnostics-format=json",
            "--color=never",
            "check-lock",
            &path,
        ])
        .output()
        .unwrap();
    assert_code(&output, 2);
    let record: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(record["kind"], "QuireError");
    assert!(record["message"].as_str().unwrap().contains(&path));
    let color = Command::new(env!("CARGO_BIN_EXE_quire"))
        .args(["--color=always", "check-lock", &path])
        .output()
        .unwrap();
    assert_code(&color, 2);
    assert!(color.stderr.contains(&0x1b));
    fs::write(&path, git("one", A)).unwrap();
    let plain = run(&[&path]);
    let pretty = Command::new(env!("CARGO_BIN_EXE_quire"))
        .args(["--pretty", "check-lock", &path])
        .output()
        .unwrap();
    assert_code(&pretty, 0);
    assert_eq!(plain.stdout, pretty.stdout);
}

#[cfg(unix)]
#[test]
#[trace("FR-027-AC-6", "FR-027-AC-7")]
fn input_permission_refusal_is_distinct_from_output_failure() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = lock(dir.path(), "Cargo.lock", &git("one", A));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let output = run(&[&path]);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert_code(&output, 2);
    #[cfg(target_os = "linux")]
    {
        let full = fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_quire"))
            .args(["check-lock", &path])
            .stdout(full)
            .output()
            .unwrap();
        assert_code(&output, 4);
    }
}

#[cfg(target_os = "linux")]
#[test]
#[trace("FR-027-AC-9")]
fn gate_leaves_inputs_unchanged_and_opens_no_network_or_child_process() {
    let dir = TempDir::new().unwrap();
    for (text, code) in [(git("one", A), 0), (git("one", A) + &git("two", B), 2)] {
        let path = lock(dir.path(), "Cargo.lock", &text);
        let before = fs::read(&path).unwrap();
        let trace = dir.path().join("trace.txt");
        let output = Command::new("strace")
            .args(["-f", "-e", "trace=network,execve", "-o"])
            .arg(&trace)
            .arg(env!("CARGO_BIN_EXE_quire"))
            .args(["--color=never", "check-lock", &path])
            .output()
            .expect("FR-027-AC-9 qualification requires strace");
        assert_code(&output, code);
        assert_eq!(fs::read(&path).unwrap(), before);
        let syscalls = fs::read_to_string(trace).unwrap();
        assert_eq!(
            syscalls
                .lines()
                .filter(|line| line.contains("execve("))
                .count(),
            1,
            "{syscalls}"
        );
        assert!(!syscalls.contains("socket(AF_INET"), "{syscalls}");
    }
}
