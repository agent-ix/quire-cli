//! Subcommand implementations. Each module owns one `quire <verb>` subcommand.
//! The quire-rs-backed verbs stay thin wrappers over the engine; `update` is a
//! thin wrapper over the package-agnostic `self_update` engine instead.

pub mod edit;
pub mod extract;
pub mod fix;
pub mod lint;
pub mod lookup;
pub mod parse;
pub mod schema;
pub mod update;
pub mod validate;

use std::path::Path;

use anyhow::{bail, Context};
use quire_cli::io::{emit_quire_diagnostics, Diagnostics};
use quire_rs::Registry;

/// Cross-command context plumbed in from global flags.
pub struct Ctx {
    pub diagnostics: Diagnostics,
    pub pretty: bool,
}

/// Load a single module registry for a `--module <PATH>` argument and
/// surface load problems eagerly (FR-004 CR note / upstream
/// FR-013-AC-13).
///
/// The tolerant engine load reports a missing `manifest.yaml` (or an
/// unloadable manifest) as an `ArchetypeLoadFailure` while returning an
/// EMPTY registry; commands that ignored `failures()` then died later
/// with a misleading `UnknownArchetype`. When the load produced zero
/// modules and at least one failure, fail fast with the real reason.
pub fn load_module_registry(ctx: &Ctx, module: &Path) -> anyhow::Result<Registry> {
    let registry = Registry::load_module(module).context("loading module registry")?;
    emit_quire_diagnostics(ctx.diagnostics, registry.diagnostics());
    if registry.module_names().count() == 0 {
        if let Some(f) = registry.failures().first() {
            bail!("module load failed: {} ({})", f.reason, f.path.display());
        }
    }
    Ok(registry)
}
