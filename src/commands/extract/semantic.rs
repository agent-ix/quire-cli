//! Assemble explicit corpus inputs for the engine semantic extractor.

use std::collections::BTreeMap;

use ix_trace_rs::implements;
use quire_rs::semantic::{
    BundleIndex, ConstructReferences, ImportedArtifact, RelationVocabulary, RequiredSections,
    SemanticContext, SemanticSeverity,
};
use quire_rs::{CompiledArchetype, Registry, SemanticExtraction};

use super::super::corpus_input::Corpus;
use super::{Args, Ctx};

#[implements("FR-003-AC-7", "FR-003-AC-8", "FR-003-AC-9")]
#[implements("FR-003-AC-10")]
pub(super) fn extract(
    ctx: &Ctx,
    args: &Args,
    registry: &Registry,
    archetype: &CompiledArchetype,
    text: &str,
) -> anyhow::Result<SemanticExtraction> {
    let path = quire_cli::safety::validate_input_path("document", &args.doc)?;
    let corpus = Corpus::from_repos(&args.repo)?;
    let mut owners = Vec::new();
    for (index, member) in corpus.members.iter().enumerate() {
        for doc in &member.load.documents {
            let loaded = doc
                .path
                .canonicalize()
                .map_err(|error| super::super::failure::input(error.into()))?;
            if loaded == path {
                owners.push(index);
            }
        }
    }
    let [owner] = owners.as_slice() else {
        invalid_request!(
            "semantic document must be loaded from exactly one declared --repo member"
        );
    };
    let member = &corpus.members[*owner];
    let identity = &args.repo[*owner].identity;
    let module = registry.semantic_module(&archetype.module).ok_or_else(|| {
        invalid_error!(
            "archetype '{}' has no semantic module contract",
            archetype.name
        )
    })?;
    let mut bundle = BundleIndex::from_documents(
        identity,
        member
            .load
            .documents
            .iter()
            .filter_map(|doc| Some((doc.frontmatter()?, doc.raw()))),
        registry.semantic_modules().map(|(_, module)| module),
    );
    // Artifact kinds can be declared by `type` alone (for example RoleSpec).
    for artifact in &mut bundle.artifacts {
        if let Some(doc) = member
            .load
            .documents
            .iter()
            .find(|doc| doc.id == artifact.id)
        {
            artifact.object = artifact
                .object
                .take()
                .or_else(|| doc.concept_type().map(str::to_string));
        }
    }
    for (index, sibling) in corpus.members.iter().enumerate() {
        if index == *owner {
            continue;
        }
        for doc in &sibling.load.documents {
            let Some(fm) = doc.frontmatter() else {
                continue;
            };
            let Some(id) = fm.get("id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let names = ["id", "title", "name"]
                .into_iter()
                .filter_map(|key| {
                    fm.get(key)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
                .collect();
            bundle.imported_artifacts.push(ImportedArtifact {
                package: args.repo[index].identity.clone(),
                id: id.to_string(),
                names,
                object: fm
                    .get("object")
                    .and_then(serde_json::Value::as_str)
                    .or_else(|| doc.concept_type())
                    .map(str::to_string),
                source_span: None,
            });
        }
    }
    let relative = path
        .strip_prefix(&member.root)
        .map_err(|_| invalid_error!("semantic document resolves outside its declared member"))?;
    let mut context = SemanticContext::new(module.clone(), relative.to_string_lossy(), bundle)
        .with_source_identity(format!("ix://{identity}/spec"))
        .with_object_type(&archetype.name)
        .with_relation_vocabulary(RelationVocabulary::from_registry(registry, archetype));
    if let Some(references) = archetype
        .construct()
        .and_then(ConstructReferences::from_construct)
    {
        context = context.with_construct_references(references);
    }
    if let Some(schema) = archetype.data_schema.as_deref() {
        let empty = BTreeMap::new();
        context = context.with_data_schema(schema).with_record_schema(
            schema,
            archetype.data_schema_documents.as_deref().unwrap_or(&empty),
        );
    }
    if let Some(dsl) = archetype.body_extraction() {
        context = context.with_body_extraction(dsl);
    }
    let required = archetype
        .body_extraction()
        .map(RequiredSections::from_extraction)
        .unwrap_or_default();
    let extraction = quire_rs::extract_semantic(text, &context, &required);
    let mut errors = false;
    for diagnostic in &extraction.diagnostics {
        let message = format!("{}: {}", diagnostic.code, diagnostic.message);
        let record = match diagnostic.severity {
            SemanticSeverity::Error => {
                errors = true;
                ix_cli_kit::streams::Record::error(&diagnostic.code, &message)
            }
            SemanticSeverity::Advisory | SemanticSeverity::Warning => {
                ix_cli_kit::streams::Record::warning(&diagnostic.code, &message)
            }
        };
        record.emit(ctx.diagnostics);
    }
    if errors {
        invalid_request!("semantic extraction failed");
    }
    Ok(extraction)
}
