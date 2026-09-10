use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use quire_dist::{
    package_npm, set_version, verify_binary_version, verify_release, PackageNpm, SetVersion,
    VerifyRelease,
};

#[derive(Debug, Parser)]
#[command(about = "Repository-local quire-cli distribution tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Update the Cargo and npm launcher versions in lockstep.
    SetVersion {
        #[arg(long)]
        version: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Verify committed Cargo and npm launcher versions.
    VerifyRelease {
        #[arg(long)]
        version: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Verify one executable's `--version` output.
    VerifyBinary {
        #[arg(long)]
        version: String,
        #[arg(long)]
        binary: PathBuf,
    },
    /// Generate the four platform npm packages from native artifacts.
    PackageNpm {
        #[arg(long)]
        version: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long, default_value = "artifacts")]
        artifacts_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::SetVersion { version, root } => set_version(&SetVersion {
            version: &version,
            cargo_manifest: &root.join("Cargo.toml"),
            launcher_manifest: &root.join("npm/quire-cli/package.json"),
        }),
        Command::VerifyRelease { version, root } => verify_release(&VerifyRelease {
            version: &version,
            cargo_manifest: &root.join("Cargo.toml"),
            launcher_manifest: &root.join("npm/quire-cli/package.json"),
        }),
        Command::VerifyBinary { version, binary } => verify_binary_version(&version, &binary),
        Command::PackageNpm {
            version,
            root,
            artifacts_dir,
        } => package_npm(&PackageNpm {
            version: &version,
            cargo_manifest: &root.join("Cargo.toml"),
            artifacts_dir: &artifacts_dir,
            output_dir: &root.join("npm/dist"),
            launcher_dir: &root.join("npm/quire-cli"),
            license: &root.join("LICENSE"),
        }),
    }
}
