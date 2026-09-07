//! `quire assurance` — the quire-rs assurance-v1 export (FR-020).
//!
//! This command owns only the process boundary: exact paths and premises,
//! diagnostic channels, and stdout. Quire-rs owns the corpus, symbol graph,
//! projection, schema, and fail-closed reader.

use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use anyhow::{bail, Context};
use clap::Parser;
use quire_cli::{io, safety};
use quire_rs::assurance::{AssuranceModulePremise, AssuranceSchemaPremise};
use quire_rs::symbols::trace::SymbolGraph;
use quire_rs::{
    build_assurance_export, read_assurance_export, AcceptedAssurancePremises, AssuranceInput,
    AssuranceSource, Spec,
};

use super::Ctx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedModule {
    name: String,
    version: String,
}

impl FromStr for ExpectedModule {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (name, version) = value
            .rsplit_once('@')
            .ok_or_else(|| "expected NAME@VERSION".to_string())?;
        if name.is_empty() || version.is_empty() {
            return Err("expected non-empty NAME@VERSION".to_string());
        }
        Ok(Self {
            name: name.to_string(),
            version: version.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedSchema {
    module: String,
    archetype: String,
    digest: String,
}

impl FromStr for ExpectedSchema {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (identity, digest) = value
            .rsplit_once('@')
            .ok_or_else(|| "expected MODULE/ARCHETYPE@SHA256".to_string())?;
        let (module, archetype) = identity
            .rsplit_once('/')
            .ok_or_else(|| "expected MODULE/ARCHETYPE@SHA256".to_string())?;
        if module.is_empty() || archetype.is_empty() {
            return Err("expected non-empty MODULE/ARCHETYPE@SHA256".to_string());
        }
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err("schema digest must be 64 lowercase hexadecimal characters".to_string());
        }
        Ok(Self {
            module: module.to_string(),
            archetype: archetype.to_string(),
            digest: digest.to_string(),
        })
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    /// Repository root. Documents are read from <scope>/spec and source from
    /// <scope> excluding spec/ and module-declared source exclusions.
    #[arg(long, default_value = ".")]
    pub scope: String,

    /// One exact module directory containing manifest.yaml. No discovery or
    /// lazy installation is performed.
    #[arg(long, value_name = "PATH")]
    pub module: String,

    /// Non-empty repository identity copied into the assurance source premise.
    #[arg(long, value_name = "IDENTITY")]
    pub repository: String,

    /// Caller-selected immutable revision: 40 lowercase hexadecimal digits.
    #[arg(long, value_name = "FULL_SHA")]
    pub revision: String,

    /// The only accepted module premise, as NAME@VERSION.
    #[arg(long, value_name = "NAME@VERSION")]
    pub expect_module: ExpectedModule,

    /// One accepted active-archetype premise, repeatable. Supplying none means
    /// the expected module has no active archetypes.
    #[arg(long, value_name = "MODULE/ARCHETYPE@SHA256")]
    pub expect_schema: Vec<ExpectedSchema>,
}

pub fn run(ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let scope = safety::validate_dir_path("--scope", &args.scope)
        .with_context(|| format!("validating --scope '{}'", args.scope))?;
    let module = safety::validate_module_path(&args.module)
        .with_context(|| format!("validating --module '{}'", args.module))?;
    let registry = super::load_module_registry(ctx, &module)?;

    let spec_root = super::spec_root_of(&scope)?;
    let spec_root = safety::validate_dir_path("document root", &spec_root.display().to_string())
        .with_context(|| format!("validating document root '{}'", spec_root.display()))?;
    let spec = Spec::from_path(&spec_root);
    io::emit_quire_diagnostics(ctx.diagnostics, spec.diagnostics());

    let empty_excludes = Vec::new();
    let model = registry.traceability();
    let extraction = quire_rs::symbols::extract_tree_scoped(
        &scope,
        &[Path::new(super::DOCUMENT_ROOT_DIR)],
        model
            .map(|traceability| &traceability.source_exclude)
            .unwrap_or(&empty_excludes),
    );
    for diagnostic in &extraction.diagnostics {
        io::emit_diagnostic(
            ctx.diagnostics,
            "SymbolExtraction",
            &format!("{}: {}", diagnostic.path, diagnostic.reason),
        );
    }
    let graph = model
        .map(|traceability| quire_rs::symbols::trace::bind(&extraction, traceability))
        .unwrap_or_else(SymbolGraph::default);

    let export = build_assurance_export(AssuranceInput {
        spec: &spec,
        registry: &registry,
        corpus_root: &spec_root,
        symbols: &extraction,
        symbol_graph: &graph,
        source: AssuranceSource {
            repository: args.repository,
            revision: args.revision,
        },
    })
    .map_err(|error| anyhow::anyhow!(error))?;

    let accepted = accepted_premises(args.expect_module, args.expect_schema)?;
    let compact = export
        .to_json_bytes()
        .map_err(|error| anyhow::anyhow!(error))?;
    let validated =
        read_assurance_export(&compact, &accepted).map_err(|error| anyhow::anyhow!(error))?;
    if validated.modules != accepted.modules {
        bail!(
            "assurance premise set does not exactly match the emitted module/schema set: expected {:?}, emitted {:?}",
            accepted.modules,
            validated.modules
        );
    }

    let mut output = if ctx.pretty {
        io::pretty_validated_json_bytes(&compact)
    } else {
        compact
    };
    output.push(b'\n');
    std::io::stdout()
        .lock()
        .write_all(&output)
        .context("writing assurance JSON to stdout")?;
    Ok(())
}

fn accepted_premises(
    module: ExpectedModule,
    schemas: Vec<ExpectedSchema>,
) -> anyhow::Result<AcceptedAssurancePremises> {
    let mut accepted_schemas = Vec::with_capacity(schemas.len());
    for schema in schemas {
        if schema.module != module.name {
            bail!(
                "schema premise module '{}' does not match expected module '{}'",
                schema.module,
                module.name
            );
        }
        accepted_schemas.push(AssuranceSchemaPremise {
            archetype: schema.archetype,
            schema_digest: schema.digest,
        });
    }
    accepted_schemas.sort();
    if accepted_schemas.windows(2).any(|pair| pair[0] == pair[1]) {
        bail!("assurance schema premise set contains a duplicate tuple");
    }
    Ok(AcceptedAssurancePremises {
        format_version: 1,
        modules: vec![AssuranceModulePremise {
            name: module.name,
            version: module.version,
            schemas: accepted_schemas,
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premise_parsers_reject_ambiguous_or_noncanonical_forms() {
        assert!(ExpectedModule::from_str("module@1.2.3").is_ok());
        assert!(ExpectedModule::from_str("module").is_err());
        assert!(ExpectedModule::from_str("@1.2.3").is_err());
        assert!(ExpectedModule::from_str("module@").is_err());

        let digest = "a".repeat(64);
        assert!(ExpectedSchema::from_str(&format!("module/FR@{digest}")).is_ok());
        let scoped = ExpectedSchema::from_str(&format!("scope/module/FR@{digest}"))
            .expect("rightmost slash separates archetype");
        assert_eq!(scoped.module, "scope/module");
        assert_eq!(scoped.archetype, "FR");
        assert!(ExpectedSchema::from_str("module/FR@ABC").is_err());
        assert!(ExpectedSchema::from_str(&format!("module@FR@{digest}")).is_err());
    }

    #[test]
    fn accepted_set_rejects_cross_module_and_duplicate_schema_tuples() {
        let module = ExpectedModule::from_str("module@1.2.3").expect("module");
        let digest = "a".repeat(64);
        let wrong = ExpectedSchema::from_str(&format!("other/FR@{digest}")).expect("schema");
        assert!(accepted_premises(module.clone(), vec![wrong]).is_err());

        let duplicate = ExpectedSchema::from_str(&format!("module/FR@{digest}")).expect("schema");
        assert!(accepted_premises(module, vec![duplicate.clone(), duplicate]).is_err());
    }
}
