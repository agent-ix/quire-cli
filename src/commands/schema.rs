//! `quire schema <ARCHETYPE> --module <PATH>`
//!
//! Surfaces `quire_rs::input_contract_for` (upstream FR-029, recast by
//! ADR 0004). Emits the archetype input contract — the frontmatter JSON
//! Schema plus the `body_extraction` **asserts** (required headings, table
//! columns, id-patterns) that `validate_document` (FR-032) enforces — as
//! deterministic JSON on stdout. There is no template-variable list:
//! templates were removed with the render retirement.
//!
//! Unknown archetypes exit 1 with `UnknownArchetype` on stderr; stdout is
//! empty. All contract derivation lives in quire-rs (StR-004 thin boundary).

use anyhow::{anyhow, Context};
use clap::Parser;

use quire_cli::io::{self, emit_quire_diagnostics};
use quire_cli::safety;
use quire_rs::Registry;

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    /// Archetype name registered in the loaded module.
    pub archetype: String,

    /// Path to the module directory (containing `manifest.yaml`).
    #[arg(long, value_name = "PATH")]
    pub module: String,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;
    let registry = Registry::load_module(&module).context("loading module registry")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());

    let contract = quire_rs::input_contract_for(&registry, &args.archetype).map_err(|e| {
        // Surface `UnknownArchetype` (and any other contract error) on the
        // error path; the leaf message carries the load-bearing identifier.
        anyhow!("{e}")
    })?;

    // `to_json()` produces deterministic, sorted-key JSON (FR-009-AC-4).
    let value = contract.to_json();
    let payload = io::encode_json(&value, ctx.pretty).context("encoding input contract as JSON")?;
    io::write_primary_stdout(payload.as_bytes()).context("writing schema output")?;
    io::write_primary_stdout(b"\n").ok();
    Ok(())
}
