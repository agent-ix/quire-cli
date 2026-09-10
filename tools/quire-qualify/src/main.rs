use std::path::PathBuf;

use clap::{Parser, Subcommand};
use quire_qualify::{benchmark, coverage, source, unsafe_comments, Error, Result};

#[derive(Debug, Parser)]
#[command(about = "Repository-local quire-cli qualification tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check the repository-owned assurance coverage population.
    AssuranceTraceability { report: PathBuf },
    /// Evaluate the first hyperfine result against a millisecond threshold.
    BenchP95 { report: PathBuf, threshold_ms: f64 },
    /// Enforce the Rust AST-backed CLI/engine boundary.
    ThinBoundary {
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Enforce or explicitly update unsafe-comment loci.
    UnsafeComments {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        update_baseline: bool,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::AssuranceTraceability { report } => {
            println!("{}", coverage::check_file(&report)?);
        }
        Command::BenchP95 {
            report,
            threshold_ms,
        } => {
            let observation = benchmark::evaluate_file(&report, threshold_ms)?;
            println!("{}", observation.summary());
            if !observation.passes() {
                return Err(Error::BenchmarkExceeded {
                    p95_ms: observation.p95_ms,
                    threshold_ms: observation.threshold_ms,
                });
            }
        }
        Command::ThinBoundary { root } => {
            source::check_thin_boundary(&root)?;
            println!("thin-boundary audit ok");
        }
        Command::UnsafeComments {
            root,
            update_baseline,
        } => {
            if update_baseline {
                let count = unsafe_comments::update_baseline(&root)?;
                println!("wrote unsafe-comment baseline with {count} entries");
            } else {
                unsafe_comments::check(&root)?;
                println!("unsafe-comment audit ok");
            }
        }
    }
    Ok(())
}
