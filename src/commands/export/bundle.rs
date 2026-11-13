//! `quire export <INPUT> <INPUT>... --bundle-title <TEXT> --bundle-id <ID>`
//! (FR-039): several committed Markdown documents as one output file, in the
//! order given. The bundle title page, document list and per-document
//! separators are generated; the documents themselves are passed unchanged.

use std::path::PathBuf;

use super::identity::{self, Identity, Source, SourceOptions};
use super::template::{self, escape_markdown, raw_block};
use super::{
    check_flags, check_output_is_not_input, check_output_target, input_class, producer, read_utf8,
    validate, write_output, Args, Ctx, Format, InputClass,
};

/// The most inputs one bundle holds.
const MAX_INPUTS: usize = 256;

pub(super) fn run(ctx: &Ctx, args: &Args) -> anyhow::Result<()> {
    check_request(args)?;
    check_output_target(&args.output)?;
    let sources = resolve_sources(ctx, args)?;
    let (Some(title), Some(id), Some(first)) = (
        args.bundle_title.as_deref(),
        args.bundle_id.as_deref(),
        sources.first(),
    ) else {
        unreachable!("check_request admits only a bundle with a title, an id and inputs");
    };
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
    let converters = producer::Converters::resolve(args)?;
    let generated = producer::Generated {
        body: "",
        bundle: sources
            .iter()
            .map(|source| producer::BundleDocument {
                separator: separator(&source.identity, args.to),
                body: &source.body,
            })
            .collect(),
        metadata: &template::metadata_yaml(&identity)?,
        identity_parts: Some(producer::IdentityParts {
            front: &front_matter(&identity, &documents, args.to),
            template: match args.to {
                Format::Pdf => template::typst_template(&identity).into_bytes(),
                Format::Docx => template::docx_reference(&identity)?,
            },
        }),
        epoch: Some(identity.epoch),
    };
    let rendered = producer::convert(&converters, args.to, &generated)?;
    write_output(&args.output, rendered.as_ref())
}

/// The request-level refusals of FR-039 and FR-040, all before any file of the
/// bundle is read.
fn check_request(args: &Args) -> anyhow::Result<()> {
    if args.input.len() < 2 {
        invalid_request!("--bundle-title and --bundle-id need more than one input");
    }
    if args.input.len() > MAX_INPUTS {
        invalid_request!(
            "a bundle holds at most {MAX_INPUTS} inputs; {} were given",
            args.input.len()
        );
    }
    let mut missing = Vec::new();
    if args.bundle_title.is_none() {
        missing.push("--bundle-title");
    }
    if args.bundle_id.is_none() {
        missing.push("--bundle-id");
    }
    if !missing.is_empty() {
        invalid_request!(
            "a bundle of {} inputs lacks {}",
            args.input.len(),
            missing.join(" and ")
        );
    }
    for input in &args.input {
        if input_class(input)? != InputClass::Markdown {
            invalid_request!("'{}': a bundle takes only Markdown inputs", input.display());
        }
        check_output_is_not_input(input, &args.output)?;
    }
    check_flags(args, InputClass::Markdown)
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

/// `in/front.md` of a bundle: the title page, a page break, the document list,
/// a page break and the table of contents.
fn front_matter(bundle: &Identity, documents: &[&Identity], to: Format) -> String {
    let page_break = || raw_block(to, template::PAGE_BREAK_TYPST, template::PAGE_BREAK_OPENXML);
    let mut front = String::new();
    match to {
        Format::Docx => front.push_str(&format!(
            "::: {{custom-style=\"Title\"}}\n{}\n:::\n\n",
            escape_markdown(&bundle.title)
        )),
        Format::Pdf => front.push_str(&format!("{}\n\n", escape_markdown(&bundle.title))),
    }
    let mut line = |label: &str, value: &str| {
        front.push_str(&format!("{label}: {}\n\n", escape_markdown(value)));
    };
    line("Bundle id", &bundle.document_id);
    if let Some(release) = &bundle.release {
        line("Release", release);
    }
    line("Source commit", &bundle.commit);
    line("Date", &bundle.date);
    front.push_str(&page_break());
    match to {
        Format::Docx => front.push_str("::: {custom-style=\"TOC Heading\"}\nDocuments\n:::\n\n"),
        Format::Pdf => front.push_str("Documents\n\n"),
    }
    front.push_str("| Id | Title | Version |\n| --- | --- | --- |\n");
    for document in documents {
        let revision = document
            .version
            .as_deref()
            .or(document.release.as_deref())
            .unwrap_or_default();
        front.push_str(&format!(
            "| {} | {} | {} |\n",
            escape_markdown(&document.document_id),
            escape_markdown(&document.title),
            escape_markdown(revision)
        ));
    }
    front.push('\n');
    front.push_str(&page_break());
    if to == Format::Docx {
        front.push_str("::: {custom-style=\"TOC Heading\"}\nContents\n:::\n\n");
    }
    front.push_str(&raw_block(to, template::TOC_TYPST, template::TOC_OPENXML));
    front.push_str(&page_break());
    front
}
