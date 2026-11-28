//! `quire provenance --json` — evidence-grade identity of the executable.

use ix_trace_rs::implements;
use clap::Parser;

use crate::commands::Ctx;

#[derive(Debug, Parser)]
pub struct Args {
    /// Emit the versioned machine contract. Retained for explicitness even
    /// though this command has no human mode yet.
    #[arg(long, default_value_t = true)]
    pub json: bool,
}

#[implements("FR-008-AC-5")]
#[implements("FR-008-AC-6")]
pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    if !args.json {
        invalid_request!("provenance is a machine contract; use --json")
    }
    primary_line!(
        "{}",
        ix_cli_kit::json::encode(&quire_cli::engine::ToolProvenance::current(), ctx.pretty)?
    );
    Ok(())
}
