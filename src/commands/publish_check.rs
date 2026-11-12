//! `quire publish-check` — refuse a module that carries restricted clause text.
//!
//! The engine decides which clause sets are restricted. This command loads one
//! module, asks the engine, and maps the verdict to an exit code: no
//! restricted set exits 0, any restricted set exits 2 naming each one.

use anyhow::Context;
use clap::{Parser, ValueEnum};
use quire_rs::{check_publishable, RestrictedClauseSet};
use serde_json::json;

use crate::commands::Ctx;
use quire_cli::{engine, io, safety};

#[derive(Debug, Parser)]
pub struct Args {
    /// Module directory to check.
    #[arg(long)]
    module: String,
    /// Output form. Human is the default.
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let restricted = load_verdict(ctx, &args.module)?;
    match args.format {
        OutputFormat::Human => {
            if restricted.is_empty() {
                primary_line!("no restricted clause sets");
            }
        }
        OutputFormat::Json => {
            let body = ix_cli_kit::json::encode(
                &engine::attach(json!({ "restricted": restricted_json(&restricted) })),
                ctx.pretty,
            )?;
            primary!("{body}\n");
        }
    }
    verdict_to_result(&restricted)
}

fn load_verdict(ctx: &Ctx, raw: &str) -> anyhow::Result<Vec<RestrictedClauseSet>> {
    let module = safety::validate_module_path(raw)
        .with_context(|| format!("validating --module {raw:?}"))?;
    let outcome = quire_rs::loader::load_single_module(&module);
    io::emit_quire_diagnostics(ctx.diagnostics, &outcome.diagnostics);
    if let Some(failure) = outcome.manifest_errors.first() {
        invalid_request!(
            "module load failed: {} ({})",
            failure.message,
            failure.path.display()
        );
    }
    if let Some(failure) = outcome.failures.first() {
        invalid_request!(
            "module load failed: {} ({})",
            failure.reason,
            failure.path.display()
        );
    }
    let [loaded] = outcome.modules.as_slice() else {
        invalid_request!("module load failed: no module at {}", module.display());
    };
    Ok(check_publishable(loaded))
}

/// Exit 0 for an empty verdict; otherwise a refusal naming every set.
fn verdict_to_result(restricted: &[RestrictedClauseSet]) -> anyhow::Result<()> {
    if restricted.is_empty() {
        return Ok(());
    }
    let names = restricted
        .iter()
        .map(|set| format!("{}/{}@{}", set.authority, set.id, set.version))
        .collect::<Vec<_>>()
        .join("\n");
    policy_refusal!("restricted clause sets:\n{names}");
}

fn restricted_json(restricted: &[RestrictedClauseSet]) -> Vec<serde_json::Value> {
    restricted
        .iter()
        .map(|set| {
            json!({
                "authority": set.authority,
                "id": set.id,
                "version": set.version,
                "text_clauses": set.text_clauses,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_cli_kit::exit::Outcome;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-029-AC-7")]
    fn exit_code_follows_the_engine_verdict_only() {
        assert!(verdict_to_result(&[]).is_ok());
        let set = RestrictedClauseSet {
            authority: "a.invalid".into(),
            id: "s".into(),
            version: "1".into(),
            text_clauses: 0,
        };
        let error = verdict_to_result(&[set]).unwrap_err();
        assert_eq!(super::super::failure::outcome(&error), Outcome::Refused);
        assert!(format!("{error:#}").contains("a.invalid/s@1"));
    }
}
