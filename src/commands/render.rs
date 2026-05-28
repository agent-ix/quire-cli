//! `quire render <archetype> --module <path> --data <file|-> [--out <path>]`
//!
//! Surfaces `quire_rs::render_by_name` (FR-001 / upstream FR-001+002+014).

use std::path::{Path, PathBuf};

use anyhow::Context;
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

    /// JSON data file (or `-` for stdin).
    #[arg(long, value_name = "FILE")]
    pub data: String,

    /// Optional output file; default stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<String>,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;
    if args.data != "-" {
        safety::validate_data_path(&args.data)
            .with_context(|| format!("validating --data '{}'", args.data))?;
    }
    let out_path: Option<PathBuf> = match args.out.as_deref() {
        Some(o) => {
            Some(safety::validate_out_path(o).with_context(|| format!("validating --out '{o}'"))?)
        }
        None => None,
    };

    let data = io::read_data(&args.data).with_context(|| "reading --data")?;
    let search_root = safety::search_root_for_module(&module);
    let module_ref: &Path = search_root.as_path();
    let registry = Registry::load_from(&[module_ref]).context("loading module registry")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());

    let rendered =
        quire_rs::render_by_name(&registry, &args.archetype, &data).with_context(|| {
            format!(
                "rendering archetype '{}' from module '{}'",
                args.archetype,
                module.display()
            )
        })?;
    emit_quire_diagnostics(ctx.diagnostics, rendered.diagnostics.iter());

    io::write_primary(out_path.as_deref(), rendered.markdown.as_bytes())
        .context("writing render output")?;
    Ok(())
}
