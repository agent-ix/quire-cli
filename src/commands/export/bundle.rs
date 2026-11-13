//! `quire export <INPUT> <INPUT>... --bundle-title <TEXT> --bundle-id <ID>`
//! (FR-039): several committed Markdown documents as one output file, in the
//! order given. The bundle title page, document list and per-document
//! separators are generated; the documents themselves are passed unchanged.

use std::path::PathBuf;

use super::identity::{self, Identity, Source, SourceOptions};
use super::template::{self, escape_markdown, raw_block};
use super::{approval, custom};
use super::{
    check_flags, check_output_is_not_input, check_output_target, input_class, mermaid_blocks,
    producer, read_utf8, validate, write_output, Args, Ctx, Format, InputClass,
};

/// The most inputs one bundle holds.
const MAX_INPUTS: usize = 256;

pub(super) fn run(ctx: &Ctx, args: &Args) -> anyhow::Result<()> {
    let (title, id) = check_request(args)?;
    check_output_target(&args.output)?;
    let organisation_template = args
        .template
        .as_deref()
        .map(|path| custom::load(path, args.to))
        .transpose()?;
    let approval = approval::load(&args.approval)?;
    let sources = resolve_sources(ctx, args)?;
    let first = &sources[0];
    let identity = Identity {
        title: title.to_string(),
        document_id: id.to_string(),
        version: None,
        release: args.release.clone(),
        commit: first.identity.commit.clone(),
        date: first.identity.date.clone(),
        epoch: first.identity.epoch,
    };
    let documents: Vec<&Identity> = sources.iter().map(|source| &source.identity).collect();
    let diagrams: Vec<_> = args
        .input
        .iter()
        .zip(&sources)
        .flat_map(|(input, source)| mermaid_blocks(input, &source.committed))
        .collect();
    let converters = producer::Converters::resolve(args, !diagrams.is_empty())?;
    let generated = producer::Generated {
        body: "",
        bundle: sources
            .iter()
            .map(|source| producer::BundleDocument {
                separator: separator(&source.identity, args.to),
                body: &source.body,
            })
            .collect(),
        approval: approval.as_deref(),
        metadata: &template::metadata_yaml(&identity)?,
        diagrams,
        identity_parts: Some(producer::IdentityParts {
            front: &template::bundle_front_matter(&identity, &documents, args.to),
            template: match (organisation_template, args.to) {
                (Some(bytes), _) => bytes,
                (None, Format::Pdf) => template::typst_template(&identity).into_bytes(),
                (None, Format::Docx) => template::docx_reference(&identity)?,
            },
        }),
        epoch: identity.epoch,
    };
    let rendered = producer::convert(&converters, args.to, &generated)?;
    write_output(&args.output, rendered.as_ref())
}

/// The request-level refusals of FR-039 and FR-040, all before any file of the
/// bundle is read.
fn check_request(args: &Args) -> anyhow::Result<(&str, &str)> {
    if args.input.len() < 2 {
        invalid_request!("--bundle-title and --bundle-id need more than one input");
    }
    if args.input.len() > MAX_INPUTS {
        invalid_request!(
            "a bundle holds at most {MAX_INPUTS} inputs; {} were given",
            args.input.len()
        );
    }
    fn present(value: &Option<String>) -> Option<&str> {
        value.as_deref().filter(|text| !text.trim().is_empty())
    }
    let (Some(title), Some(id)) = (present(&args.bundle_title), present(&args.bundle_id)) else {
        let mut missing = Vec::new();
        if present(&args.bundle_title).is_none() {
            missing.push("--bundle-title");
        }
        if present(&args.bundle_id).is_none() {
            missing.push("--bundle-id");
        }
        invalid_request!(
            "a bundle of {} inputs lacks a non-blank {}",
            args.input.len(),
            missing.join(" and ")
        );
    };
    for input in &args.input {
        if input_class(input)? != InputClass::Markdown {
            invalid_request!("'{}': a bundle takes only Markdown inputs", input.display());
        }
        check_output_is_not_input(input, &args.output)?;
    }
    check_flags(args, InputClass::Markdown)?;
    Ok((title, id))
}

/// Reads, identifies and validates every input, in order, from the one
/// repository. The first failure names its file.
fn resolve_sources(ctx: &Ctx, args: &Args) -> anyhow::Result<Vec<Source>> {
    let options = SourceOptions {
        repo: args.source_repo.as_deref(),
        rev: args.source_rev.as_deref(),
        release: args.release.as_deref(),
    };
    let mut sources: Vec<Source> = Vec::with_capacity(args.input.len());
    let mut first: Option<(&PathBuf, PathBuf)> = None;
    for input in &args.input {
        let text = read_utf8(input)?;
        let source = identity::resolve(input, &text, &options)?;
        match &first {
            None => first = Some((input, source.repository.clone())),
            Some((first_input, repository)) if *repository != source.repository => {
                invalid_request!(
                    "a bundle takes inputs from one repository: '{}' is in '{}' but '{}' is in '{}'",
                    first_input.display(),
                    repository.display(),
                    input.display(),
                    source.repository.display()
                );
            }
            Some(_) => {}
        }
        if let Some(earlier) = sources
            .first()
            .filter(|earlier| earlier.identity.commit != source.identity.commit)
        {
            invalid_request!(
                "a bundle takes inputs from one source commit: '{}' is at {} but '{}' is at {}",
                args.input[0].display(),
                earlier.identity.commit,
                input.display(),
                source.identity.commit
            );
        }
        sources.push(source);
    }
    for (input, source) in args.input.iter().zip(&sources) {
        validate::run_with(
            ctx,
            validate::Args {
                documents: vec![input.display().to_string()],
                module: args.module.clone(),
                scope: ".".to_string(),
                archetype: None,
                okf: false,
                strict: false,
                summary: false,
                severity: Vec::new(),
                repo: Vec::new(),
            },
            validate::Source::Committed(&source.committed),
        )?;
    }
    Ok(sources)
}

/// `in/separator-<n>.md`: a page break and the level-1 heading `<id>: <title>`.
fn separator(document: &Identity, to: Format) -> String {
    format!(
        "{}# {}: {}\n\n",
        raw_block(to, template::PAGE_BREAK_TYPST, template::PAGE_BREAK_OPENXML),
        escape_markdown(&document.document_id),
        escape_markdown(&document.title)
    )
}
