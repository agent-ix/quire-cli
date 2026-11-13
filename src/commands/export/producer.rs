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

use super::diagrams::{self, Block};
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
    /// Resolved only when the export has a Mermaid block (FR-042).
    mmdr: Option<Tool>,
}

impl Converters {
    /// `diagrams` is true when the export holds at least one Mermaid block, the
    /// only case that needs `mmdr` (FR-042).
    pub(super) fn resolve(args: &Args, diagrams: bool) -> anyhow::Result<Self> {
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
        let mmdr = if diagrams {
            Some(tool::resolve("mmdr", "--mmdr", args.mmdr.as_deref())?)
        } else {
            None
        };
        Ok(Self {
            executor,
            pandoc,
            typst,
            mmdr,
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
    /// The path, relative to the capability root, of the required output.
    fn output(self) -> &'static str {
        match self {
            Self::PandocDocx => "out/document.docx",
            Self::PandocTypst => "typ/document.typ",
            Self::TypstPdf => "out/document.pdf",
        }
    }

    fn arguments(self, plan: Plan) -> Vec<String> {
        let Plan {
            with_identity,
            documents,
            approval,
            ..
        } = plan;
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
            // `with_identity` marks a Markdown input. Only a Markdown input has
            // code blocks to keep byte-exact (FR-032) and gets the diagram
            // filter, with a block count of 0 when it has no block (FR-042).
            if with_identity {
                arguments.push("--preserve-tabs".to_string());
                arguments.extend([template[0], template[1]].map(str::to_string));
                arguments.extend(["--lua-filter", diagrams::FILTER].map(str::to_string));
                arguments.push("in/front.md".to_string());
            }
            if documents == 0 {
                arguments.push("in/document.md".to_string());
            }
            for number in 1..=documents {
                arguments.push(format!("in/separator-{number}.md"));
                arguments.push(format!("in/document-{number}.md"));
            }
            if approval {
                arguments.push("in/approval.md".to_string());
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
    /// The approval block, written to `in/approval.md` and passed last.
    pub(super) approval: Option<&'a str>,
    pub(super) metadata: &'a str,
    /// The Mermaid blocks of the export in document order (bundle: input order),
    /// each rendered to an image before pandoc runs (FR-042). Empty when there
    /// is none.
    pub(super) diagrams: Vec<Block>,
    /// Front matter and the template, present for a Markdown input;
    /// a JSON report carries no identity and gets neither.
    pub(super) identity_parts: Option<IdentityParts<'a>>,
    /// The `SOURCE_DATE_EPOCH` of every request: the source commit time, or 0 for
    /// a JSON report, which has no commit.
    pub(super) epoch: i64,
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
    if let Some(approval) = generated.approval {
        files.push(("in/approval.md".to_string(), approval.as_bytes()));
    }
    let filter = diagrams::filter_source(generated.diagrams.len(), image_directory(to));
    if generated.identity_parts.is_some() {
        files.push((diagrams::FILTER.to_string(), filter.as_bytes()));
    }
    if !generated.diagrams.is_empty() {
        for directory in ["in/diagrams", image_directory(to)] {
            fs::create_dir_all(root_path.join(directory))
                .with_context(|| format!("creating {directory}"))?;
        }
    }
    let diagram_files: Vec<(String, &[u8])> = generated
        .diagrams
        .iter()
        .zip(1..)
        .flat_map(|(block, number)| {
            [
                (diagrams::source_file(number), block.source.as_bytes()),
                (diagrams::alt_file(number), block.alt.as_bytes()),
            ]
        })
        .collect();
    for (name, content) in files.into_iter().chain(diagram_files) {
        fs::write(root_path.join(&name), content).with_context(|| format!("writing {name}"))?;
    }

    for (block, number) in generated.diagrams.iter().zip(1..) {
        render_diagram(
            converters,
            &root_path,
            &root_text,
            generated.epoch,
            to,
            block,
            number,
        )?;
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
                approval: generated.approval.is_some(),
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
    approval: bool,
    epoch: i64,
}

/// The image directory of an output format, relative to the capability root:
/// where `mmdr` writes and where the filter reads (FR-042 section B).
fn image_directory(to: Format) -> &'static str {
    match to {
        Format::Pdf => "typ/img",
        Format::Docx => "img",
    }
}

/// Renders block `number` (1-based over the export) with one `mmdr` request. A
/// request that does not complete, or leaves no PNG or an empty one, is an
/// export error that names the block (FR-042 section D).
fn render_diagram(
    converters: &Converters,
    root: &Path,
    root_text: &str,
    epoch: i64,
    to: Format,
    block: &Block,
    number: usize,
) -> anyhow::Result<()> {
    let mmdr = converters.mmdr.as_ref().context("mmdr was not resolved")?;
    let image = format!("{}/{number}.png", image_directory(to));
    let arguments = [
        "-i",
        &diagrams::source_file(number),
        "-o",
        &image,
        "-e",
        "png",
    ]
    .map(str::to_string)
    .to_vec();
    let request = envelope(
        root_text,
        mmdr,
        epoch,
        &arguments,
        json!({"role": "diagram", "path": image, "required": true}),
        60_000,
    );
    let named = |error: anyhow::Error| {
        failure::classified(
            failure::outcome(&error),
            anyhow::anyhow!("{}: {error:#}", block.describe()),
        )
    };
    let result = execute(&converters.executor, &request).map_err(named)?;
    interpret(mmdr, root, &result, cfg!(target_os = "linux")).map_err(named)?;
    if fs::metadata(root.join(&image)).map_or(true, |meta| !meta.is_file() || meta.len() == 0) {
        return Err(named(converter_failure(format!(
            "mmdr reported completion but wrote no {image}"
        ))));
    }
    Ok(())
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

/// The complete FR-032 request for one pandoc or typst step.
fn request(root: &str, tool: &Tool, plan: Plan) -> Value {
    envelope(
        root,
        tool,
        plan.epoch,
        &plan.step.arguments(plan),
        json!({"role": "document", "path": plan.step.output(), "required": true}),
        120_000,
    )
}

/// The FR-032 request field set for `tool`. Fields not in the table are not set.
fn envelope(
    root: &str,
    tool: &Tool,
    epoch: i64,
    arguments: &[String],
    output: Value,
    timeout_millis: u64,
) -> Value {
    let environment = json!({
        "HOME": format!("{root}/out/home"),
        "TMPDIR": format!("{root}/out/tmp"),
        "SOURCE_DATE_EPOCH": epoch.to_string(),
    });
    json!({
        "protocol": "engineering-assurance.producer-execution-request/v1",
        "producer": {
            "name": tool.name,
            "version": "unreported",
            "executable": tool.executable,
        },
        "caller": {"kind": "quire.export", "version": "1"},
        "procedure": "direct",
        "capabilityRoot": root,
        "arguments": arguments
            .iter()
            .map(|value| json!({"kind": "literal", "value": value}))
            .collect::<Vec<_>>(),
        "environment": environment,
        "inputs": [],
        "stdin": {"kind": "null"},
        "outputs": [output],
        "containment": {
            "profile": "process-group-v1",
            "contract": {"kind": "quire.export-containment", "version": "1"},
        },
        "cancellation": {"kind": "disabled"},
        "budget": {
            "timeoutMillis": timeout_millis,
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
