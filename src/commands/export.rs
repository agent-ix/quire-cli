//! `quire export <INPUT> --to pdf|docx --output <PATH>` (FR-031..FR-033).
//!
//! Renders a Markdown artifact as PDF or DOCX. The conversion is the work of
//! external converters launched by `engineering-assurance producer-execute`
//! (NFR-010); this module validates the input, assembles the converter input
//! under a temporary capability root, and copies the result to `--output`.
//! Markdown parsing and validation stay in `quire-rs` (StR-004).

mod bundle;
mod custom;
mod identity;
mod json;
mod producer;
mod template;
mod tool;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, ValueEnum};
use ix_cli_kit::exit::Outcome;

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
    /// A Markdown (`.md`) or JSON (`.json`) file to export. Several Markdown
    /// files are exported as one bundle, in the order given.
    #[arg(value_name = "INPUT", required = true)]
    pub input: Vec<PathBuf>,

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

    /// Git repository holding the input; default is the repository that
    /// contains the input file. Refused for a JSON input.
    #[arg(long, value_name = "DIR")]
    pub source_repo: Option<PathBuf>,

    /// Revision of the source repository to export; default `HEAD`. Refused
    /// for a JSON input.
    #[arg(long, value_name = "REV")]
    pub source_rev: Option<String>,

    /// Tag of the source repository that names the source commit, shown as
    /// the release of the document. Refused for a JSON input.
    #[arg(long, value_name = "TAG")]
    pub release: Option<String>,

    /// Organisation template, used in place of the default: a Typst `.typ`
    /// file for `pdf`, a Word `.docx` reference document for `docx`. Refused
    /// for a JSON input.
    #[arg(long, value_name = "PATH")]
    pub template: Option<PathBuf>,

    /// Title of a bundle of several inputs; required for a bundle, refused for
    /// one input.
    #[arg(long, value_name = "TEXT")]
    pub bundle_title: Option<String>,

    /// Id of a bundle of several inputs; required for a bundle, refused for one
    /// input.
    #[arg(long, value_name = "ID")]
    pub bundle_id: Option<String>,

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

impl Args {
    fn has_bundle_option(&self) -> bool {
        self.bundle_title.is_some() || self.bundle_id.is_some()
    }
}

/// The input class an extension selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputClass {
    Markdown,
    Json,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let input = &args.input[0];
    let class = input_class(input)?;
    if args.input.len() > 1 || (class == InputClass::Markdown && args.has_bundle_option()) {
        return bundle::run(ctx, &args);
    }
    check_flags(&args, class)?;
    check_output_is_not_input(input, &args.output)?;
    let text = read_utf8(input)?;
    check_output_target(&args.output)?;
    match class {
        InputClass::Markdown => export_markdown(ctx, &args, &text),
        InputClass::Json => match args.kind {
            Some(kind) => json::export(&args, kind, &text),
            None => invalid_request!("a JSON input requires --kind"),
        },
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
        InputClass::Json if args.source_repo.is_some() => {
            invalid_request!("--source-repo is refused for a JSON input");
        }
        InputClass::Json if args.source_rev.is_some() => {
            invalid_request!("--source-rev is refused for a JSON input");
        }
        InputClass::Json if args.bundle_title.is_some() => {
            invalid_request!("--bundle-title is refused for a JSON input");
        }
        InputClass::Json if args.bundle_id.is_some() => {
            invalid_request!("--bundle-id is refused for a JSON input");
        }
        InputClass::Json if args.release.is_some() => {
            invalid_request!("--release is refused for a JSON input");
        }
        InputClass::Json if args.template.is_some() => {
            invalid_request!("--template is refused for a JSON input");
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

/// `--output` must be a writable file in an existing directory; checked before
/// any conversion and again at the copy. Writability is probed by creating (and
/// dropping) a sibling temporary file, the same operation the copy performs.
fn check_output_target(output: &Path) -> anyhow::Result<()> {
    if output.is_dir() {
        return Err(output_error(format!(
            "--output '{}' is a directory",
            output.display()
        )));
    }
    let parent = parent_dir(output);
    if !parent.is_dir() {
        return Err(output_error(format!(
            "--output directory '{}' does not exist",
            parent.display()
        )));
    }
    tempfile::NamedTempFile::new_in(parent).map_err(|error| {
        output_error(format!(
            "--output directory '{}' is not writable: {error}",
            parent.display()
        ))
    })?;
    Ok(())
}

/// An unusable `--output` is an output failure (exit 4).
fn output_error(message: String) -> anyhow::Error {
    failure::classified(Outcome::Internal, anyhow::anyhow!(message))
}

fn export_markdown(ctx: &Ctx, args: &Args, text: &str) -> anyhow::Result<()> {
    let organisation_template = args
        .template
        .as_deref()
        .map(|path| custom::load(path, args.to))
        .transpose()?;
    let source = identity::resolve(
        &args.input[0],
        text,
        &identity::SourceOptions {
            repo: args.source_repo.as_deref(),
            rev: args.source_rev.as_deref(),
            release: args.release.as_deref(),
        },
    )?;
    // The committed text is what is validated, under the input's own path.
    validate::run_with(
        ctx,
        validate::Args {
            documents: vec![args.input[0].display().to_string()],
            module: args.module.clone(),
            scope: ".".to_string(),
            archetype: None,
            okf: false,
            strict: false,
            summary: false,
            severity: Vec::new(),
        },
        validate::Source::Committed(&source.committed),
    )?;

    let converters = producer::Converters::resolve(args)?;
    let identity = &source.identity;
    let generated = producer::Generated {
        body: &source.body,
        bundle: Vec::new(),
        metadata: &template::metadata_yaml(identity)?,
        identity_parts: Some(producer::IdentityParts {
            front: &template::front_matter(identity, args.to),
            template: match (organisation_template, args.to) {
                (Some(bytes), _) => bytes,
                (None, Format::Pdf) => template::typst_template(identity).into_bytes(),
                (None, Format::Docx) => template::docx_reference(identity)?,
            },
        }),
        epoch: identity.epoch,
    };
    let rendered = producer::convert(&converters, args.to, &generated)?;
    write_output(&args.output, rendered.as_ref())
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
