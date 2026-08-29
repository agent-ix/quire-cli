//! `quire provenance --json` — evidence-grade identity of the executable.

use clap::Parser;

use crate::commands::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    /// Emit the versioned machine contract. Retained for explicitness even
    /// though this command has no human mode yet.
    #[arg(long, default_value_t = true)]
    pub json: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    if !args.json {
        anyhow::bail!("provenance is a machine contract; use --json")
    }
    println!(
        "{}",
        quire_cli::io::encode_json(&quire_cli::engine::ToolProvenance::current(), ctx.pretty)?
    );
    Ok(())
}
