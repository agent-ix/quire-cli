//! `quire update [--check] [--registry <URL>]`
//!
//! Thin, quire-specific wrapper over the package-agnostic
//! [`quire_cli::self_update`] engine. This file is the ONLY place quire's
//! install coordinates live; the engine itself is reusable and is the unit
//! intended to move into a shared Rust CLI kit later.

use clap::Parser;

use quire_cli::io;
use quire_cli::self_update::{self, SelfUpdateConfig, SelfUpdateOpts};

use super::Ctx;

/// quire's distribution coordinates. Quire ships on public npm
/// (`@agent-ix/quire-cli`, prebuilt-binary wrapper) and via `cargo install`
/// from its GitHub repo.
const CONFIG: SelfUpdateConfig = SelfUpdateConfig {
    npm_package: "@agent-ix/quire-cli",
    cargo_git: "https://github.com/agent-ix/quire-cli",
    releases_url: "https://github.com/agent-ix/quire-cli/releases",
};

#[derive(Parser, Debug)]
pub struct Args {
    /// Report whether an update is available without installing.
    #[arg(long)]
    pub check: bool,

    /// Force an npm registry to query/install from (npm channel only).
    /// Defaults to the ambient npm config — i.e. however quire was installed.
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,
}

pub fn run(_ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let report = self_update::run_self_update(
        &CONFIG,
        &SelfUpdateOpts {
            check: args.check,
            registry: args.registry,
        },
    )?;

    // The engine already let npm/cargo draw their own progress to the inherited
    // streams; here we print the summary lines as the command's primary output.
    for line in &report.messages {
        io::write_primary_stdout(format!("{line}\n").as_bytes())?;
    }
    Ok(())
}
