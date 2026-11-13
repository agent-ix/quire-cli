//! `export` ITs (FR-031, FR-032, FR-033).
//!
//! Quire launches only `engineering-assurance`. These tests run the installed
//! `engineering-assurance producer-execute` over shell stand-ins for pandoc and
//! typst, written into a temporary directory, so the repository holds no
//! converter. The ignored tests need the real tools and fail naming the missing
//! one.

mod common;

use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ix_trace_rs::trace;
use serde_json::Value;

use common::{fixture_root, quire, validate_doc, validate_module};

const PANDOC_COPY: &str = r#"out=""; last=""
while [ $# -gt 0 ]; do
  case "$1" in
    --output) out="$2"; shift ;;
    *) last="$1" ;;
  esac
  shift
done
echo pandoc >> "$LOG/calls"
cp in/metadata.yaml "$LOG/metadata.yaml"
cp in/document.md "$LOG/document.md"
cp "$last" "$out"
"#;

const TYPST_COPY: &str = "echo typst >> \"$LOG/calls\"\ncp \"$2\" \"$3\"\n";

fn export_fixture() -> PathBuf {
    fixture_root().join("export/export-fr.md")
}

/// The body of a document after its frontmatter block.
fn body_of(text: &str) -> &str {
    let after_open = &text["---\n".len()..];
    let close = after_open.find("\n---\n").expect("closing fence");
    &after_open[close + "\n---\n".len()..]
}

fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}")).expect("script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("mode");
}

fn real_executor() -> PathBuf {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .map(|dir| dir.join("engineering-assurance"))
        .find(|candidate| candidate.is_file())
        .expect("these tests need `engineering-assurance` on PATH, and none was found")
}

/// A temporary toolbox: stand-in pandoc and typst, and an
/// `engineering-assurance` that records each request before running the real
/// one.
struct Rig {
    dir: tempfile::TempDir,
}

impl Rig {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("rig");
        let root = fs::canonicalize(dir.path()).expect("canonical rig");
        for sub in ["bin", "log", "log/reqs", "work"] {
            fs::create_dir_all(root.join(sub)).expect("rig directory");
        }
        let log = root.join("log");
        let header = format!("LOG={}\n", log.display());
        script(&root.join("bin/pandoc"), &format!("{header}{PANDOC_COPY}"));
        script(&root.join("bin/typst"), &format!("{header}{TYPST_COPY}"));
        let executor = format!(
            "{header}n=$(ls \"$LOG/reqs\" | wc -l)\necho \"$*\" >> \"$LOG/argv\"\ncat > \"$LOG/reqs/$n.json\"\n\
             exec {} producer-execute --request \"$(cat \"$LOG/reqs/$n.json\")\"\n",
            real_executor().display()
        );
        script(&root.join("bin/engineering-assurance"), &executor);
        Self { dir }
    }

    fn root(&self) -> PathBuf {
        fs::canonicalize(self.dir.path()).expect("canonical rig")
    }

    fn bin(&self, name: &str) -> PathBuf {
        self.root().join("bin").join(name)
    }

    fn work(&self, name: &str) -> PathBuf {
        self.root().join("work").join(name)
    }

    /// `quire export <input> --to <to> --output <output>` with every tool named.
    fn export(&self, input: &Path, to: &str, output: &Path) -> Command {
        let mut command = quire();
        command
            .current_dir(self.root().join("work"))
            .arg("export")
            .arg(input)
            .args(["--to", to, "--output"])
            .arg(output)
            .arg("--module")
            .arg(validate_module())
            .arg("--pandoc")
            .arg(self.bin("pandoc"))
            .arg("--typst")
            .arg(self.bin("typst"))
            .arg("--engineering-assurance")
            .arg(self.bin("engineering-assurance"));
        command
    }

    fn requests(&self) -> Vec<Value> {
        let mut files: Vec<_> = fs::read_dir(self.root().join("log/reqs"))
            .expect("requests")
            .map(|entry| entry.expect("entry").path())
            .collect();
        files.sort();
        files
            .iter()
            .map(|path| serde_json::from_slice(&fs::read(path).expect("read")).expect("JSON"))
            .collect()
    }

    fn calls(&self) -> String {
        fs::read_to_string(self.root().join("log/calls")).unwrap_or_default()
    }
}

fn run(mut command: Command) -> Output {
    command.output().expect("quire runs")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn schema_validator() -> jsonschema::JSONSchema {
    let output = Command::new(real_executor())
        .args(["producer-execute", "--schema", "request"])
        .output()
        .expect("schema");
    let schema: Value = serde_json::from_slice(&output.stdout).expect("schema JSON");
    jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&schema)
        .expect("schema compiles")
}

fn literals(request: &Value) -> Vec<&str> {
    request["arguments"]
        .as_array()
        .expect("arguments")
        .iter()
        .map(|argument| argument["value"].as_str().expect("literal"))
        .collect()
}

const PANDOC_ARGUMENTS: [&str; 11] = [
    "--sandbox",
    "--standalone",
    "--from",
    "markdown-yaml_metadata_block",
    "--to",
    "TO",
    "--output",
    "OUT",
    "--metadata-file",
    "in/metadata.yaml",
    "in/document.md",
];

fn pandoc_arguments(to: &str, output: &str) -> Vec<String> {
    PANDOC_ARGUMENTS
        .iter()
        .map(|value| match *value {
            "TO" => to.to_string(),
            "OUT" => output.to_string(),
            other => other.to_string(),
        })
        .collect()
}

#[trace(
    "FR-032-AC-1",
    "FR-032-AC-2",
    "FR-032-AC-3",
    "FR-032-AC-5",
    "FR-031-AC-6"
)]
#[test]
fn docx_export_makes_exactly_one_valid_pandoc_request() {
    let rig = Rig::new();
    let output = rig.work("out.docx");
    let result = run(rig.export(&export_fixture(), "docx", &output));
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(result.stdout.is_empty());

    let requests = rig.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert!(schema_validator().is_valid(request), "{request}");
    assert_eq!(request["producer"]["name"], "pandoc");
    assert_eq!(request["producer"]["version"], "unreported");
    assert_eq!(
        request["producer"]["executable"],
        rig.bin("pandoc").display().to_string()
    );
    assert_eq!(request["caller"]["kind"], "quire.export");
    assert_eq!(request["procedure"], "direct");
    assert_eq!(request["inputs"], serde_json::json!([]));
    assert_eq!(request["stdin"], serde_json::json!({"kind": "null"}));
    assert_eq!(
        request["cancellation"],
        serde_json::json!({"kind": "disabled"})
    );
    assert_eq!(request["caller"]["version"], "1");
    assert_eq!(
        request["containment"],
        serde_json::json!({
            "profile": "process-group-v1",
            "contract": {"kind": "quire.export-containment", "version": "1"}
        })
    );
    assert_eq!(
        request["response"],
        serde_json::json!({
            "protocol": {"kind": "engineering-assurance.process-evidence", "version": "1"},
            "adapter": {"kind": "engineering-assurance.cli-process-evidence-adapter", "version": "1"},
            "exitCodes": {"kind": "exact", "codes": [0]}
        })
    );
    assert_eq!(
        request["budget"],
        serde_json::json!({
            "timeoutMillis": 120_000,
            "maxStdoutBytes": 1_048_576,
            "maxStderrBytes": 1_048_576,
            "maxInputBytes": 268_435_456,
            "maxOutputArtifacts": 1,
            "maxOutputBytes": 268_435_456,
            "maxDescendants": 64,
            "maxConcurrency": 1
        })
    );
    let mut keys: Vec<_> = request
        .as_object()
        .expect("object")
        .keys()
        .cloned()
        .collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "arguments",
            "budget",
            "caller",
            "cancellation",
            "capabilityRoot",
            "containment",
            "environment",
            "inputs",
            "outputs",
            "procedure",
            "producer",
            "protocol",
            "response",
            "stdin"
        ]
    );
    assert_eq!(
        request["outputs"],
        serde_json::json!([{"role": "document", "path": "out/document.docx", "required": true}])
    );
    let root = request["capabilityRoot"].as_str().expect("root");
    assert_eq!(
        request["environment"],
        serde_json::json!({"HOME": format!("{root}/out/home"), "TMPDIR": format!("{root}/out/tmp")})
    );
    assert_eq!(
        literals(request),
        pandoc_arguments("docx", "out/document.docx")
    );
    assert_eq!(rig.calls(), "pandoc\n");
    // quire passes exactly `producer-execute --request -`, the request on stdin.
    assert_eq!(
        fs::read_to_string(rig.root().join("log/argv")).expect("argv"),
        "producer-execute --request -\n"
    );

    // The converter input is the committed body, and the output is its copy.
    let text = fs::read_to_string(export_fixture()).expect("fixture");
    let log = rig.root().join("log");
    assert_eq!(
        fs::read_to_string(log.join("document.md")).expect("input"),
        body_of(&text)
    );
    assert_eq!(fs::read_to_string(&output).expect("output"), body_of(&text));
    assert!(body_of(&text).contains("```mermaid"));
}

#[trace("FR-031-AC-7")]
#[test]
fn only_the_title_reaches_the_converter_as_metadata() {
    let rig = Rig::new();
    let result = run(rig.export(&export_fixture(), "docx", &rig.work("out.docx")));
    assert!(result.status.success(), "{}", stderr(&result));
    let metadata = fs::read_to_string(rig.root().join("log/metadata.yaml")).expect("metadata");
    assert_eq!(metadata, "{\"title\":\"Fictional export fixture\"}\n");
}

#[trace("FR-032-AC-1", "FR-032-AC-3", "FR-032-AC-6")]
#[test]
fn pdf_export_makes_a_pandoc_then_a_typst_request_over_one_root() {
    let rig = Rig::new();
    let output = rig.work("out.pdf");
    let result = run(rig.export(&export_fixture(), "pdf", &output));
    assert!(result.status.success(), "{}", stderr(&result));

    let requests = rig.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["producer"]["name"], "pandoc");
    assert_eq!(
        literals(&requests[0]),
        pandoc_arguments("typst", "typ/document.typ")
    );
    assert_eq!(requests[1]["producer"]["name"], "typst");
    assert_eq!(
        literals(&requests[1]),
        ["compile", "typ/document.typ", "out/document.pdf"]
    );
    assert_eq!(requests[0]["capabilityRoot"], requests[1]["capabilityRoot"]);
    assert_eq!(
        requests[1]["outputs"],
        serde_json::json!([{"role": "document", "path": "out/document.pdf", "required": true}])
    );
    assert_eq!(rig.calls(), "pandoc\ntypst\n");
    let root = requests[0]["capabilityRoot"].as_str().expect("root");
    assert!(!Path::new(root).exists(), "capability root must be removed");
    assert!(output.is_file());
}

#[trace("FR-032-AC-2")]
#[test]
fn a_converter_behind_a_symbolic_link_is_passed_as_its_target() {
    let rig = Rig::new();
    let link = rig.root().join("bin/pandoc-link");
    symlink(rig.bin("pandoc"), &link).expect("symlink");
    let mut command = quire();
    command
        .current_dir(rig.root().join("work"))
        .arg("export")
        .arg(export_fixture())
        .args(["--to", "docx", "--output"])
        .arg(rig.work("out.docx"))
        .arg("--module")
        .arg(validate_module())
        .arg("--pandoc")
        .arg(&link)
        .arg("--engineering-assurance")
        .arg(rig.bin("engineering-assurance"));
    let result = run(command);
    assert!(result.status.success(), "{}", stderr(&result));
    assert_eq!(
        rig.requests()[0]["producer"]["executable"],
        rig.bin("pandoc").display().to_string()
    );

    let directory_link = rig.root().join("bin/pandoc-dir");
    symlink(rig.root().join("log"), &directory_link).expect("symlink");
    let mut command = quire();
    command
        .current_dir(rig.root().join("work"))
        .arg("export")
        .arg(export_fixture())
        .args(["--to", "docx", "--output"])
        .arg(rig.work("other.docx"))
        .arg("--module")
        .arg(validate_module())
        .arg("--pandoc")
        .arg(&directory_link)
        .arg("--engineering-assurance")
        .arg(rig.bin("engineering-assurance"));
    let result = run(command);
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("pandoc"));
}

#[trace("FR-032-AC-4", "FR-033-AC-4")]
#[test]
fn pandoc_failure_exits_4_with_its_stderr_and_no_typst_request() {
    let rig = Rig::new();
    script(
        &rig.bin("pandoc"),
        "echo 'pandoc: fictional conversion failure' >&2\nexit 7\n",
    );
    let output = rig.work("out.pdf");
    let result = run(rig.export(&export_fixture(), "pdf", &output));
    assert_eq!(result.status.code(), Some(4));
    let message = stderr(&result);
    assert!(
        message.contains("pandoc: fictional conversion failure"),
        "{message}"
    );
    assert!(message.contains("exit_code 7"), "{message}");
    assert_eq!(rig.requests().len(), 1);
    assert!(!output.exists());
}

#[trace("FR-033-AC-5", "FR-032-AC-6")]
#[test]
fn exit_zero_without_the_output_exits_4_and_removes_the_root() {
    let rig = Rig::new();
    script(&rig.bin("pandoc"), "exit 0\n");
    let output = rig.work("out.docx");
    let result = run(rig.export(&export_fixture(), "docx", &output));
    assert_eq!(result.status.code(), Some(4));
    assert!(!output.exists());
    let root = rig.requests()[0]["capabilityRoot"]
        .as_str()
        .expect("root")
        .to_string();
    assert!(!Path::new(&root).exists());
}

#[trace("FR-033-AC-1", "FR-033-AC-2", "FR-033-AC-3")]
#[test]
fn a_missing_tool_exits_3_naming_it_and_its_flag_before_any_conversion() {
    let rig = Rig::new();
    let output = rig.work("out.docx");

    let mut command = rig.export(&export_fixture(), "docx", &output);
    command.env("PATH", "");
    fs::remove_file(rig.bin("pandoc")).expect("remove pandoc");
    let result = run(command);
    assert_eq!(result.status.code(), Some(3));
    let message = stderr(&result);
    assert!(
        message.contains("pandoc") && message.contains("--pandoc"),
        "{message}"
    );
    assert!(!output.exists());

    let rig = Rig::new();
    fs::remove_file(rig.bin("typst")).expect("remove typst");
    let result = run(rig.export(&export_fixture(), "pdf", &rig.work("out.pdf")));
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("typst"));
    assert!(
        rig.requests().is_empty(),
        "no pandoc request may precede the check"
    );

    let rig = Rig::new();
    fs::remove_file(rig.bin("engineering-assurance")).expect("remove executor");
    let result = run(rig.export(&export_fixture(), "docx", &rig.work("out.docx")));
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("engineering-assurance"));
}

#[trace("FR-033-AC-8")]
#[test]
fn executor_errors_and_containment_failures_exit_4() {
    let rig = Rig::new();
    script(
        &rig.bin("engineering-assurance"),
        "cat >/dev/null\n\
         echo '{\"protocol\":\"engineering-assurance.error/v1\",\"capability\":\"producer-execute\",\"code\":\"producer_execute_request_invalid\",\"message\":\"fictional\"}'\n\
         exit 2\n",
    );
    let result = run(rig.export(&export_fixture(), "docx", &rig.work("out.docx")));
    assert_eq!(result.status.code(), Some(4));
    let message = stderr(&result);
    assert!(
        message.contains("producer_execute_request_invalid"),
        "{message}"
    );
    assert!(message.contains("may not match"), "{message}");

    script(
        &rig.bin("engineering-assurance"),
        "cat >/dev/null\n\
         echo '{\"protocol\":\"engineering-assurance.producer-execution-result/v1\",\"process\":null,\"state\":{\"kind\":\"containment_failure\"}}'\n",
    );
    let result = run(rig.export(&export_fixture(), "docx", &rig.work("out.docx")));
    assert_eq!(result.status.code(), Some(4));
    assert!(stderr(&result).contains("containment_failure"));
}

#[trace("FR-033-AC-6")]
#[test]
fn invalid_inputs_exit_3_without_starting_a_converter() {
    let rig = Rig::new();
    let output = rig.work("out.docx");

    let missing = rig.work("absent.md");
    let result = run(rig.export(&missing, "docx", &output));
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("absent.md"));

    let text = rig.work("notes.txt");
    fs::write(&text, "text").expect("txt");
    let result = run(rig.export(&text, "docx", &output));
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("notes.txt"));

    let binary = rig.work("binary.md");
    fs::write(&binary, [0xff, 0xfe, 0x00]).expect("binary");
    let result = run(rig.export(&binary, "docx", &output));
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("binary.md"));

    assert!(rig.requests().is_empty());
    assert!(!output.exists());
}

#[trace("FR-031-AC-3")]
#[test]
fn refused_flag_combinations_exit_3_naming_the_flag() {
    let rig = Rig::new();
    let output = rig.work("out.docx");

    let mut command = rig.export(&export_fixture(), "docx", &output);
    command.args(["--kind", "table"]);
    let result = run(command);
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("--kind"));

    let json = rig.work("report.json");
    fs::write(&json, "{}").expect("json");
    let mut command = quire();
    command
        .arg("export")
        .arg(&json)
        .args(["--to", "docx", "--output"])
        .arg(&output);
    let result = run(command);
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("--kind"));

    let mut command = quire();
    command
        .arg("export")
        .arg(&json)
        .args(["--to", "docx", "--kind", "table", "--module"])
        .arg(validate_module())
        .arg("--output")
        .arg(&output);
    let result = run(command);
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("--module"));
    assert!(rig.requests().is_empty());
}

#[trace("FR-031-AC-2")]
#[test]
fn an_invalid_artifact_exits_3_and_the_same_file_passes_a_module_that_admits_it() {
    let rig = Rig::new();
    let output = rig.work("out.docx");
    let broken = validate_doc("broken-fr.md");
    let result = run(rig.export(&broken, "docx", &output));
    assert_eq!(result.status.code(), Some(3));
    assert!(!output.exists());
    assert!(rig.requests().is_empty());
    assert!(rig.calls().is_empty());

    let valid = run(rig.export(&export_fixture(), "docx", &output));
    assert!(valid.status.success(), "{}", stderr(&valid));
}

#[trace("FR-033-AC-7")]
#[test]
fn a_failed_export_leaves_an_existing_output_unchanged() {
    let rig = Rig::new();
    script(&rig.bin("pandoc"), "exit 9\n");
    let output = rig.work("out.docx");
    fs::write(&output, "previous bytes").expect("previous");
    let result = run(rig.export(&export_fixture(), "docx", &output));
    assert_eq!(result.status.code(), Some(4));
    assert_eq!(fs::read(&output).expect("output"), b"previous bytes");
}

#[trace("FR-033-AC-9")]
#[test]
fn output_and_permission_refusals_start_no_converter() {
    let rig = Rig::new();

    let unreadable = rig.work("locked.md");
    fs::copy(export_fixture(), &unreadable).expect("copy");
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).expect("chmod");
    let result = run(rig.export(&unreadable, "docx", &rig.work("out.docx")));
    assert_eq!(result.status.code(), Some(2));
    assert!(stderr(&result).contains("locked.md"));

    let same = rig.work("same.md");
    fs::copy(export_fixture(), &same).expect("copy");
    let before = fs::read(&same).expect("before");
    let result = run(rig.export(&same, "docx", &same));
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(fs::read(&same).expect("after"), before);

    let result = run(rig.export(&export_fixture(), "docx", &rig.work("nodir/out.docx")));
    assert_eq!(result.status.code(), Some(4));

    let directory = rig.work("adir");
    fs::create_dir(&directory).expect("dir");
    let result = run(rig.export(&export_fixture(), "docx", &directory));
    assert_eq!(result.status.code(), Some(4));

    assert!(rig.requests().is_empty());
}

#[trace("FR-033-AC-10")]
#[test]
fn input_text_never_reaches_a_request_argument() {
    let rig = Rig::new();
    let hostile = rig.work("hostile.md");
    let text = fs::read_to_string(export_fixture())
        .expect("fixture")
        .replace("# [FR-001] Ember Overview", "# [FR-001] --output=/tmp/x");
    fs::write(&hostile, &text).expect("hostile");
    let result = run(rig.export(&hostile, "docx", &rig.work("out.docx")));
    assert!(result.status.success(), "{}", stderr(&result));
    let log = rig.root().join("log");
    assert!(fs::read_to_string(log.join("document.md"))
        .expect("input")
        .contains("--output=/tmp/x"));
    assert_eq!(
        literals(&rig.requests()[0]),
        pandoc_arguments("docx", "out/document.docx")
    );
}

#[trace("FR-031-AC-8")]
#[test]
fn an_export_writes_no_file_other_than_the_output() {
    let rig = Rig::new();
    let input = rig.work("in.md");
    fs::copy(export_fixture(), &input).expect("copy");
    let result = run(rig.export(&input, "pdf", &rig.work("out.pdf")));
    assert!(result.status.success(), "{}", stderr(&result));
    let mut names: Vec<_> = fs::read_dir(rig.root().join("work"))
        .expect("work")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    assert_eq!(names, ["in.md", "out.pdf"]);
}

/// An `engineering-assurance` stand-in that answers every request with `state`.
fn fixed_state_executor(rig: &Rig, state: &str) {
    script(
        &rig.bin("engineering-assurance"),
        &format!(
            "cat >/dev/null\n\
             echo '{{\"protocol\":\"engineering-assurance.producer-execution-result/v1\",\"process\":null,\"state\":{state}}}'\n"
        ),
    );
}

#[trace("FR-033-AC-4")]
#[test]
fn a_timed_out_converter_exits_4_naming_the_state() {
    let rig = Rig::new();
    fixed_state_executor(&rig, "{\"kind\":\"timed_out\"}");
    let output = rig.work("out.docx");
    let result = run(rig.export(&export_fixture(), "docx", &output));
    assert_eq!(result.status.code(), Some(4));
    assert!(stderr(&result).contains("timed_out"), "{}", stderr(&result));
    assert!(!output.exists());
}

#[trace("FR-032-AC-2")]
#[test]
fn a_refused_request_names_the_canonical_executable_and_the_capability_root() {
    let rig = Rig::new();
    fixed_state_executor(&rig, "{\"kind\":\"refused\",\"reason\":\"executable\"}");
    let result = run(rig.export(&export_fixture(), "docx", &rig.work("out.docx")));
    assert_eq!(result.status.code(), Some(4));
    let message = stderr(&result);
    assert!(message.contains("refused (executable)"), "{message}");
    assert!(
        message.contains(&rig.bin("pandoc").display().to_string()),
        "{message}"
    );
    assert!(message.contains("quire-export-"), "{message}");
}

#[trace("FR-033-AC-9")]
#[test]
fn an_unwritable_output_directory_is_refused_before_any_conversion() {
    let rig = Rig::new();
    let locked = rig.work("locked");
    fs::create_dir(&locked).expect("dir");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).expect("chmod");
    let result = run(rig.export(&export_fixture(), "docx", &locked.join("out.docx")));
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("chmod back");
    assert_eq!(result.status.code(), Some(4));
    assert!(
        stderr(&result).contains("not writable"),
        "{}",
        stderr(&result)
    );
    assert!(rig.requests().is_empty());
}

#[trace("FR-032-AC-2")]
#[test]
fn a_converter_path_that_is_not_utf8_is_refused_clearly() {
    use std::os::unix::ffi::OsStrExt;
    let rig = Rig::new();
    let odd = rig
        .root()
        .join(std::ffi::OsStr::from_bytes(b"bin/pandoc-\xff"));
    fs::copy(rig.bin("pandoc"), &odd).expect("copy");
    let mut command = quire();
    command
        .current_dir(rig.root().join("work"))
        .arg("export")
        .arg(export_fixture())
        .args(["--to", "docx", "--output"])
        .arg(rig.work("out.docx"))
        .arg("--module")
        .arg(validate_module())
        .arg("--pandoc")
        .arg(&odd)
        .arg("--engineering-assurance")
        .arg(rig.bin("engineering-assurance"));
    let result = run(command);
    assert_eq!(result.status.code(), Some(3));
    assert!(stderr(&result).contains("UTF-8"), "{}", stderr(&result));
    assert!(rig.requests().is_empty());
}

#[trace("NFR-010-AC-3")]
#[test]
fn the_export_adds_no_network_client_crate() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("manifest");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|rest| rest.split("\n[").next())
        .expect("dependencies table");
    for client in [
        "reqwest",
        "hyper",
        "ureq",
        "isahc",
        "curl",
        "attohttpc",
        "surf",
        "awc",
    ] {
        assert!(
            !dependencies.contains(&format!("\n{client} ")),
            "network client crate {client} is a direct dependency"
        );
    }
}

fn require_tool(name: &str) -> PathBuf {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("this test needs the real tool `{name}` on PATH; none was found"))
}

fn real_export(to: &str, output: &Path) -> Output {
    require_tool("pandoc");
    require_tool("typst");
    let mut command = quire();
    command
        .arg("export")
        .arg(export_fixture())
        .args(["--to", to, "--output"])
        .arg(output)
        .arg("--module")
        .arg(validate_module());
    run(command)
}

fn docx_text(path: &Path) -> String {
    require_tool("unzip");
    let output = Command::new("unzip")
        .args(["-p"])
        .arg(path)
        .arg("word/document.xml")
        .output()
        .expect("unzip runs");
    assert!(output.status.success(), "{}", stderr(&output));
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn pdf_text(path: &Path) -> String {
    let pdftotext = require_tool("pdftotext");
    let output = Command::new(pdftotext)
        .arg(path)
        .arg("-")
        .output()
        .expect("pdftotext runs");
    assert!(output.status.success(), "{}", stderr(&output));
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[trace("FR-031-AC-1", "FR-032-AC-7")]
#[test]
#[ignore = "needs real pandoc, typst, unzip and pdftotext on PATH"]
fn real_converters_keep_the_fixture_headings_and_code() {
    let dir = tempfile::tempdir().expect("dir");
    let docx = dir.path().join("out.docx");
    let pdf = dir.path().join("out.pdf");
    for (to, path) in [("docx", &docx), ("pdf", &pdf)] {
        let result = real_export(to, path);
        assert!(result.status.success(), "{}", stderr(&result));
    }
    assert_eq!(&fs::read(&pdf).expect("pdf")[..4], b"%PDF");
    assert_eq!(&fs::read(&docx).expect("docx")[..2], b"PK");
    for text in [docx_text(&docx), pdf_text(&pdf)] {
        for needle in [
            "Ember Overview",
            "Description",
            "Specification",
            "Acceptance Criteria",
            "quill_fixture",
            "first fixture item",
            "second fixture item",
            "FR-001-AC-1",
            "It does the other thing",
        ] {
            assert!(text.contains(needle), "missing {needle:?} in {text}");
        }
    }
}

fn json_fixture(name: &str) -> PathBuf {
    fixture_root().join("export/json").join(name)
}

/// Markdown-literal text as the exporter escapes it: ASCII punctuation is
/// backslash-escaped.
fn escaped(text: &str) -> String {
    text.chars()
        .flat_map(|ch| {
            let prefix = ch.is_ascii_punctuation().then_some('\\');
            prefix.into_iter().chain([ch])
        })
        .collect()
}

impl Rig {
    /// `quire export <input> --kind <kind> --to <to> --output <output>` with
    /// every tool named.
    fn export_json(&self, input: &Path, kind: &str, to: &str, output: &Path) -> Command {
        let mut command = quire();
        command
            .current_dir(self.root().join("work"))
            .arg("export")
            .arg(input)
            .args(["--kind", kind, "--to", to, "--output"])
            .arg(output)
            .arg("--pandoc")
            .arg(self.bin("pandoc"))
            .arg("--typst")
            .arg(self.bin("typst"))
            .arg("--engineering-assurance")
            .arg(self.bin("engineering-assurance"));
        command
    }

    /// Exports a JSON fixture and returns the Markdown the converter received.
    fn json_markdown(&self, fixture: &str, kind: &str, to: &str) -> String {
        let output = self.work("out.bin");
        let result = run(self.export_json(&json_fixture(fixture), kind, to, &output));
        assert!(result.status.success(), "{}", stderr(&result));
        assert!(result.stdout.is_empty());
        let markdown = fs::read_to_string(self.root().join("log/document.md")).expect("input");
        assert_eq!(fs::read_to_string(&output).expect("output"), markdown);
        markdown
    }
}

/// The pipe-table body rows (not header or separator) under `heading`.
fn table_rows<'a>(markdown: &'a str, heading: &str) -> Vec<&'a str> {
    let start = format!("## {}\n", escaped(heading));
    let after = markdown.split_once(&start).expect("heading").1;
    let block = after
        .trim_start()
        .split("\n\n")
        .next()
        .expect("table block");
    block.lines().skip(2).collect()
}

#[trace("FR-031-AC-4")]
#[test]
fn a_matrix_report_yields_one_table_per_document_with_every_criterion() {
    let rig = Rig::new();
    let markdown = rig.json_markdown("matrix.json", "matrix", "docx");
    for id in ["FR-901-AC-1", "FR-901-AC-2", "FR-902-AC-1"] {
        assert!(markdown.contains(&escaped(id)), "missing {id}");
    }
    assert_eq!(
        table_rows(&markdown, "spec/functional/FR-901-ember.md").len(),
        2
    );
    assert_eq!(
        table_rows(&markdown, "spec/functional/FR-902-cinder.md").len(),
        1
    );
    // The untagged rows and the late `criticality` member are kept.
    assert!(markdown.contains("untagged"));
    assert!(markdown.contains("criticality"));
    assert!(!markdown.contains("No obligations matched"));
}

#[trace("FR-031-AC-4")]
#[test]
fn a_matrix_report_without_coverage_matrix_yields_the_zero_population_line() {
    let rig = Rig::new();
    let markdown = rig.json_markdown("matrix-empty.json", "matrix", "pdf");
    assert_eq!(markdown, "No obligations matched this scope.\n\n");
}

#[trace("FR-031-AC-5")]
#[test]
fn a_table_report_keeps_every_member_and_nested_values_as_compact_json() {
    let rig = Rig::new();
    let markdown = rig.json_markdown("table.json", "table", "docx");
    assert!(markdown.contains(&escaped(r#"{"nested":[1,2],"ok":true}"#)));
    for member in [
        "ROW-1",
        "ROW-2",
        "ROW-3",
        "late member",
        "OTHER-1",
        "extra",
        "detail",
    ] {
        assert!(markdown.contains(&escaped(member)), "missing {member}");
    }
    assert!(!markdown.contains("ignored scalar"));
    assert_eq!(table_rows(&markdown, "items").len(), 3);
    assert_eq!(table_rows(&markdown, "others").len(), 1);
}

#[trace("FR-031-AC-4", "FR-031-AC-5")]
#[test]
fn report_text_cannot_inject_markup_or_split_a_row() {
    let rig = Rig::new();
    let markdown = rig.json_markdown("matrix.json", "matrix", "docx");
    // Literal expectations of the generated Markdown, not the escape rule.
    assert!(
        markdown.contains(r"| Ember \| glows \`red\` |"),
        "{markdown}"
    );
    assert!(
        markdown.contains(r"Ember dims \<w\:p\>raw\<\/w\:p\>"),
        "{markdown}"
    );
    assert!(
        markdown.contains(r"Cinder \#strong\[cools\] \$x\$ \@ref"),
        "{markdown}"
    );
    assert!(!markdown.contains("<w:p>"));
    assert!(!markdown.replace("\\`", "").contains('`'));
    let rows = table_rows(&markdown, "spec/functional/FR-901-ember.md");
    assert_eq!(rows.len(), 2);
    assert!(rows
        .iter()
        .all(|row| row.starts_with("| ") && row.ends_with(" |")));
}

/// The header cells of the table under `heading`.
fn table_header(markdown: &str, heading: &str) -> Vec<String> {
    let start = format!("## {}\n", escaped(heading));
    let after = markdown.split_once(&start).expect("heading").1;
    let line = after.trim_start().lines().next().expect("header");
    line.trim_matches(|c| c == '|' || c == ' ')
        .split(" | ")
        .map(str::to_string)
        .collect()
}

/// The dash count of each column in the separator row of the table under `heading`.
fn separator_widths(markdown: &str, heading: &str) -> Vec<usize> {
    let start = format!("## {}\n", escaped(heading));
    let after = markdown.split_once(&start).expect("heading").1;
    let line = after.trim_start().lines().nth(1).expect("separator");
    line.trim_matches(|c| c == '|' || c == ' ')
        .split(" | ")
        .map(str::len)
        .collect()
}

#[trace("FR-031-AC-12")]
#[test]
fn rmm_and_soup_columns_are_fixed_in_spec_order_whatever_the_key_order() {
    let rig = Rig::new();
    let rmm = rig.json_markdown("rmm.json", "rmm", "docx");
    let expected: Vec<String> = [
        "clauseId",
        "force",
        "applicability",
        "compliance",
        "reason",
        "evidenceRefs",
        "tailoringDescription",
        "tailoringRationale",
    ]
    .iter()
    .map(|name| escaped(name))
    .collect();
    assert_eq!(table_header(&rmm, "fict/ember-std@1"), expected);
    let soup = rig.json_markdown("soup.json", "soup", "docx");
    let header = table_header(&soup, "rows");
    assert_eq!(header.len(), 14);
    assert_eq!(header[0], "purl");
    assert_eq!(header[1], "id");
    assert_eq!(header[13], escaped("no_advisory_reported_by"));
    // Members outside the spec list follow, sorted by name.
    let table = rig.json_markdown("table.json", "table", "docx");
    assert_eq!(table_header(&table, "items"), ["detail", "extra", "id"]);
}

#[trace("FR-031-AC-13")]
#[test]
fn a_wide_table_gives_each_column_a_relative_width_of_its_content() {
    let rig = Rig::new();
    let soup = rig.json_markdown("soup.json", "soup", "pdf");
    let header = table_header(&soup, "rows");
    let widths = separator_widths(&soup, "rows");
    assert_eq!(widths.len(), header.len());
    assert!(
        widths.iter().all(|width| (3..=48).contains(width)),
        "{widths:?}"
    );
    let width_of = |name: &str| {
        let index = header
            .iter()
            .position(|cell| *cell == escaped(name))
            .expect(name);
        widths[index]
    };
    // "SOUP-CINDER" is 11 characters, "version" 7, and the long advisories cell is capped.
    assert_eq!(width_of("id"), 11);
    assert_eq!(width_of("version"), 7);
    assert_eq!(width_of("advisories"), 48);
}

#[trace("FR-031-AC-9", "FR-031-AC-11")]
#[test]
fn an_rmm_report_yields_its_rows_as_of_line_and_unused_facts() {
    let rig = Rig::new();
    let markdown = rig.json_markdown("rmm.json", "rmm", "docx");
    assert!(markdown.contains(&format!("As of: {}", escaped("2026-11-12T00:00:00Z"))));
    assert_eq!(table_rows(&markdown, "fict/ember-std@1").len(), 3);
    assert_eq!(table_rows(&markdown, "unusedFacts").len(), 1);
    for needle in [
        "EMB-4.1",
        "EMB-4.2",
        "EMB-4.3",
        "FACT-UNUSED-1",
        "undischarged",
    ] {
        assert!(markdown.contains(&escaped(needle)), "missing {needle}");
    }
}

#[trace("FR-031-AC-9")]
#[test]
fn an_rmm_pack_yields_the_index_and_one_table_per_matrix() {
    let rig = Rig::new();
    let markdown = rig.json_markdown("rmm-pack.json", "rmm", "docx");
    assert!(markdown.starts_with("Pack: ember\\-pack\n\n"), "{markdown}");
    assert!(markdown.contains(&format!("As of: {}", escaped("2026-11-12T00:00:00Z"))));
    // The context the verdicts were evaluated against.
    assert_eq!(table_rows(&markdown, "context"), ["| class | b |"]);
    // The index shows each set as its key, like the matrix headings.
    assert_eq!(
        table_rows(&markdown, "index"),
        [
            format!("| {} | 1 |", escaped("fict/set-a@1")),
            format!("| {} | 2 |", escaped("fict/set-b@2")),
        ]
    );
    assert_eq!(table_rows(&markdown, "fict/set-a@1").len(), 1);
    assert_eq!(table_rows(&markdown, "fict/set-b@2").len(), 2);
    // An unusedFacts table is labelled with its set.
    assert_eq!(table_rows(&markdown, "unusedFacts: fict/set-b@2").len(), 1);
    for id in ["PA-1", "PB-1", "PB-2", "PB-UNUSED-1"] {
        assert!(markdown.contains(&escaped(id)), "missing {id}");
    }
}

#[trace("FR-031-AC-9", "FR-031-AC-11")]
#[test]
fn a_soup_report_yields_rows_scans_unmatched_and_unreadable_scans() {
    let rig = Rig::new();
    let markdown = rig.json_markdown("soup.json", "soup", "pdf");
    assert!(markdown.starts_with(&format!("As of: {}", escaped("2026-11-12"))));
    assert_eq!(table_rows(&markdown, "rows").len(), 2);
    assert_eq!(table_rows(&markdown, "unmatched").len(), 1);
    assert_eq!(table_rows(&markdown, "scans").len(), 1);
    for needle in [
        "SOUP-EMBER",
        "SOUP-CINDER",
        "RUSTSEC-UNMATCHED-9",
        "SCAN-SUITE-7",
    ] {
        assert!(markdown.contains(&escaped(needle)), "missing {needle}");
    }
    let list = markdown
        .split_once(&format!("## {}\n\n", escaped("unreadable_scans")))
        .expect("list")
        .1;
    for path in ["scans/broken-scan-1.json", "scans/broken-scan-2.json"] {
        assert!(
            list.contains(&format!("- {}\n", escaped(path))),
            "missing {path}"
        );
    }
}

#[trace("FR-031-AC-10", "FR-033-AC-6")]
#[test]
fn json_that_does_not_match_its_kind_exits_3_without_a_converter() {
    let rig = Rig::new();
    let cases = [
        ("soup", r#"{"as_of": "x", "unmatched": []}"#),
        ("soup", r#"{"rows": {"a": 1}}"#),
        ("rmm", r#"{"rows": []}"#),
        (
            "rmm",
            r#"{"schemaVersion": "requirements-mapping-v2", "rows": []}"#,
        ),
        (
            "rmm",
            r#"{"schemaVersion": "requirements-mapping-pack-v1", "index": []}"#,
        ),
        (
            "rmm",
            r#"{"schemaVersion": "requirements-mapping-pack-v1", "matrices": []}"#,
        ),
        (
            "rmm",
            r#"{"schemaVersion": "requirements-mapping-pack-v1", "index": [],
                "matrices": [{"schemaVersion": "requirements-mapping-v1"}]}"#,
        ),
        ("matrix", r#"{"coverage_matrix": 5}"#),
        ("matrix", r#"{"coverage_matrix": [{"document": "d"}]}"#),
        ("matrix", "[]"),
        ("matrix", "{not json"),
        ("table", r#"{"a": 1, "b": [1, 2]}"#),
        (
            "table",
            r#"{"items": [{"id": "A"}, 5], "others": [{"id": "B"}]}"#,
        ),
    ];
    for (kind, text) in cases {
        let input = rig.work("bad.json");
        fs::write(&input, text).expect("input");
        let output = rig.work("out.docx");
        let result = run(rig.export_json(&input, kind, "docx", &output));
        assert_eq!(
            result.status.code(),
            Some(3),
            "{kind} {text}: {}",
            stderr(&result)
        );
        assert!(stderr(&result).contains("bad.json"), "{}", stderr(&result));
        // A shape diagnostic names the flag value as typed, not a Rust type name.
        if stderr(&result).contains("does not match") {
            assert!(
                stderr(&result).contains(&format!("--kind {kind}:")),
                "{}",
                stderr(&result)
            );
        }
        assert!(!output.exists());
        assert_eq!(rig.calls(), "", "{kind} {text}");
    }
}

/// The character data of an XML text: tags removed, the five entities decoded.
fn strip_tags(xml: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn require_real_tools() {
    for tool in ["pandoc", "typst", "unzip", "pdftotext"] {
        require_tool(tool);
    }
}

#[trace("FR-031-AC-4", "FR-031-AC-9", "FR-031-AC-11")]
#[test]
#[ignore = "needs real pandoc, typst, unzip and pdftotext on PATH"]
fn real_converters_keep_every_report_row_id() {
    require_real_tools();
    let dir = tempfile::tempdir().expect("dir");
    let cases: [(&str, &str, &[&str]); 4] = [
        (
            "matrix.json",
            "matrix",
            &[
                "FR-901-AC-1",
                "FR-901-AC-2",
                "FR-902-AC-1",
                // Hostile cell text must come out literal.
                "Ember | glows `red`",
                "Ember dims <w:p>raw</w:p>",
                "Cinder #strong[cools] $x$ @ref",
            ],
        ),
        (
            "rmm.json",
            "rmm",
            &[
                "EMB-4.1",
                "EMB-4.2",
                "EMB-4.3",
                "FACT-UNUSED-1",
                "fict/ember-std@1",
                "As of: 2026-11-12",
            ],
        ),
        (
            "rmm-pack.json",
            "rmm",
            &[
                "PA-1",
                "PB-1",
                "PB-2",
                "PB-UNUSED-1",
                "Pack: ember-pack",
                "As of: 2026-11-12",
                // The index table (its headers occur nowhere else) and the context table.
                "clauseSet",
                "rowCount",
                "class",
            ],
        ),
        (
            "soup.json",
            "soup",
            &[
                "SOUP-EMBER",
                "SOUP-CINDER",
                "RUSTSEC-UNMATCHED-9",
                "SCAN-SUITE-7",
                "As of: 2026-11-12",
                "broken-scan-1.json",
                "broken-scan-2.json",
            ],
        ),
    ];
    for (fixture, kind, needles) in cases {
        for to in ["docx", "pdf"] {
            let output = dir.path().join(format!("{fixture}.{to}"));
            let mut command = quire();
            command
                .arg("export")
                .arg(json_fixture(fixture))
                .args(["--kind", kind, "--to", to, "--output"])
                .arg(&output);
            let result = run(command);
            assert!(result.status.success(), "{}", stderr(&result));
            let text = if to == "docx" {
                strip_tags(&docx_text(&output))
            } else {
                // PDF text wraps long tokens; compare with whitespace removed.
                pdf_text(&output)
            };
            let squashed: String = text.split_whitespace().collect();
            for needle in needles {
                let needle: String = needle.split_whitespace().collect();
                assert!(
                    squashed.contains(&needle),
                    "{fixture} {to}: missing {needle}"
                );
            }
        }
    }
}
