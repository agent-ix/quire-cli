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

    fn arguments(self, with_identity: bool, documents: usize) -> Vec<String> {
        let pandoc = |to: &'static str, output: &'static str, template: [&'static str; 2]| {
            let mut arguments: Vec<String> = [
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
            ]
            .map(str::to_string)
            .to_vec();
            if with_identity {
                arguments.extend([template[0], template[1], "in/front.md"].map(str::to_string));
            }
            if documents == 0 {
                arguments.push("in/document.md".to_string());
            }
            for number in 1..=documents {
                arguments.push(format!("in/separator-{number}.md"));
                arguments.push(format!("in/document-{number}.md"));
            }
            arguments
        };
        match self {
            Self::PandocDocx => pandoc(
                "docx",
                "out/document.docx",
                ["--reference-doc", "in/template.docx"],
            ),
            Self::PandocTypst => pandoc(
                "typst",
                "typ/document.typ",
                ["--template", "in/template.typ"],
            ),
            Self::TypstPdf => ["compile", "typ/document.typ", "out/document.pdf"]
                .map(str::to_string)
                .to_vec(),
        }
    }
}

/// The parts that come with a document identity: the front matter and the
/// template in use.
pub(super) struct IdentityParts<'a> {
    pub(super) front: &'a str,
    /// The template in use, the default or the organisation's: Typst source
    /// (PDF) or a DOCX reference document.
    pub(super) template: Vec<u8>,
}

/// One document of a bundle and the generated separator before it.
pub(super) struct BundleDocument<'a> {
    pub(super) separator: String,
    pub(super) body: &'a str,
}

/// The generated converter inputs of one export, written under `in/`.
pub(super) struct Generated<'a> {
    /// The document of a single export, written to `in/document.md`.
    pub(super) body: &'a str,
    /// The documents of a bundle in order, each with its separator, written to
    /// `in/separator-<n>.md` and `in/document-<n>.md`. Empty for a single export,
    /// where `body` is used.
    pub(super) bundle: Vec<BundleDocument<'a>>,
    pub(super) metadata: &'a str,
    /// Front matter and the template, present for a Markdown input;
    /// a JSON report carries no identity and gets neither.
    pub(super) identity_parts: Option<IdentityParts<'a>>,
    /// The source commit time, the `SOURCE_DATE_EPOCH` of every request.
    pub(super) epoch: Option<i64>,
}

/// Runs the export's requests over one capability root and returns the path of
/// the converted file inside it. The root is removed when the returned guard
/// drops, on every exit path.
pub(super) fn convert(
    converters: &Converters,
    to: Format,
    generated: &Generated<'_>,
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
    let root_text = root_path
        .to_str()
        .ok_or_else(|| {
            failure::invalid(anyhow::anyhow!(
                "the capability root {} is not valid UTF-8; set TMPDIR to a UTF-8 path",
                root_path.display()
            ))
        })?
        .to_string();
    for directory in ["in", "typ", "out/home", "out/tmp"] {
        fs::create_dir_all(root_path.join(directory))
            .with_context(|| format!("creating {directory}"))?;
    }
    let template_name = match to {
        Format::Pdf => "in/template.typ",
        Format::Docx => "in/template.docx",
    };
    let mut files = vec![(
        "in/metadata.yaml".to_string(),
        generated.metadata.as_bytes(),
    )];
    if generated.bundle.is_empty() {
        files.push(("in/document.md".to_string(), generated.body.as_bytes()));
    }
    for (index, document) in generated.bundle.iter().enumerate() {
        let number = index + 1;
        files.push((
            format!("in/separator-{number}.md"),
            document.separator.as_bytes(),
        ));
        files.push((format!("in/document-{number}.md"), document.body.as_bytes()));
    }
    if let Some(parts) = &generated.identity_parts {
        files.push(("in/front.md".to_string(), parts.front.as_bytes()));
        files.push((template_name.to_string(), parts.template.as_slice()));
    }
    for (name, content) in files {
        fs::write(root_path.join(&name), content).with_context(|| format!("writing {name}"))?;
    }

    let steps: &[Step] = match to {
        Format::Docx => &[Step::PandocDocx],
        Format::Pdf => &[Step::PandocTypst, Step::TypstPdf],
    };
    for &step in steps {
        run_step(
            converters,
            &root_path,
            &root_text,
            Plan {
                step,
                with_identity: generated.identity_parts.is_some(),
                documents: generated.bundle.len(),
                epoch: generated.epoch,
            },
        )?;
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

/// One converter request: the step and the parts of the export it depends on.
#[derive(Clone, Copy)]
struct Plan {
    step: Step,
    with_identity: bool,
    /// The number of bundle documents; 0 for a single export.
    documents: usize,
    epoch: Option<i64>,
}

fn run_step(
    converters: &Converters,
    root: &Path,
    root_text: &str,
    plan: Plan,
) -> anyhow::Result<()> {
    let step = plan.step;
    let tool = match step {
        Step::TypstPdf => converters
            .typst
            .as_ref()
            .context("typst was not resolved")?,
        Step::PandocDocx | Step::PandocTypst => &converters.pandoc,
    };
    let request = request(root_text, tool, plan);
    let result = execute(&converters.executor, &request)?;
    interpret(tool, root, &result, cfg!(target_os = "linux"))?;
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
fn request(root: &str, tool: &Tool, plan: Plan) -> Value {
    let step = plan.step;
    let mut environment = json!({
        "HOME": format!("{root}/out/home"),
        "TMPDIR": format!("{root}/out/tmp"),
    });
    if let (Some(epoch), Some(map)) = (plan.epoch, environment.as_object_mut()) {
        map.insert("SOURCE_DATE_EPOCH".to_string(), json!(epoch.to_string()));
    }
    json!({
        "protocol": "engineering-assurance.producer-execution-request/v1",
        "producer": {
            "name": step.producer(),
            "version": "unreported",
            "executable": tool.executable,
        },
        "caller": {"kind": "quire.export", "version": "1"},
        "procedure": "direct",
        "capabilityRoot": root,
        "arguments": step
            .arguments(plan.with_identity, plan.documents)
            .into_iter()
            .map(|value| json!({"kind": "literal", "value": value}))
            .collect::<Vec<_>>(),
        "environment": environment,
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

/// The FR-033 message for `containment_failure`. The result carries no reason
/// member, so the message names the state only.
fn containment_message(linux: bool) -> &'static str {
    if linux {
        "containment_failure: producer-execute could not contain the converter"
    } else {
        "export is unsupported on this host because producer-execute containment requires Linux"
    }
}

/// Maps a result state to the FR-033 failure kinds.
fn interpret(
    tool: &Tool,
    root: &Path,
    result: &ExecutionResult,
    linux: bool,
) -> anyhow::Result<()> {
    match &result.state {
        State::Completed => Ok(()),
        State::Unavailable => Err(failure::invalid(anyhow::anyhow!(
            "missing converter: '{}' is unavailable at '{}'; pass --{} <PATH> to name it",
            tool.name,
            tool.path.display(),
            tool.name
        ))),
        State::ContainmentFailure => Err(converter_failure(containment_message(linux).to_string())),
        state => {
            let mut message = format!("{} did not complete: {}", tool.name, state.name());
            if matches!(state, State::Refused { .. }) {
                message.push_str(&format!(
                    "; executable '{}', capability root '{}'",
                    tool.path.display(),
                    root.display()
                ));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-033-AC-8")]
    fn containment_failure_message_depends_on_the_host() {
        assert!(containment_message(false).contains("unsupported on this host"));
        assert!(containment_message(true).contains("containment_failure"));
        assert!(!containment_message(true).contains("unsupported"));
    }

    #[test]
    #[trace("FR-033-AC-8")]
    fn a_non_linux_host_reports_the_unsupported_host_message() {
        let tool = Tool {
            name: "pandoc",
            path: PathBuf::from("/x/pandoc"),
            executable: "/x/pandoc".to_string(),
        };
        let result = ExecutionResult {
            protocol: RESULT_PROTOCOL.to_string(),
            process: None,
            state: State::ContainmentFailure,
        };
        let error = interpret(&tool, Path::new("/r"), &result, false).expect_err("failure");
        assert!(format!("{error:#}").contains("unsupported on this host"));
    }
}
