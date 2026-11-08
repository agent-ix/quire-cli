use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use quire_dist::verify_binary_version;

#[derive(Debug, Parser)]
#[command(about = "Repository-local quire-cli release tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Verify one executable's `--version` output.
    VerifyBinary {
        #[arg(long)]
        version: String,
        #[arg(long)]
        binary: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::VerifyBinary { version, binary } => verify_binary_version(&version, &binary),
    }
}
