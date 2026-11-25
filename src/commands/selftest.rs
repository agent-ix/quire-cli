//! List and run the self-test cases embedded in the installed `quire` binary.

use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context};
use clap::{ArgGroup, Parser};
use serde::Serialize;
use tempfile::TempDir;

use crate::commands::{failure, Ctx};

#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("action")
        .required(true)
        .multiple(false)
        .args(["list", "run"])
))]
pub struct Args {
    /// List every self-test case embedded in this binary.
    #[arg(long, group = "action")]
    list: bool,

    /// Run every available case, or only the cases selected with `--case`.
    #[arg(long, group = "action")]
    run: bool,

    /// Select a case by id. Repeatable; duplicate ids run only once.
    #[arg(long, value_name = "CASE_ID", requires = "run")]
    case: Vec<String>,

    /// Also write the run record to this path.
    #[arg(long, value_name = "FILE", requires = "run")]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy)]
struct FileResource {
    path: &'static str,
    bytes: &'static [u8],
}

struct CaseResource {
    id: &'static str,
    source: &'static str,
    files: Vec<FileResource>,
}

static LOCAL_CASES: &[(&str, &[FileResource])] = &[(
    "parse-empty-input",
    &[
        FileResource {
            path: "run.sh",
            bytes: include_bytes!("../../selftest/cases/parse-empty-input/run.sh"),
        },
        FileResource {
            path: "required-tools",
            bytes: include_bytes!("../../selftest/cases/parse-empty-input/required-tools"),
        },
        FileResource {
            path: "input/empty.md",
            bytes: include_bytes!("../../selftest/cases/parse-empty-input/input/empty.md"),
        },
        FileResource {
            path: "expected/exit_code",
            bytes: include_bytes!("../../selftest/cases/parse-empty-input/expected/exit_code"),
        },
        FileResource {
            path: "expected/stdout",
            bytes: include_bytes!("../../selftest/cases/parse-empty-input/expected/stdout"),
        },
    ],
)];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunRecord {
    schema_version: u32,
    tool: &'static str,
    tool_version: String,
    os: &'static str,
    architecture: &'static str,
    suite: &'static str,
    timestamp: String,
    results: Vec<CaseResult>,
    retained_location: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CaseStatus {
    Pass,
    Fail,
    Error,
    NotExecuted,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaseResult {
    id: String,
    status: CaseStatus,
    criteria: Vec<String>,
    detail: String,
    stderr: String,
}

enum Destination {
    Explicit(PathBuf),
    EvidenceStore {
        canonical: PathBuf,
        output: Option<PathBuf>,
    },
}

pub fn run(_ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let cases = all_cases()?;
    if args.list {
        return list_cases(&cases);
    }
    if !args.run {
        bail!("choose either --list or --run");
    }
    ensure_supported_platform(std::env::consts::OS)?;

    let selected = select_cases(&cases, &args.case).map_err(failure::invalid)?;
    let timestamp = timestamp_rfc3339()?;
    let destination = destination(args.out.as_deref(), &timestamp).map_err(failure::invalid)?;
    let location = destination.retained_path().display().to_string();

    let version_output = version_output()?;
    let results = selected.into_iter().map(run_case).collect::<Vec<_>>();

    let record = RunRecord {
        schema_version: 1,
        tool: "quire",
        tool_version: version_output,
        os: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        suite: "SELFTEST",
        timestamp,
        results,
        retained_location: location,
    };
    let mut bytes = serde_json::to_vec_pretty(&record).context("encoding self-test run record")?;
    bytes.push(b'\n');
    let should_write_stdout = destination.retain(&bytes).map_err(failure::input)?;
    if should_write_stdout {
        ix_cli_kit::streams::write_primary_stdout(&bytes).context("writing run record")?;
    }

    if record.results.iter().any(|case| {
        matches!(
            case.status,
            CaseStatus::Fail | CaseStatus::Error | CaseStatus::NotExecuted
        )
    }) {
        return Err(failure::classified(
            ix_cli_kit::exit::Outcome::Partial,
            anyhow::anyhow!("one or more self-test cases did not pass"),
        ));
    }
    Ok(())
}

fn ensure_supported_platform(os: &str) -> anyhow::Result<()> {
    if os != "linux" {
        return Err(failure::refused(anyhow::anyhow!(
            "unsupported-platform: self-test execution is unavailable on {} because deterministic sandbox denial-path auditing is unavailable",
            os
        )));
    }
    Ok(())
}

impl Destination {
    fn retained_path(&self) -> &Path {
        match self {
            Self::Explicit(path) => path,
            Self::EvidenceStore { canonical, .. } => canonical,
        }
    }

    fn retain(&self, bytes: &[u8]) -> anyhow::Result<bool> {
        match self {
            Self::Explicit(path) => {
                fs::write(path, bytes)
                    .with_context(|| format!("writing self-test result to {}", path.display()))?;
                Ok(false)
            }
            Self::EvidenceStore { canonical, output } => {
                let parent = canonical
                    .parent()
                    .context("self-test evidence path has no parent directory")?;
                fs::create_dir_all(parent).with_context(|| {
                    format!("creating self-test evidence directory {}", parent.display())
                })?;
                fs::write(canonical, bytes).with_context(|| {
                    format!("retaining self-test result at {}", canonical.display())
                })?;
                if let Some(output) = output {
                    fs::write(output, bytes).with_context(|| {
                        format!("writing self-test result to {}", output.display())
                    })?;
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
        }
    }
}

fn destination(out: Option<&Path>, timestamp: &str) -> anyhow::Result<Destination> {
    let current = std::env::current_dir().context("resolving current directory")?;
    destination_from(&current, out, timestamp)
}

fn destination_from(
    current: &Path,
    out: Option<&Path>,
    timestamp: &str,
) -> anyhow::Result<Destination> {
    let current = fs::canonicalize(current).context("canonicalizing current directory")?;
    for ancestor in current.ancestors() {
        let evidence = ancestor.join("spec/evidence");
        if evidence.is_dir() {
            let canonical_evidence = fs::canonicalize(&evidence)
                .with_context(|| format!("canonicalizing evidence store {}", evidence.display()))?;
            let canonical = canonical_evidence
                .join("runs/SELFTEST")
                .join(format!("{timestamp}.json"));
            let canonical = canonicalize_for_write(&canonical)?;
            let output = out.map(canonicalize_for_write).transpose()?;
            return Ok(Destination::EvidenceStore { canonical, output });
        }
    }
    if let Some(path) = out {
        return Ok(Destination::Explicit(canonicalize_for_write(path)?));
    }
    bail!("no repository evidence store is available; pass --out <file>")
}

fn canonicalize_for_write(path: &Path) -> anyhow::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("resolving current directory")?
            .join(path)
    };
    if absolute.exists() {
        return fs::canonicalize(&absolute)
            .with_context(|| format!("canonicalizing output path {}", absolute.display()));
    }
    let parent = absolute
        .parent()
        .context("output path has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("creating output directory {}", parent.display()))?;
    let parent = fs::canonicalize(parent)
        .with_context(|| format!("canonicalizing output directory {}", parent.display()))?;
    let name = absolute
        .file_name()
        .context("output path has no file name")?;
    Ok(parent.join(name))
}

fn all_cases() -> anyhow::Result<Vec<CaseResource>> {
    let mut cases =
        Vec::with_capacity(LOCAL_CASES.len() + quire_rs::selftest::embedded_cases().len());
    for (id, files) in LOCAL_CASES {
        cases.push(CaseResource {
            id,
            source: "quire-cli",
            files: files.to_vec(),
        });
    }
    for case in quire_rs::selftest::embedded_cases() {
        cases.push(CaseResource {
            id: case.id,
            source: "quire-rs",
            files: case
                .files
                .iter()
                .map(|file| FileResource {
                    path: file.path,
                    bytes: file.bytes,
                })
                .collect(),
        });
    }
    let mut ids = BTreeSet::new();
    for case in &cases {
        if !ids.insert(case.id) {
            bail!("duplicate embedded self-test case id `{}`", case.id);
        }
        for file in &case.files {
            validate_relative_path(file.path)?;
        }
    }
    Ok(cases)
}

fn list_cases(cases: &[CaseResource]) -> anyhow::Result<()> {
    let mut output = Vec::new();
    for case in cases {
        output.extend_from_slice(case.id.as_bytes());
        output.push(b'\t');
        output.extend_from_slice(case.source.as_bytes());
        output.push(b'\n');
    }
    ix_cli_kit::streams::write_primary_stdout(&output).context("writing self-test case list")?;
    Ok(())
}

fn select_cases<'a>(
    cases: &'a [CaseResource],
    requested: &[String],
) -> anyhow::Result<Vec<&'a CaseResource>> {
    if requested.is_empty() {
        return Ok(cases.iter().collect());
    }
    for id in requested {
        if !cases.iter().any(|case| case.id == id.as_str()) {
            bail!("unknown self-test case id `{id}`");
        }
    }
    let requested = requested
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    Ok(cases
        .iter()
        .filter(|case| requested.contains(case.id))
        .collect())
}

fn timestamp_rfc3339() -> anyhow::Result<String> {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .context("formatting self-test timestamp")
}

fn case_tempdir() -> std::io::Result<TempDir> {
    tempfile::tempdir()
}

fn version_output() -> anyhow::Result<String> {
    let executable = std::env::current_exe().context("resolving current executable")?;
    let output = std::process::Command::new(executable)
        .arg("--version")
        .output()
        .context("reading quire version output")?;
    if !output.status.success() {
        bail!("quire --version exited with {}", output.status);
    }
    String::from_utf8(output.stdout).context("quire --version output is not UTF-8")
}

fn run_case(case: &CaseResource) -> CaseResult {
    let criteria = match criteria(case) {
        Ok(criteria) => criteria,
        Err(error) => {
            return CaseResult {
                id: case.id.to_string(),
                status: CaseStatus::Error,
                criteria: Vec::new(),
                detail: error.to_string(),
                stderr: String::new(),
            };
        }
    };
    let missing_tool = match missing_required_tool(case) {
        Ok(missing) => missing,
        Err(error) => {
            return CaseResult {
                id: case.id.to_string(),
                status: CaseStatus::Error,
                criteria,
                detail: error.to_string(),
                stderr: String::new(),
            };
        }
    };
    if let Some(tool) = missing_tool {
        return CaseResult {
            id: case.id.to_string(),
            status: CaseStatus::NotExecuted,
            criteria,
            detail: format!("required executable `{tool}` is unavailable"),
            stderr: String::new(),
        };
    }

    let temp = match case_tempdir().context("creating case temporary directory") {
        Ok(temp) => temp,
        Err(error) => return case_error(case, criteria, error.to_string()),
    };
    if let Err(error) = materialize(case, temp.path()) {
        return case_error(case, criteria, error.to_string());
    }
    let home = match case_tempdir().context("creating empty HOME directory") {
        Ok(home) => home,
        Err(error) => return case_error(case, criteria, error.to_string()),
    };
    let process = match execute_case_in_sandbox(case, &temp, &home) {
        Ok(process) => process,
        Err(error) => return case_error(case, criteria, error.to_string()),
    };
    compare_case(case, temp.path(), criteria, process)
}

struct ProcessOutput {
    exit_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
    denied_paths: Vec<String>,
}

struct BoundedOutput {
    exit_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
}

fn compare_case(
    case: &CaseResource,
    root: &Path,
    criteria: Vec<String>,
    process: ProcessOutput,
) -> CaseResult {
    let stderr = String::from_utf8_lossy(&process.stderr).into_owned();
    if process.timed_out {
        return case_error_with_stderr(
            case,
            criteria,
            "timeout after 60 seconds".to_owned(),
            stderr,
        );
    }
    if !process.denied_paths.is_empty() {
        return case_error_with_stderr(
            case,
            criteria,
            format!("sandbox denied writes: {}", process.denied_paths.join(", ")),
            stderr,
        );
    }
    let expected_exit = match expected_file(case, "expected/exit_code")
        .and_then(|bytes| std::str::from_utf8(bytes).context("expected exit code is not UTF-8"))
        .and_then(|text| {
            text.trim()
                .parse::<i32>()
                .context("expected exit code is not an integer")
        }) {
        Ok(code) => code,
        Err(error) => return case_error(case, criteria, error.to_string()),
    };
    let Some(actual_exit) = process.exit_code else {
        return case_error_with_stderr(
            case,
            criteria,
            "case process terminated without an exit code".to_owned(),
            stderr,
        );
    };
    if actual_exit != expected_exit {
        return CaseResult {
            id: case.id.to_owned(),
            status: CaseStatus::Fail,
            criteria,
            detail: format!(
                "exit code mismatch: expected {expected_exit}, got {}",
                actual_exit
            ),
            stderr,
        };
    }
    let expected_stdout = match expected_file(case, "expected/stdout") {
        Ok(bytes) => bytes,
        Err(error) => return case_error(case, criteria, error.to_string()),
    };
    if process.stdout != expected_stdout {
        return CaseResult {
            id: case.id.to_owned(),
            status: CaseStatus::Fail,
            criteria,
            detail: "stdout bytes differ from expected/stdout".to_owned(),
            stderr,
        };
    }
    for file in case
        .files
        .iter()
        .filter(|file| file.path.starts_with("expected/files/"))
    {
        let relative = match file.path.strip_prefix("expected/files/") {
            Some(path) => path,
            None => continue,
        };
        let actual = root.join(relative);
        let bytes = match fs::read(&actual) {
            Ok(bytes) => bytes,
            Err(error) => {
                return CaseResult {
                    id: case.id.to_owned(),
                    status: CaseStatus::Fail,
                    criteria,
                    detail: format!("expected file `{relative}` is missing or unreadable: {error}"),
                    stderr,
                };
            }
        };
        if bytes != file.bytes {
            return CaseResult {
                id: case.id.to_owned(),
                status: CaseStatus::Fail,
                criteria,
                detail: format!("file bytes differ for `{relative}`"),
                stderr,
            };
        }
    }
    CaseResult {
        id: case.id.to_owned(),
        status: CaseStatus::Pass,
        criteria,
        detail: String::new(),
        stderr,
    }
}

fn expected_file<'a>(case: &'a CaseResource, path: &str) -> anyhow::Result<&'a [u8]> {
    case.files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.bytes)
        .with_context(|| format!("case `{}` has no `{path}`", case.id))
}

fn case_error(case: &CaseResource, criteria: Vec<String>, detail: String) -> CaseResult {
    case_error_with_stderr(case, criteria, detail, String::new())
}

fn case_error_with_stderr(
    case: &CaseResource,
    criteria: Vec<String>,
    detail: String,
    stderr: String,
) -> CaseResult {
    CaseResult {
        id: case.id.to_owned(),
        status: CaseStatus::Error,
        criteria,
        detail,
        stderr,
    }
}

fn criteria(case: &CaseResource) -> anyhow::Result<Vec<String>> {
    let script = case
        .files
        .iter()
        .find(|file| file.path == "run.sh")
        .with_context(|| format!("case `{}` has no run.sh", case.id))?;
    let mut criteria = Vec::new();
    for line in script.bytes.split(|byte| *byte == b'\n') {
        let line = std::str::from_utf8(line).context("case run.sh is not UTF-8")?;
        if let Some(marker) = line.trim().strip_prefix("# trace:") {
            criteria.extend(
                marker
                    .split(',')
                    .map(str::trim)
                    .filter(|id| !id.is_empty())
                    .map(str::to_owned),
            );
        }
    }
    if criteria.is_empty() {
        bail!("case `{}` has no trace marker", case.id);
    }
    criteria.sort();
    criteria.dedup();
    Ok(criteria)
}

fn missing_required_tool(case: &CaseResource) -> anyhow::Result<Option<String>> {
    let file = case
        .files
        .iter()
        .find(|file| file.path == "required-tools")
        .with_context(|| format!("case `{}` has no required-tools file", case.id))?;
    let contents = std::str::from_utf8(file.bytes).context("required-tools is not UTF-8")?;
    let mut previous: Option<&str> = None;
    let path = std::env::var_os("PATH");
    for name in contents.lines() {
        if name.is_empty()
            || name.trim() != name
            || name.contains('/')
            || name.contains('\\')
            || name.contains('\0')
        {
            bail!(
                "case `{}` has invalid required executable `{name}`",
                case.id
            );
        }
        if previous.is_some_and(|last| last >= name) {
            bail!(
                "case `{}` required-tools entries are not lexical and unique",
                case.id
            );
        }
        previous = Some(name);
        if !path_has_executable(name, path.as_deref()) {
            return Ok(Some(name.to_string()));
        }
    }
    Ok(None)
}

fn path_has_executable(name: &str, path: Option<&std::ffi::OsStr>) -> bool {
    let Some(path) = path else {
        return false;
    };
    for directory in std::env::split_paths(path) {
        let candidate = directory.join(name);
        if candidate.exists() {
            return candidate.is_file() && is_executable(&candidate);
        }
    }
    false
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn materialize(case: &CaseResource, root: &Path) -> anyhow::Result<()> {
    for file in &case.files {
        validate_relative_path(file.path)?;
        let path = root.join(file.path);
        let parent = path
            .parent()
            .with_context(|| format!("case file `{}` has no parent", file.path))?;
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        fs::write(&path, file.bytes).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

fn validate_relative_path(value: &str) -> anyhow::Result<()> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("invalid embedded self-test resource path `{value}`");
    }
    Ok(())
}

fn execute_case_in_sandbox(
    case: &CaseResource,
    temp: &TempDir,
    home: &TempDir,
) -> anyhow::Result<ProcessOutput> {
    #[cfg(target_os = "linux")]
    {
        return execute_linux_case(case, temp, home);
    }
    #[cfg(target_os = "macos")]
    {
        return execute_macos_case(case, temp, home);
    }
    #[allow(unreachable_code)]
    {
        let _ = (case, temp, home);
        bail!("self-test execution is unsupported on this operating system")
    }
}

#[cfg(target_os = "linux")]
fn execute_linux_case(
    _case: &CaseResource,
    temp: &TempDir,
    home: &TempDir,
) -> anyhow::Result<ProcessOutput> {
    let executable = std::env::current_exe().context("resolving quire executable")?;
    let audit_dir =
        tempfile::tempdir_in("/tmp").context("creating runner-owned Linux audit directory")?;
    let trace_path = audit_dir.path().join("trace");
    let mut command = Command::new("/usr/bin/strace");
    command
        .args(["-qq", "-f", "-s", "4096", "-e", "trace=%file", "-o"])
        .arg(&trace_path)
        .args([
            "--",
            "bwrap",
            "--die-with-parent",
            "--unshare-net",
            "--unshare-pid",
            "--clearenv",
            "--ro-bind",
            "/",
            "/",
            "--bind",
        ])
        .arg(temp.path())
        .arg("/var/tmp")
        .args(["--ro-bind"])
        .arg(home.path())
        .arg("/home")
        .args([
            "--tmpfs",
            "/tmp",
            "--remount-ro",
            "/tmp",
            "--dev",
            "/dev",
            "--proc",
            "/proc",
            "--chdir",
        ])
        .arg("/var/tmp")
        .args([
            "--setenv",
            "CASE_TMP",
            "/var/tmp",
            "--setenv",
            "HOME",
            "/home",
            "--setenv",
            "TMPDIR",
            "/var/tmp",
            "--setenv",
            "QUIRE",
            executable.to_string_lossy().as_ref(),
            "--setenv",
            "PATH",
        ])
        .arg(std::env::var_os("PATH").unwrap_or_default())
        .args(["--", "/bin/sh"])
        .arg("/var/tmp/run.sh")
        .current_dir(temp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let bounded = run_bounded(command)?;
    if !trace_path.is_file() {
        bail!(
            "Linux self-test sandbox or write audit failed to start: {}",
            String::from_utf8_lossy(&bounded.stderr).trim()
        );
    }
    let trace = fs::read_to_string(&trace_path)
        .with_context(|| format!("reading Linux write audit {}", trace_path.display()))?;
    let denied_paths = denied_linux_writes(&trace, temp.path());
    fs::remove_file(&trace_path).context("removing Linux write audit")?;

    Ok(ProcessOutput {
        exit_code: bounded.exit_code,
        stdout: bounded.stdout,
        stderr: bounded.stderr,
        timed_out: bounded.timed_out,
        denied_paths,
    })
}

#[cfg(target_os = "macos")]
fn execute_macos_case(
    _case: &CaseResource,
    _temp: &TempDir,
    _home: &TempDir,
) -> anyhow::Result<ProcessOutput> {
    bail!("macOS self-test sandbox is not implemented; refusing to execute unsandboxed")
}

#[cfg(target_os = "linux")]
fn run_bounded(mut command: Command) -> anyhow::Result<BoundedOutput> {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
    let mut child = command
        .spawn()
        .context("starting self-test sandbox and audit process")?;
    let process_id = child.id();
    let mut stdout = child.stdout.take().context("capturing self-test stdout")?;
    let mut stderr = child.stderr.take().context("capturing self-test stderr")?;
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });

    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().context("waiting for self-test process")? {
            break status.code();
        }
        if started.elapsed() >= Duration::from_secs(60) {
            if let Some(status) = child.try_wait().context("checking timed-out process")? {
                break status.code();
            }
            timed_out = true;
            let group_killed = Command::new("/bin/kill")
                .args(["-KILL", "--"])
                .arg(format!("-{process_id}"))
                .status();
            if !group_killed.is_ok_and(|status| status.success()) {
                let _ = child.kill();
            }
            let _ = child.wait();
            break None;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| anyhow::anyhow!("self-test stdout reader panicked"))?
        .context("reading self-test stdout")?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| anyhow::anyhow!("self-test stderr reader panicked"))?
        .context("reading self-test stderr")?;
    Ok(BoundedOutput {
        exit_code: status,
        stdout,
        stderr,
        timed_out,
    })
}

#[cfg(target_os = "linux")]
fn denied_linux_writes(trace: &str, case_root: &Path) -> Vec<String> {
    let mut paths = BTreeSet::new();
    for line in trace.lines() {
        if !line.contains("= -1 EROFS")
            && !line.contains("= -1 EACCES")
            && !line.contains("= -1 EPERM")
        {
            continue;
        }
        let Some((prefix, _)) = line.split_once('(') else {
            continue;
        };
        let Some(call) = prefix.split_whitespace().last() else {
            continue;
        };
        let Some(path) = denied_write_path(call, line) else {
            continue;
        };
        paths.insert(normalize_attempted_path(path, case_root));
    }
    paths.into_iter().collect()
}

#[cfg(target_os = "linux")]
fn denied_write_path<'a>(call: &str, line: &'a str) -> Option<&'a str> {
    let mut quoted = line.split('"').skip(1).step_by(2);
    let first = quoted.next()?;
    let last = line.split('"').skip(1).step_by(2).last()?;
    let write_call = matches!(
        call,
        "creat"
            | "mkdir"
            | "mkdirat"
            | "rmdir"
            | "unlink"
            | "unlinkat"
            | "rename"
            | "renameat"
            | "renameat2"
            | "link"
            | "linkat"
            | "symlink"
            | "symlinkat"
            | "truncate"
            | "chmod"
            | "fchmodat"
            | "chown"
            | "lchown"
            | "fchownat"
            | "utime"
            | "utimes"
            | "utimensat"
    );
    if write_call {
        return Some(
            if matches!(
                call,
                "rename" | "renameat" | "renameat2" | "link" | "linkat" | "symlink" | "symlinkat"
            ) {
                last
            } else {
                first
            },
        );
    }
    if matches!(call, "open" | "openat" | "openat2")
        && [
            "O_WRONLY",
            "O_RDWR",
            "O_CREAT",
            "O_TRUNC",
            "O_APPEND",
            "O_TMPFILE",
        ]
        .iter()
        .any(|flag| line.contains(flag))
    {
        return Some(first);
    }
    None
}

#[cfg(target_os = "linux")]
fn normalize_attempted_path(path: &str, case_root: &Path) -> String {
    let path = Path::new(path);
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        case_root.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[trace("FR-047-AC-1", "FR-047-AC-10")]
    #[test]
    fn embedded_case_list_includes_both_owners_in_declaration_order() {
        let cases = all_cases().expect("embedded case inventory is valid");
        assert_eq!(cases.first().map(|case| case.source), Some("quire-cli"));
        assert!(
            cases.iter().any(|case| case.source == "quire-rs"),
            "the linked quire-rs case must be included"
        );
        assert_eq!(cases[0].id, "parse-empty-input");
    }

    #[trace("FR-047-AC-10")]
    #[test]
    fn selected_case_ids_are_unique_and_keep_declaration_order() {
        let cases = all_cases().expect("embedded case inventory is valid");
        let requested = vec![
            cases[1].id.to_owned(),
            cases[0].id.to_owned(),
            cases[1].id.to_owned(),
        ];
        let selected = select_cases(&cases, &requested).expect("selection is valid");
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, cases[0].id);
        assert_eq!(selected[1].id, cases[1].id);
    }

    #[trace("FR-047-AC-10")]
    #[test]
    fn unknown_case_is_rejected_before_selection() {
        let cases = all_cases().expect("embedded case inventory is valid");
        let error = match select_cases(&cases, &["unknown-case".to_owned()]) {
            Ok(_) => panic!("unknown ids are rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("unknown-case"));
    }

    #[trace("FR-047-AC-3")]
    #[test]
    fn local_trace_marker_is_retained_in_run_metadata() {
        let cases = all_cases().expect("embedded case inventory is valid");
        let trace = criteria(&cases[0]).expect("case has criterion trace");
        assert_eq!(trace, ["FR-002-AC-4"]);
    }

    #[trace("FR-047-AC-9")]
    #[test]
    fn unsupported_platforms_are_refused_before_case_selection_or_execution() {
        for os in ["macos", "windows", "freebsd"] {
            let error = ensure_supported_platform(os)
                .expect_err("unsupported operating systems must fail closed");
            assert!(error.to_string().contains("unsupported-platform"));
            assert_eq!(failure::outcome(&error), ix_cli_kit::exit::Outcome::Refused);
        }
    }

    #[trace("FR-047-AC-5")]
    #[test]
    fn missing_required_executable_records_not_executed_without_running_script() {
        static SCRIPT: &[u8] = b"#!/bin/sh\n# trace: FR-047-AC-5\nexit 0\n";
        static REQUIRED_TOOLS: &[u8] = b"quire-selftest-tool-that-does-not-exist-7f83b0\n";
        let case = CaseResource {
            id: "missing-required-tool",
            source: "test",
            files: vec![
                FileResource {
                    path: "run.sh",
                    bytes: SCRIPT,
                },
                FileResource {
                    path: "required-tools",
                    bytes: REQUIRED_TOOLS,
                },
            ],
        };

        let result = run_case(&case);

        assert!(matches!(result.status, CaseStatus::NotExecuted));
        assert!(result
            .detail
            .contains("quire-selftest-tool-that-does-not-exist-7f83b0"));
    }

    #[trace("FR-047-AC-1")]
    #[test]
    fn stderr_is_recorded_without_changing_a_passing_result() {
        let cases = all_cases().expect("embedded case inventory is valid");
        let case = &cases[0];
        let process = ProcessOutput {
            exit_code: Some(0),
            stdout: expected_file(case, "expected/stdout")
                .expect("case has expected stdout")
                .to_vec(),
            stderr: vec![0xff],
            timed_out: false,
            denied_paths: Vec::new(),
        };

        let result = compare_case(
            case,
            Path::new("."),
            criteria(case).expect("case has criterion trace"),
            process,
        );
        let other_stderr = compare_case(
            case,
            Path::new("."),
            criteria(case).expect("case has criterion trace"),
            ProcessOutput {
                exit_code: Some(0),
                stdout: expected_file(case, "expected/stdout")
                    .expect("case has expected stdout")
                    .to_vec(),
                stderr: b"another diagnostic\n".to_vec(),
                timed_out: false,
                denied_paths: Vec::new(),
            },
        );

        assert!(matches!(result.status, CaseStatus::Pass));
        assert_eq!(result.stderr, "\u{fffd}");
        assert!(matches!(other_stderr.status, CaseStatus::Pass));
        assert_eq!(other_stderr.stderr, "another diagnostic\n");
    }

    #[cfg(target_os = "linux")]
    #[trace("FR-047-AC-9")]
    #[test]
    fn sandbox_denials_are_audited_with_both_attempted_paths() {
        let outer = tempfile::tempdir().expect("outer temporary directory");
        let temp = tempfile::tempdir_in(outer.path()).expect("case temporary directory");
        let home = case_tempdir().expect("empty HOME directory");
        static CASE_SCRIPT: &[u8] = b"#!/bin/sh\n# trace: FR-047-AC-9\nset +e\nprintf attacker-controlled > \"$CASE_TMP/.quire-write-audit\"\nprintf parent > \"$CASE_TMP/../outside-parent\" 2>/dev/null\noutside=\"${CASE_TMP%/*}/outside-absolute\"\nprintf absolute > \"$outside\" 2>/dev/null\nexit 0\n";
        fs::write(temp.path().join("run.sh"), CASE_SCRIPT).expect("case script");
        let case = CaseResource {
            id: "sandbox-write-denials",
            source: "test",
            files: vec![
                FileResource {
                    path: "run.sh",
                    bytes: CASE_SCRIPT,
                },
                FileResource {
                    path: "expected/exit_code",
                    bytes: b"0",
                },
                FileResource {
                    path: "expected/stdout",
                    bytes: b"",
                },
            ],
        };
        let process = execute_case_in_sandbox(&case, &temp, &home)
            .expect("OS sandbox starts and audits the case");
        let result = compare_case(
            &case,
            temp.path(),
            criteria(&case).expect("case has trace marker"),
            process,
        );

        assert!(
            matches!(result.status, CaseStatus::Error),
            "case did not report denied writes: {result:?}"
        );
        assert!(
            result.detail.contains("/outside-parent"),
            "parent-relative attempted path was not reported: {:?}",
            result.detail
        );
        assert!(
            result.detail.contains("/var/outside-absolute"),
            "absolute attempted path was not reported: {:?}",
            result.detail
        );
        assert_eq!(
            fs::read(temp.path().join(".quire-write-audit")).expect("attacker audit file"),
            b"attacker-controlled"
        );
    }

    #[cfg(target_os = "linux")]
    #[trace("FR-047-AC-6")]
    #[test]
    fn linux_case_runs_with_empty_home_and_no_network_routes() {
        let host_routes = fs::read_to_string("/proc/net/route").expect("host route table");
        assert!(
            host_routes
                .lines()
                .skip(1)
                .any(|line| !line.trim().is_empty()),
            "test requires a host route to distinguish the isolated namespace"
        );

        let temp = case_tempdir().expect("case temporary directory");
        let home = case_tempdir().expect("empty HOME directory");
        static CASE_SCRIPT: &[u8] = b"#!/bin/sh\n# trace: FR-047-AC-6\nfor entry in \"$HOME\"/* \"$HOME\"/.[!.]* \"$HOME\"/..?*; do [ ! -e \"$entry\" ] || { printf 'HOME is not empty\\n'; exit 1; }; done\nexec 3</proc/net/route\nIFS= read -r header <&3\nif IFS= read -r route <&3; then printf 'network route is available\\n'; exit 1; fi\nprintf 'offline\\n'\n";
        let case = CaseResource {
            id: "empty-home-offline",
            source: "test",
            files: vec![
                FileResource {
                    path: "run.sh",
                    bytes: CASE_SCRIPT,
                },
                FileResource {
                    path: "expected/exit_code",
                    bytes: b"0",
                },
                FileResource {
                    path: "expected/stdout",
                    bytes: b"offline\n",
                },
            ],
        };
        materialize(&case, temp.path()).expect("materialize offline case");
        let process = execute_case_in_sandbox(&case, &temp, &home)
            .expect("Linux sandbox starts with empty HOME and isolated network");
        let result = compare_case(
            &case,
            temp.path(),
            criteria(&case).expect("case has trace marker"),
            process,
        );

        assert!(
            matches!(result.status, CaseStatus::Pass),
            "empty HOME/offline case did not pass: {result:?}"
        );
    }

    #[cfg(target_os = "linux")]
    #[trace("FR-047-AC-9")]
    #[test]
    fn denied_linux_write_parser_normalizes_parent_relative_paths() {
        let root = Path::new("/tmp/case-root");
        let trace = "openat(AT_FDCWD, \"../outside\", O_WRONLY|O_CREAT, 0666) = -1 EROFS (Read-only file system)\n";

        assert_eq!(
            denied_linux_writes(trace, root),
            ["/tmp/outside".to_owned()]
        );
    }

    #[trace("FR-047-AC-4")]
    #[test]
    fn evidence_store_discovery_creates_missing_run_directories() {
        let root = tempfile::tempdir().expect("repository root");
        let current = root.path().join("nested/working-directory");
        fs::create_dir_all(&current).expect("nested working directory");
        fs::create_dir_all(root.path().join("spec/evidence")).expect("evidence store");

        let destination = destination_from(&current, None, "1234").expect("evidence destination");
        assert!(matches!(destination, Destination::EvidenceStore { .. }));
        assert!(destination
            .retained_path()
            .ends_with("spec/evidence/runs/SELFTEST/1234.json"));

        assert!(destination
            .retain(b"record\n")
            .expect("evidence record is retained"));
        assert_eq!(
            fs::read(destination.retained_path()).expect("retained record is readable"),
            b"record\n"
        );
    }

    #[trace("FR-047-AC-4")]
    #[test]
    fn explicit_output_duplicates_the_canonical_store_record_without_stdout() {
        let root = tempfile::tempdir().expect("repository root");
        let current = root.path().join("working-directory");
        fs::create_dir_all(&current).expect("working directory");
        fs::create_dir_all(root.path().join("spec/evidence")).expect("evidence store");
        let output = root.path().join("external/result.json");

        let destination = destination_from(&current, Some(&output), "1234")
            .expect("evidence and explicit output destinations");
        let retained_path = destination.retained_path().to_path_buf();
        assert_ne!(retained_path, output);
        assert!(!destination
            .retain(b"canonical record\n")
            .expect("both records are retained"));
        assert_eq!(
            fs::read(&retained_path).expect("canonical record exists"),
            fs::read(&output).expect("explicit output exists")
        );
    }
}
