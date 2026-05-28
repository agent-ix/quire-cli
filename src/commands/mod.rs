//! Subcommand implementations. Each module owns one of the four
//! `quire <verb>` subcommands and stays a thin wrapper over `quire-rs`.

pub mod extract;
pub mod parse;
pub mod render;
pub mod validate;

use quire_cli::io::DiagnosticsFormat;

/// Cross-command context plumbed in from global flags.
pub struct Ctx {
    pub diagnostics: DiagnosticsFormat,
    pub pretty: bool,
}
