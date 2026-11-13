//! Running the converters through `engineering-assurance producer-execute`
//! (FR-032). `quire` launches only `engineering-assurance`; the request it
//! sends is the field table of FR-032 section A.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Context;
use ix_cli_kit::exit::Outcome;
use serde::Deserialize;
use serde_json::{json, Value};

use super::tool::{self, Tool};
use super::{Args, Format};
use crate::commands::failure;

const RESULT_PROTOCOL: &str = "engineering-assurance.producer-execution-result/v1";

/// The resolved executables an export needs. All are resolved before any
/// conversion starts, so a missing Typst fails before pandoc runs.
pub(super) struct Converters {
    executor: Tool,
    pandoc: Tool,
    typst: Option<Tool>,
}

impl Converters {
    pub(super) fn resolve(args: &Args) -> anyhow::Result<Self> {
        let executor = tool::resolve(
            "engineering-assurance",
            "--engineering-assurance",
            args.engineering_assurance.as_deref(),
        )?;
        let pandoc = tool::resolve("pandoc", "--pandoc", args.pandoc.as_deref())?;
        let typst = match args.to {
            Format::Pdf => Some(tool::resolve("typst", "--typst", args.typst.as_deref())?),
            Format::Docx => None,
        };
        Ok(Self {
            executor,
            pandoc,
            typst,
        })
    }
}

/// One converter request of an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    PandocDocx,
    PandocTypst,
    TypstPdf,
}

impl Step {
    fn producer(self) -> &'static str {
        match self {
            Self::PandocDocx | Self::PandocTypst => "pandoc",
            Self::TypstPdf => "typst",
        }
    }

    /// The path, relative to the capability root, of the required output.
    fn output(self) -> &'static str {
        match self {
            Self::PandocDocx => "out/document.docx",
            Self::PandocTypst => "typ/document.typ",
            Self::TypstPdf => "out/document.pdf",
        }
    }

    fn arguments(self) -> Vec<&'static str> {
        let pandoc = |to: &'static str, output: &'static str| {
            vec![
                "--sandbox",
                "--standalone",
                "--from",
                "markdown-yaml_metadata_block",
                "--to",
                to,
                "--output",
                output,
                "--metadata-file",
                "in/metadata.yaml",
                "in/document.md",
            ]
        };
        match self {
            Self::PandocDocx => pandoc("docx", "out/document.docx"),
            Self::PandocTypst => pandoc("typst", "typ/document.typ"),
            Self::TypstPdf => vec!["compile", "typ/document.typ", "out/document.pdf"],
        }
    }
}

/// Runs the export's requests over one capability root and returns the path of
/// the converted file inside it. The root is removed when the returned guard
/// drops, on every exit path.
pub(super) fn convert(
    converters: &Converters,
    to: Format,
    body: &str,
    metadata: &str,
) -> anyhow::Result<RenderedFile> {
    let temp = std::env::temp_dir()
        .canonicalize()
        .context("resolving the temporary directory")?;
    let root = tempfile::Builder::new()
        .prefix("quire-export-")
        .tempdir_in(temp)
        .context("creating the capability root")?;
    let root_path = root
        .path()
        .canonicalize()
        .context("resolving the capability root")?;
    for directory in ["in", "typ", "out/home", "out/tmp"] {
        fs::create_dir_all(root_path.join(directory))
            .with_context(|| format!("creating {directory}"))?;
    }
    fs::write(root_path.join("in/document.md"), body).context("writing in/document.md")?;
    fs::write(root_path.join("in/metadata.yaml"), metadata).context("writing in/metadata.yaml")?;

    let steps: &[Step] = match to {
        Format::Docx => &[Step::PandocDocx],
        Format::Pdf => &[Step::PandocTypst, Step::TypstPdf],
    };
    for &step in steps {
        run_step(converters, &root_path, step)?;
    }
    let last = steps.last().map_or("", |step| step.output());
    let path = root_path.join(last);
    Ok(RenderedFile { path, _root: root })
}

/// A converted file inside a capability root that is removed on drop.
pub(super) struct RenderedFile {
    path: PathBuf,
    _root: tempfile::TempDir,
}

impl AsRef<Path> for RenderedFile {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

fn run_step(converters: &Converters, root: &Path, step: Step) -> anyhow::Result<()> {
    let tool = match step {
        Step::TypstPdf => converters
            .typst
            .as_ref()
            .context("typst was not resolved")?,
        Step::PandocDocx | Step::PandocTypst => &converters.pandoc,
    };
    let request = request(root, tool, step);
    let result = execute(&converters.executor, &request)?;
    interpret(tool, &result)?;
    if !root.join(step.output()).is_file() {
        return Err(converter_failure(format!(
            "{} reported completion but wrote no {}",
            tool.name,
            step.output()
        )));
    }
    Ok(())
}

/// The complete FR-032 request for one step. Fields not in the table are not set.
fn request(root: &Path, tool: &Tool, step: Step) -> Value {
    let root = root.display().to_string();
    json!({
        "protocol": "engineering-assurance.producer-execution-request/v1",
        "producer": {
            "name": step.producer(),
            "version": "unreported",
            "executable": tool.path.display().to_string(),
        },
        "caller": {"kind": "quire.export", "version": "1"},
        "procedure": "direct",
        "capabilityRoot": root,
        "arguments": step
            .arguments()
            .into_iter()
            .map(|value| json!({"kind": "literal", "value": value}))
            .collect::<Vec<_>>(),
        "environment": {
            "HOME": format!("{root}/out/home"),
            "TMPDIR": format!("{root}/out/tmp"),
        },
        "inputs": [],
        "stdin": {"kind": "null"},
        "outputs": [{"role": "document", "path": step.output(), "required": true}],
        "containment": {
            "profile": "process-group-v1",
            "contract": {"kind": "quire.export-containment", "version": "1"},
        },
        "cancellation": {"kind": "disabled"},
        "budget": {
            "timeoutMillis": 120_000,
            "maxStdoutBytes": 1_048_576,
            "maxStderrBytes": 1_048_576,
            "maxInputBytes": 268_435_456,
            "maxOutputArtifacts": 1,
            "maxOutputBytes": 268_435_456,
            "maxDescendants": 64,
            "maxConcurrency": 1,
        },
        "response": {
            "protocol": {"kind": "engineering-assurance.process-evidence", "version": "1"},
            "adapter": {
                "kind": "engineering-assurance.cli-process-evidence-adapter",
                "version": "1",
            },
            "exitCodes": {"kind": "exact", "codes": [0]},
        },
    })
}

/// The members of the FR-029 result line that the export reads. The result
/// carries more members (timing, host); they are not consulted.
#[derive(Debug, Deserialize)]
struct ExecutionResult {
    protocol: String,
    process: Option<Process>,
    state: State,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Process {
    terminal_status: Option<TerminalStatus>,
    stderr: Stream,
}

#[derive(Debug, Deserialize)]
struct TerminalStatus {
    kind: String,
    value: i64,
}

#[derive(Debug, Deserialize)]
struct Stream {
    bytes: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum State {
    Completed,
    Unavailable,
    TimedOut,
    MalformedResponse,
    ContainmentFailure,
    Cancelled,
    Refused { reason: String },
    Failed { reason: FailureReason },
}

#[derive(Debug, Deserialize)]
struct FailureReason {
    kind: String,
}

impl State {
    fn name(&self) -> String {
        match self {
            Self::Completed => "completed".to_string(),
            Self::Unavailable => "unavailable".to_string(),
            Self::TimedOut => "timed_out".to_string(),
            Self::MalformedResponse => "malformed_response".to_string(),
            Self::ContainmentFailure => "containment_failure".to_string(),
            Self::Cancelled => "cancelled".to_string(),
            Self::Refused { reason } => format!("refused ({reason})"),
            Self::Failed { reason } => format!("failed ({})", reason.kind),
        }
    }
}

/// The machine error `producer-execute` writes to stdout when it exits 2.
#[derive(Debug, Deserialize)]
struct MachineError {
    code: String,
    message: String,
}

/// Launches `engineering-assurance producer-execute --request -` with the
/// request on stdin and reads its single result line.
fn execute(executor: &Tool, request: &Value) -> anyhow::Result<ExecutionResult> {
    let mut child = Command::new(&executor.path)
        .args(["producer-execute", "--request", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            failure::invalid(anyhow::anyhow!(
                "missing converter: cannot start '{}': {error}; pass --engineering-assurance <PATH>",
                executor.path.display()
            ))
        })?;
    let mut stdin = child.stdin.take().context("executor stdin was not piped")?;
    let payload = serde_json::to_vec(request).context("encoding the producer request")?;
    // A write failure means the executor exited early; its own output explains why.
    let _ = stdin.write_all(&payload);
    drop(stdin);
    let output = child
        .wait_with_output()
        .context("waiting for engineering-assurance")?;

    match output.status.code() {
        Some(0) => {
            let result: ExecutionResult = serde_json::from_slice(&output.stdout)
                .map_err(|error| executor_error(format!("unreadable result line: {error}")))?;
            if result.protocol != RESULT_PROTOCOL {
                return Err(executor_error(format!(
                    "unexpected result protocol '{}'",
                    result.protocol
                )));
            }
            Ok(result)
        }
        Some(2) => {
            let detail = serde_json::from_slice::<MachineError>(&output.stdout).map_or_else(
                |_| String::from_utf8_lossy(&output.stderr).trim().to_string(),
                |error| format!("{}: {}", error.code, error.message),
            );
            Err(executor_error(detail))
        }
        other => Err(executor_error(format!(
            "exited with {other:?}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))),
    }
}

fn executor_error(detail: String) -> anyhow::Error {
    converter_failure(format!(
        "engineering-assurance producer-execute failed ({detail}); the installed \
         engineering-assurance may not match this quire"
    ))
}

fn converter_failure(message: String) -> anyhow::Error {
    failure::classified(Outcome::Internal, anyhow::anyhow!(message))
}

/// Maps a result state to the FR-033 failure kinds.
fn interpret(tool: &Tool, result: &ExecutionResult) -> anyhow::Result<()> {
    match &result.state {
        State::Completed => Ok(()),
        State::Unavailable => Err(failure::invalid(anyhow::anyhow!(
            "missing converter: '{}' is unavailable at '{}'; pass --{} <PATH> to name it",
            tool.name,
            tool.path.display(),
            tool.name
        ))),
        State::ContainmentFailure if !cfg!(target_os = "linux") => Err(converter_failure(
            "export is unsupported on this host because producer-execute containment requires Linux"
                .to_string(),
        )),
        state => {
            let mut message = format!("{} did not complete: {}", tool.name, state.name());
            if let Some(process) = &result.process {
                if let Some(status) = &process.terminal_status {
                    message.push_str(&format!("; {} {}", status.kind, status.value));
                }
                let stderr = String::from_utf8_lossy(&process.stderr.bytes);
                if !stderr.trim().is_empty() {
                    message.push_str(&format!("; stderr: {}", stderr.trim_end()));
                }
            }
            Err(converter_failure(message))
        }
    }
}
