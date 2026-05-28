//! `quire validate <archetype> --module <path> --data <file|->`
//!
//! Surfaces `quire_rs::validate` (upstream FR-002 / FR-017).
//! Exit 0 on success, exit 1 on validation failure. NEVER writes stdout.

use std::path::Path;

use anyhow::{anyhow, Context};
use clap::Parser;

use quire_cli::io::{self, emit_quire_diagnostics};
use quire_cli::safety;
use quire_rs::Registry;

use super::Ctx;

#[derive(Parser, Debug)]
pub struct Args {
    pub archetype: String,

    #[arg(long, value_name = "PATH")]
    pub module: String,

    #[arg(long, value_name = "FILE")]
    pub data: String,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;
    if args.data != "-" {
        safety::validate_data_path(&args.data)
            .with_context(|| format!("validating --data '{}'", args.data))?;
    }
    let data = io::read_data(&args.data).context("reading --data")?;

    let search_root = safety::search_root_for_module(&module);
    let module_ref: &Path = search_root.as_path();
    let registry = Registry::load_from(&[module_ref]).context("loading module registry")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());

    let archetype = registry
        .archetype(&args.archetype)
        .ok_or_else(|| anyhow!("unknown archetype '{}'", args.archetype))?;

    quire_rs::validate(archetype, &data)
        .with_context(|| format!("validating against archetype '{}'", args.archetype))?;
    Ok(())
}
