//! `quire export <INPUT> --to pdf|docx --output <PATH>` (FR-031..FR-033).
//!
//! Renders a Markdown artifact as PDF or DOCX. The conversion is the work of
//! external converters launched by `engineering-assurance producer-execute`
//! (NFR-010); this module validates the input, assembles the converter input
//! under a temporary capability root, and copies the result to `--output`.
//! Markdown parsing and validation stay in `quire-rs` (StR-004).

mod producer;
mod tool;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, ValueEnum};
use serde_json::{Map, Value};

use super::{failure, validate, Ctx};

/// The output format selected by `--to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// PDF, typeset by Typst from pandoc's Typst output.
    Pdf,
    /// DOCX, written by pandoc.
    Docx,
}

/// The table rendering selected by `--kind` for a JSON input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Kind {
    /// `quire matrix --format json` output.
    Matrix,
    /// `quoin discharge --rmm --json` output.
    Rmm,
    /// `quoin soup report --json` output.
    Soup,
    /// Any object with array-of-object members.
    Table,
}

#[derive(Parser, Debug)]
pub struct Args {
    /// A Markdown (`.md`) or JSON (`.json`) file to export.
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Output format.
    #[arg(long, value_enum)]
    pub to: Format,

    /// Output file; written only on success.
    #[arg(long, value_name = "PATH")]
    pub output: PathBuf,

    /// Table rendering for a JSON input; refused for a Markdown input.
    #[arg(long, value_enum)]
    pub kind: Option<Kind>,

    /// Module directory used to validate a Markdown input, resolved as
    /// `quire validate --module` is. Repeatable; refused for a JSON input.
    #[arg(long, value_name = "PATH")]
    pub module: Vec<String>,

    /// Path to `pandoc`, overriding the PATH lookup.
    #[arg(long, value_name = "PATH")]
    pub pandoc: Option<PathBuf>,

    /// Path to `typst`, overriding the PATH lookup.
    #[arg(long, value_name = "PATH")]
    pub typst: Option<PathBuf>,

    /// Path to `engineering-assurance`, overriding the PATH lookup.
    #[arg(long, value_name = "PATH")]
    pub engineering_assurance: Option<PathBuf>,
}

/// The input class an extension selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputClass {
    Markdown,
    Json,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let class = input_class(&args.input)?;
    check_flags(&args, class)?;
    check_output_is_not_input(&args.input, &args.output)?;
    let text = read_utf8(&args.input)?;
    check_output_target(&args.output)?;
    match class {
        InputClass::Markdown => export_markdown(ctx, &args, &text),
        InputClass::Json => invalid_request!(
            "'{}': JSON report export is not available in this build",
            args.input.display()
        ),
    }
}

fn input_class(input: &Path) -> anyhow::Result<InputClass> {
    match input.extension().and_then(|ext| ext.to_str()) {
        Some("md") => Ok(InputClass::Markdown),
        Some("json") => Ok(InputClass::Json),
        _ => invalid_request!(
            "'{}': the input must be a .md or .json file ('-' is not accepted)",
            input.display()
        ),
    }
}

/// The flag combinations FR-031 refuses.
fn check_flags(args: &Args, class: InputClass) -> anyhow::Result<()> {
    match class {
        InputClass::Markdown if args.kind.is_some() => {
            invalid_request!("--kind is refused for a Markdown input");
        }
        InputClass::Json if args.kind.is_none() => {
            invalid_request!("a JSON input requires --kind");
        }
        InputClass::Json if !args.module.is_empty() => {
            invalid_request!("--module is refused for a JSON input");
        }
        _ => Ok(()),
    }
}

/// `--output` must not name the input file; checked before any conversion.
fn check_output_is_not_input(input: &Path, output: &Path) -> anyhow::Result<()> {
    let Ok(input) = fs::canonicalize(input) else {
        return Ok(());
    };
    let resolved = match fs::canonicalize(output) {
        Ok(path) => Some(path),
        Err(_) => output
            .file_name()
            .zip(parent_dir(output).canonicalize().ok())
            .map(|(name, dir)| dir.join(name)),
    };
    if resolved.is_some_and(|path| path == input) {
        policy_refusal!(
            "--output '{}' names the input file; refusing to overwrite it",
            output.display()
        );
    }
    Ok(())
}

fn parent_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// The input as UTF-8 text. A permission refusal is exit 2, a missing file or
/// non-UTF-8 content is exit 3.
fn read_utf8(input: &Path) -> anyhow::Result<String> {
    let bytes = fs::read(input)
        .map_err(anyhow::Error::from)
        .map_err(failure::input)
        .with_context(|| format!("reading '{}'", input.display()))?;
    String::from_utf8(bytes).map_err(|_| invalid_error!("'{}' is not valid UTF-8", input.display()))
}

/// `--output` must be a file in an existing directory; checked before any
/// conversion and again at the copy.
fn check_output_target(output: &Path) -> anyhow::Result<()> {
    if output.is_dir() {
        anyhow::bail!("--output '{}' is a directory", output.display());
    }
    if !parent_dir(output).is_dir() {
        anyhow::bail!(
            "--output directory '{}' does not exist",
            parent_dir(output).display()
        );
    }
    Ok(())
}

fn export_markdown(ctx: &Ctx, args: &Args, text: &str) -> anyhow::Result<()> {
    validate::run(
        ctx,
        validate::Args {
            documents: vec![args.input.display().to_string()],
            module: args.module.clone(),
            scope: ".".to_string(),
            archetype: None,
            okf: false,
            strict: false,
            summary: false,
            severity: Vec::new(),
        },
    )?;

    let parsed = quire_rs::extract_frontmatter(text);
    let metadata = metadata_yaml(parsed.frontmatter.as_ref())?;
    let converters = producer::Converters::resolve(args)?;
    let rendered = producer::convert(&converters, args.to, &parsed.body, &metadata)?;
    write_output(&args.output, rendered.as_ref())
}

/// The pandoc metadata file. The frontmatter `title` is the only member passed
/// to the converter; the rest of the frontmatter is not converter input. JSON
/// is a YAML subset, so the file is a single JSON object.
fn metadata_yaml(frontmatter: Option<&Map<String, Value>>) -> anyhow::Result<String> {
    let mut metadata = Map::new();
    if let Some(title) = frontmatter
        .and_then(|map| map.get("title"))
        .and_then(Value::as_str)
    {
        metadata.insert("title".to_string(), Value::String(title.to_string()));
    }
    let mut yaml = serde_json::to_string(&metadata).context("encoding converter metadata")?;
    yaml.push('\n');
    Ok(yaml)
}

/// Copies the converter output to `--output` through a sibling temporary file,
/// so a failure leaves a pre-existing `--output` unchanged.
fn write_output(output: &Path, rendered: &Path) -> anyhow::Result<()> {
    check_output_target(output)?;
    let staged = tempfile::NamedTempFile::new_in(parent_dir(output))
        .with_context(|| format!("writing --output '{}'", output.display()))?;
    fs::copy(rendered, staged.path())
        .with_context(|| format!("writing --output '{}'", output.display()))?;
    staged.persist(output).map_err(|error| {
        let kind = error.error.kind();
        anyhow::Error::from(error.error)
            .context(format!("writing --output '{}' ({kind})", output.display()))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-031-AC-7")]
    fn metadata_carries_only_the_title() {
        let mut map = Map::new();
        map.insert("title".into(), Value::String("A \"quoted\" title".into()));
        map.insert("type".into(), Value::String("FR".into()));
        let yaml = metadata_yaml(Some(&map)).expect("metadata");
        assert_eq!(yaml, "{\"title\":\"A \\\"quoted\\\" title\"}\n");
        assert_eq!(metadata_yaml(None).expect("metadata"), "{}\n");
    }

    #[test]
    #[trace("FR-033-AC-6")]
    fn extension_selects_the_input_class() {
        assert_eq!(
            input_class(Path::new("a.md")).ok(),
            Some(InputClass::Markdown)
        );
        assert_eq!(
            input_class(Path::new("a.json")).ok(),
            Some(InputClass::Json)
        );
        assert!(input_class(Path::new("a.txt")).is_err());
        assert!(input_class(Path::new("-")).is_err());
    }

    #[test]
    #[trace("FR-033-AC-9")]
    fn a_missing_output_directory_is_refused() {
        let error = check_output_target(Path::new("/nonexistent-quire-dir/out.pdf"))
            .expect_err("missing directory");
        assert!(format!("{error:#}").contains("does not exist"));
    }
}
