//! `quire lint <DOC.md|-> --module <PATH> [--archetype <NAME>]`
//!
//! Advisory lint: evaluates the module's declarative `lint_rules`
//! (upstream FR-036) against an authored document. Lint is a posture
//! distinct from `validate` — findings flag authoring-convention drift
//! (vocabulary discipline), never structural invalidity, and lint never
//! blocks extraction or sync.
//!
//! Findings are emitted on stderr (`LintWarning` / `LintError` per the
//! owning rule's severity). Exit 0 when there are no findings or only
//! warnings; exit 1 when any `error`-severity finding fires. NEVER
//! writes stdout. All rule evaluation lives in quire-rs (StR-004 thin
//! boundary).

use anyhow::{bail, Context};
use clap::Parser;

use quire_cli::io;
use quire_cli::safety;
use quire_rs::LintSeverity;

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Markdown document path (or `-` for stdin).
    pub document: String,

    /// Path to the module directory (containing `manifest.yaml`).
    #[arg(long, value_name = "PATH")]
    pub module: String,

    /// Override the archetype resolved from the document frontmatter
    /// `artifact_type` (used only for rule scoping).
    #[arg(long, value_name = "NAME")]
    pub archetype: Option<String>,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;
    let registry = super::load_module_registry(ctx, &module)?;

    if args.document != "-" {
        safety::validate_input_path("document", &args.document)
            .with_context(|| format!("validating document '{}'", args.document))?;
    }
    let text =
        io::read_text(&args.document).with_context(|| format!("reading '{}'", args.document))?;
    let doc = quire_rs::parse_document(&text);

    // Tolerant archetype resolution (FR-036-AC-3): scoping only —
    // an unresolvable archetype runs unfiltered rules, never errors.
    let archetype = args.archetype.clone().or_else(|| {
        doc.frontmatter.as_ref().and_then(|fm| {
            fm.get("artifact_type")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
    });

    let findings = quire_rs::lint_document(registry.lint_rules(), archetype.as_deref(), &doc);

    let mut errors = 0usize;
    for finding in &findings {
        let kind = match finding.severity {
            LintSeverity::Warning => "LintWarning",
            LintSeverity::Error => {
                errors += 1;
                "LintError"
            }
        };
        // Severity rides in the message so the human format (which
        // drops `kind`) still shows it; JSON carries both.
        io::emit_diagnostic(
            ctx.diagnostics,
            kind,
            &format!(
                "{}: {}: {}",
                finding.severity.as_str(),
                finding.rule,
                finding.message
            ),
        );
    }

    if errors > 0 {
        bail!("{errors} error-severity lint finding(s)");
    }
    Ok(())
}
