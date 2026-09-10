//! NFR-007 — exact Rust toolchain and reproducible build-tool policy.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;

const QUALIFIED_RUST: &str = "1.98.1";

#[derive(Debug, PartialEq, Eq)]
struct Finding {
    path: PathBuf,
    line: usize,
    reason: String,
}

impl Finding {
    fn new(path: impl Into<PathBuf>, line: usize, reason: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line,
            reason: reason.into(),
        }
    }
}

fn read(root: &Path, relative: &Path) -> Result<String, Finding> {
    fs::read_to_string(root.join(relative)).map_err(|error| {
        Finding::new(
            relative,
            0,
            format!("required policy input cannot be read: {error}"),
        )
    })
}

fn quoted_assignment(text: &str, key: &str) -> Option<(usize, String)> {
    text.lines().enumerate().find_map(|(index, line)| {
        let (observed_key, value) = line.trim().split_once('=')?;
        if observed_key.trim() != key {
            return None;
        }
        Some((index + 1, value.trim().trim_matches('"').to_owned()))
    })
}

fn require_assignment(
    root: &Path,
    relative: &Path,
    key: &str,
    expected: &str,
    findings: &mut Vec<Finding>,
) {
    let text = match read(root, relative) {
        Ok(text) => text,
        Err(finding) => {
            findings.push(finding);
            return;
        }
    };
    match quoted_assignment(&text, key) {
        Some((_, value)) if value == expected => {}
        Some((line, value)) => findings.push(Finding::new(
            relative,
            line,
            format!("{key} must be {expected}, found {value}"),
        )),
        None => findings.push(Finding::new(
            relative,
            0,
            format!("required {key} declaration is missing"),
        )),
    }
}

fn workflow_paths(root: &Path, findings: &mut Vec<Finding>) -> Vec<PathBuf> {
    let workflow_dir = root.join(".github/workflows");
    let Ok(entries) = fs::read_dir(&workflow_dir) else {
        findings.push(Finding::new(
            ".github/workflows",
            0,
            "workflow directory cannot be read",
        ));
        return Vec::new();
    };

    let mut paths = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                findings.push(Finding::new(
                    ".github/workflows",
                    0,
                    format!("workflow entry cannot be read: {error}"),
                ));
                continue;
            }
        };
        let path = entry.path();
        if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("yml" | "yaml")
        ) {
            match path.strip_prefix(root) {
                Ok(relative) => paths.push(relative.to_path_buf()),
                Err(error) => findings.push(Finding::new(
                    path,
                    0,
                    format!("workflow path is outside the repository root: {error}"),
                )),
            }
        }
    }
    paths.sort();
    if paths.is_empty() {
        findings.push(Finding::new(
            ".github/workflows",
            0,
            "no .yml or .yaml workflow policy inputs found",
        ));
    }
    paths
}

fn action_reference(line: &str) -> Option<&str> {
    line.split_once("uses:")?.1.split_whitespace().next()
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn cargo_command_needs_lock(line: &str) -> bool {
    [
        "cargo bench",
        "cargo build",
        "cargo check",
        "cargo clippy",
        "cargo doc",
        "cargo run",
        "cargo test",
    ]
    .iter()
    .any(|command| line.contains(command))
}

fn make_cargo_command_needs_lock(line: &str) -> bool {
    ["bench", "build", "check", "clippy", "doc", "run", "test"]
        .iter()
        .any(|command| line.contains(&format!("$(CARGO) {command}")))
}

fn audit_workflow(relative: &Path, text: &str, findings: &mut Vec<Finding>) {
    let lines: Vec<_> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let number = index + 1;
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with("- name:") {
            continue;
        }

        if let Some(action) = action_reference(line) {
            let revision = action.rsplit_once('@').map(|(_, revision)| revision);
            if !revision.is_some_and(is_full_sha) {
                findings.push(Finding::new(
                    relative,
                    number,
                    "action is not pinned to a full commit SHA",
                ));
            }
        }
        if line.contains("uses: dtolnay/rust-toolchain@") {
            let selected = lines
                .iter()
                .skip(index + 1)
                .take(5)
                .take_while(|candidate| !candidate.trim_start().starts_with("- uses:"))
                .find_map(|candidate| candidate.trim().strip_prefix("toolchain:"))
                .map(|value| value.trim().trim_matches('"'));
            if selected != Some(QUALIFIED_RUST) {
                findings.push(Finding::new(
                    relative,
                    number,
                    format!(
                        "Rust toolchain action must select exact {QUALIFIED_RUST}, found {selected:?}"
                    ),
                ));
            }
        }
        if (trimmed.starts_with("runs-on:") || trimmed.starts_with("os:"))
            && trimmed.contains("latest")
        {
            findings.push(Finding::new(relative, number, "mutable runner label"));
        }
        if let Some(value) = trimmed.strip_prefix("toolchain:") {
            let value = value.trim().trim_matches('"');
            if value != QUALIFIED_RUST {
                findings.push(Finding::new(
                    relative,
                    number,
                    format!("compiler selector must be exact {QUALIFIED_RUST}, found {value}"),
                ));
            }
        }
        if matches!(trimmed, "tool: cargo-deny" | "tool: hyperfine") {
            findings.push(Finding::new(
                relative,
                number,
                "installed utility is not exact",
            ));
        }
        if line.contains("npm install -g npm@latest") {
            findings.push(Finding::new(relative, number, "npm is mutable"));
        }
        if cargo_command_needs_lock(line) && !line.contains("--locked") {
            findings.push(Finding::new(
                relative,
                number,
                "Cargo resolution is not --locked",
            ));
        }
        if line.contains("cargo deny") && !line.contains("--locked") {
            findings.push(Finding::new(
                relative,
                number,
                "cargo-deny resolution is not --locked",
            ));
        }
    }
}

fn audit_makefile(root: &Path, findings: &mut Vec<Finding>) {
    let relative = Path::new("Makefile");
    let text = match read(root, relative) {
        Ok(text) => text,
        Err(finding) => {
            findings.push(finding);
            return;
        }
    };
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if make_cargo_command_needs_lock(line) && !line.contains("--locked") {
            findings.push(Finding::new(
                relative,
                number,
                "canonical Cargo command is not --locked",
            ));
        }
        if line.contains("$(CARGO) deny") && !line.contains("--locked") {
            findings.push(Finding::new(
                relative,
                number,
                "cargo-deny resolution is not --locked",
            ));
        }
    }
    if !text
        .lines()
        .any(|line| line.trim() == "$(CARGO) deny --locked check")
    {
        findings.push(Finding::new(
            relative,
            0,
            "canonical deny target must run advisories, bans, licenses, and sources",
        ));
    }
    let ci = text
        .lines()
        .find(|line| line.starts_with("ci:"))
        .unwrap_or_default();
    if !ci.split_whitespace().any(|target| target == "cargo-audit") {
        findings.push(Finding::new(
            relative,
            0,
            "ci aggregate does not run cargo-audit",
        ));
    }
}

fn audit(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    require_assignment(
        root,
        Path::new("Cargo.toml"),
        "rust-version",
        QUALIFIED_RUST,
        &mut findings,
    );
    require_assignment(
        root,
        Path::new("tools/quire-dist/Cargo.toml"),
        "rust-version",
        QUALIFIED_RUST,
        &mut findings,
    );
    require_assignment(
        root,
        Path::new("rust-toolchain.toml"),
        "channel",
        QUALIFIED_RUST,
        &mut findings,
    );
    require_assignment(
        root,
        Path::new("clippy.toml"),
        "msrv",
        QUALIFIED_RUST,
        &mut findings,
    );

    for path in workflow_paths(root, &mut findings) {
        match read(root, &path) {
            Ok(text) => audit_workflow(&path, &text, &mut findings),
            Err(finding) => findings.push(finding),
        }
    }
    audit_makefile(root, &mut findings);
    findings
}

fn copy_policy_tree(source: &Path, destination: &Path) {
    for relative in [
        "Cargo.toml",
        "tools/quire-dist/Cargo.toml",
        "rust-toolchain.toml",
        "clippy.toml",
        "Makefile",
    ] {
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).expect("create policy parent");
        }
        fs::copy(source.join(relative), target).expect("copy policy file");
    }
    fs::create_dir_all(destination.join(".github/workflows")).expect("create workflows");
    for relative in workflow_paths(source, &mut Vec::new()) {
        fs::copy(source.join(&relative), destination.join(&relative)).expect("copy workflow");
    }
}

fn governed_lines(root: &Path) -> Vec<(PathBuf, usize)> {
    let mut paths = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("tools/quire-dist/Cargo.toml"),
        PathBuf::from("rust-toolchain.toml"),
        PathBuf::from("clippy.toml"),
    ];
    paths.extend(workflow_paths(root, &mut Vec::new()));

    paths
        .into_iter()
        .flat_map(|path| {
            let text = fs::read_to_string(root.join(&path)).expect("read governed file");
            text.lines()
                .enumerate()
                .filter(|(_, line)| {
                    let line = line.trim();
                    line.starts_with("rust-version =")
                        || line.starts_with("channel =")
                        || line.starts_with("msrv =")
                        || line.starts_with("toolchain:")
                })
                .map(|(index, _)| (path.clone(), index + 1))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn replace_line(path: &Path, line_number: usize, replacement: &str) {
    let text = fs::read_to_string(path).expect("read mutation target");
    let mut lines: Vec<_> = text.lines().map(str::to_owned).collect();
    lines[line_number - 1] = replacement.to_owned();
    fs::write(path, format!("{}\n", lines.join("\n"))).expect("write mutant");
}

fn remove_line(path: &Path, line_number: usize) {
    let text = fs::read_to_string(path).expect("read mutation target");
    let mut lines: Vec<_> = text.lines().map(str::to_owned).collect();
    lines.remove(line_number - 1);
    fs::write(path, format!("{}\n", lines.join("\n"))).expect("write mutant");
}

#[derive(Debug)]
struct MakefileMutation {
    line: usize,
    replacement: String,
    expected_reason: &'static str,
}

fn makefile_mutations(root: &Path) -> Vec<MakefileMutation> {
    let text = fs::read_to_string(root.join("Makefile")).expect("read Makefile mutations");
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let (replacement, expected_reason) = if make_cargo_command_needs_lock(line) {
                (
                    line.replacen("--locked", "", 1),
                    "canonical Cargo command is not --locked",
                )
            } else if line.contains("$(CARGO) deny") {
                (
                    line.replacen("--locked", "", 1),
                    "cargo-deny resolution is not --locked",
                )
            } else if line.starts_with("ci:") && line.contains("cargo-audit") {
                (
                    line.replacen("cargo-audit", "", 1),
                    "ci aggregate does not run cargo-audit",
                )
            } else {
                return None;
            };
            Some(MakefileMutation {
                line: index + 1,
                replacement,
                expected_reason,
            })
        })
        .collect()
}

#[trace("TC-142", "NFR-007-AC-1", "NFR-007-AC-4")]
#[test]
fn tc142_every_compiler_declaration_is_exact_and_mutation_sensitive() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(audit(root), Vec::new(), "production tool policy drifted");

    let declarations = governed_lines(root);
    assert_eq!(
        declarations.len(),
        9,
        "the governed declaration census changed"
    );
    for (relative, line) in declarations {
        let fixture = tempfile::tempdir().expect("tempdir");
        copy_policy_tree(root, fixture.path());
        replace_line(
            &fixture.path().join(&relative),
            line,
            if relative == Path::new("Cargo.toml")
                || relative == Path::new("tools/quire-dist/Cargo.toml")
            {
                "rust-version = \"1.94.1\""
            } else if relative == Path::new("rust-toolchain.toml") {
                "channel = \"stable\""
            } else if relative == Path::new("clippy.toml") {
                "msrv = \"1.75\""
            } else {
                "          toolchain: stable"
            },
        );
        let findings = audit(fixture.path());
        assert!(
            findings.iter().any(|finding| finding.path == relative),
            "mutating {}:{line} escaped the audit: {findings:?}",
            relative.display()
        );

        if relative.starts_with(".github/workflows") {
            let fixture = tempfile::tempdir().expect("tempdir");
            copy_policy_tree(root, fixture.path());
            remove_line(&fixture.path().join(&relative), line);
            let findings = audit(fixture.path());
            assert!(
                findings.iter().any(|finding| finding.path == relative),
                "removing {}:{line} escaped the audit: {findings:?}",
                relative.display()
            );
        }
    }

    let workflow = Path::new(".github/workflows/ci.yml");
    let workflow_text = fs::read_to_string(root.join(workflow)).expect("read action mutation");
    let (line, original, action) = workflow_text
        .lines()
        .enumerate()
        .find_map(|(index, line)| action_reference(line).map(|action| (index + 1, line, action)))
        .expect("at least one governed action");
    let (_, revision) = action
        .rsplit_once('@')
        .expect("production action has a revision");
    let fixture = tempfile::tempdir().expect("tempdir");
    copy_policy_tree(root, fixture.path());
    replace_line(
        &fixture.path().join(workflow),
        line,
        &original.replacen(&format!("@{revision}"), "", 1),
    );
    let findings = audit(fixture.path());
    assert!(
        findings.iter().any(|finding| {
            finding.path == workflow
                && finding.line == line
                && finding.reason == "action is not pinned to a full commit SHA"
        }),
        "removing an action revision escaped the audit: {findings:?}"
    );

    let fixture = tempfile::tempdir().expect("tempdir");
    copy_policy_tree(root, fixture.path());
    fs::write(
        fixture.path().join(".github/workflows/escape.yaml"),
        "name: escape\non: workflow_dispatch\njobs:\n  check:\n    runs-on: ubuntu-24.04\n    steps:\n      - uses: dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c\n        with:\n          toolchain: stable\n",
    )
    .expect("write .yaml mutant");
    let findings = audit(fixture.path());
    assert!(
        findings
            .iter()
            .any(|finding| finding.path == Path::new(".github/workflows/escape.yaml")),
        "a .yaml workflow escaped the audit: {findings:?}"
    );
}

#[trace("TC-143", "NFR-007-AC-3")]
#[test]
fn tc143_makefile_locking_policy_is_mutation_sensitive() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(audit(root), Vec::new(), "production tool policy drifted");
    let mutations = makefile_mutations(root);
    assert_eq!(mutations.len(), 19, "the Makefile mutation census changed");

    for mutation in mutations {
        let fixture = tempfile::tempdir().expect("tempdir");
        copy_policy_tree(root, fixture.path());
        replace_line(
            &fixture.path().join("Makefile"),
            mutation.line,
            &mutation.replacement,
        );
        let findings = audit(fixture.path());
        assert!(
            findings.iter().any(|finding| {
                finding.path == Path::new("Makefile") && finding.reason == mutation.expected_reason
            }),
            "mutating Makefile:{} escaped the audit: {findings:?}",
            mutation.line
        );
    }
}
