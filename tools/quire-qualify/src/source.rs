//! Rust-AST-backed source-boundary qualification.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::{LineColumn, Span, TokenStream, TokenTree};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    ExprMethodCall, ExprUnsafe, ItemEnum, ItemExternCrate, ItemStruct, ItemUse, Lit, Macro, UseTree,
};

use crate::{walk, Error, FindingList, Result};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Finding {
    pub path: PathBuf,
    pub line: usize,
    pub reason: String,
}

impl Finding {
    fn new(path: &Path, line: usize, reason: impl Into<String>) -> Self {
        Self {
            path: path.to_path_buf(),
            line,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug)]
struct LocatedPath {
    value: String,
    line: usize,
}

#[derive(Clone, Debug)]
struct Import {
    full: String,
    local: Option<String>,
    glob: bool,
    line: usize,
}

#[derive(Default)]
struct SyntaxIndex {
    paths: Vec<LocatedPath>,
    imports: Vec<Import>,
    methods: Vec<(String, usize)>,
    structs: Vec<(String, usize)>,
    enums: Vec<(String, usize)>,
    unsafe_lines: Vec<usize>,
    literal_spans: Vec<(LineColumn, LineColumn)>,
}

impl SyntaxIndex {
    fn parse(source: &str) -> syn::Result<Self> {
        let file = syn::parse_file(source)?;
        let mut index = Self::default();
        index.visit_file(&file);
        Ok(index)
    }

    fn resolved(&self, path: &LocatedPath) -> Vec<String> {
        let mut results = BTreeSet::from([path.value.clone()]);
        let mut segments = path.value.split("::");
        let Some(first) = segments.next() else {
            return results.into_iter().collect();
        };
        let suffix: Vec<_> = segments.collect();
        for import in self
            .imports
            .iter()
            .filter(|import| import.local.as_deref() == Some(first))
        {
            let mut value = import.full.clone();
            if !suffix.is_empty() {
                value.push_str("::");
                value.push_str(&suffix.join("::"));
            }
            results.insert(value);
        }
        results.into_iter().collect()
    }

    fn all_references(&self) -> Vec<LocatedPath> {
        let mut references = self.paths.clone();
        references.extend(self.imports.iter().map(|import| LocatedPath {
            value: if import.glob {
                format!("{}::*", import.full)
            } else {
                import.full.clone()
            },
            line: import.line,
        }));
        references
    }

    fn has_resolved_suffix(&self, suffix: &str) -> bool {
        self.all_references().iter().any(|path| {
            self.resolved(path)
                .iter()
                .any(|resolved| resolved == suffix || resolved.ends_with(&format!("::{suffix}")))
        })
    }

    fn extend(&mut self, mut other: Self) {
        self.paths.append(&mut other.paths);
        self.imports.append(&mut other.imports);
        self.methods.append(&mut other.methods);
        self.structs.append(&mut other.structs);
        self.enums.append(&mut other.enums);
        self.unsafe_lines.append(&mut other.unsafe_lines);
        self.literal_spans.append(&mut other.literal_spans);
    }
}

impl<'ast> Visit<'ast> for SyntaxIndex {
    fn visit_item_extern_crate(&mut self, node: &'ast ItemExternCrate) {
        self.imports.push(Import {
            full: node.ident.to_string(),
            local: Some(
                node.rename
                    .as_ref()
                    .map_or_else(|| node.ident.to_string(), |(_, rename)| rename.to_string()),
            ),
            glob: false,
            line: node.ident.span().start().line,
        });
        visit::visit_item_extern_crate(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        collect_use_tree(&node.tree, &mut Vec::new(), node.span(), &mut self.imports);
        visit::visit_item_use(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        self.paths.push(LocatedPath {
            value: path_string(node),
            line: node.span().start().line,
        });
        visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        self.methods
            .push((node.method.to_string(), node.method.span().start().line));
        visit::visit_expr_method_call(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.structs
            .push((node.ident.to_string(), node.ident.span().start().line));
        visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        self.enums
            .push((node.ident.to_string(), node.ident.span().start().line));
        visit::visit_item_enum(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast ExprUnsafe) {
        self.unsafe_lines.push(node.unsafe_token.span.start().line);
        visit::visit_expr_unsafe(self, node);
    }

    fn visit_lit(&mut self, node: &'ast Lit) {
        let span = node.span();
        self.literal_spans.push((span.start(), span.end()));
        visit::visit_lit(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        visit::visit_macro(self, node);
        collect_literal_token_spans(node.tokens.clone(), &mut self.literal_spans);
        if let Ok(expression) = syn::parse2::<syn::Expr>(node.tokens.clone()) {
            let mut nested = Self::default();
            nested.visit_expr(&expression);
            self.extend(nested);
        } else if let Ok(statements) = syn::Block::parse_within.parse2(node.tokens.clone()) {
            let mut nested = Self::default();
            for statement in &statements {
                nested.visit_stmt(statement);
            }
            self.extend(nested);
        }
    }
}

fn collect_literal_token_spans(stream: TokenStream, spans: &mut Vec<(LineColumn, LineColumn)>) {
    for token in stream {
        match token {
            TokenTree::Group(group) => collect_literal_token_spans(group.stream(), spans),
            TokenTree::Literal(literal) => {
                let span = literal.span();
                spans.push((span.start(), span.end()));
            }
            TokenTree::Ident(_) | TokenTree::Punct(_) => {}
        }
    }
}

fn path_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn collect_use_tree(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    fallback_span: Span,
    imports: &mut Vec<Import>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_tree(&path.tree, prefix, path.ident.span(), imports);
            prefix.pop();
        }
        UseTree::Name(name) => {
            let name = name.ident.to_string();
            let (full, local) = if name == "self" {
                (prefix.join("::"), prefix.last().cloned())
            } else {
                let mut full = prefix.clone();
                full.push(name.clone());
                (full.join("::"), Some(name))
            };
            imports.push(Import {
                full,
                local,
                glob: false,
                line: name_span(tree).unwrap_or(fallback_span).start().line,
            });
        }
        UseTree::Rename(rename) => {
            let original = rename.ident.to_string();
            let mut full = prefix.clone();
            if original != "self" {
                full.push(original);
            }
            imports.push(Import {
                full: full.join("::"),
                local: Some(rename.rename.to_string()),
                glob: false,
                line: rename.ident.span().start().line,
            });
        }
        UseTree::Glob(glob) => imports.push(Import {
            full: prefix.join("::"),
            local: None,
            glob: true,
            line: glob.star_token.span.start().line,
        }),
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree(item, prefix, group.brace_token.span.open(), imports);
            }
        }
    }
}

fn name_span(tree: &UseTree) -> Option<Span> {
    match tree {
        UseTree::Name(name) => Some(name.ident.span()),
        _ => None,
    }
}

fn relative(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn parse_index(root: &Path, path: &Path, findings: &mut Vec<Finding>) -> Option<SyntaxIndex> {
    let rel = relative(root, path);
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            findings.push(Finding::new(&rel, 0, format!("read Rust source: {error}")));
            return None;
        }
    };
    match SyntaxIndex::parse(&source) {
        Ok(index) => Some(index),
        Err(error) => {
            findings.push(Finding::new(&rel, 0, error.to_string()));
            None
        }
    }
}

fn matches_prefix(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}::"))
}

fn forbidden_reference(
    index: &SyntaxIndex,
    path: &LocatedPath,
    targets: &[&str],
) -> Option<String> {
    index.resolved(path).into_iter().find(|resolved| {
        targets
            .iter()
            .any(|target| matches_prefix(resolved, target))
    })
}

const ENGINE_PRIMITIVES: &[&str] = &[
    "quire_rs::parse_document",
    "quire_rs::render",
    "quire_rs::render_block",
    "quire_rs::render_with_env",
    "quire_rs::validate",
    "quire_rs::validate_all",
    "quire_rs::validate_block",
    "quire_rs::extract",
    "quire_rs::harvest_edges",
    "jsonschema",
    "minijinja",
];

fn is_admitted_dispatch(path: &Path) -> bool {
    path == Path::new("src/main.rs")
        || (path.extension().and_then(|value| value.to_str()) == Some("rs")
            && path.parent() == Some(Path::new("src/commands")))
}

fn audit_general_file(path: &Path, index: &SyntaxIndex, findings: &mut Vec<Finding>) {
    if is_admitted_dispatch(path) {
        return;
    }
    for reference in index.all_references() {
        if let Some(resolved) = forbidden_reference(index, &reference, ENGINE_PRIMITIVES) {
            findings.push(Finding::new(
                path,
                reference.line,
                format!(
                    "forbidden engine reference `{resolved}` outside an admitted dispatch site"
                ),
            ));
        }
    }
    for import in &index.imports {
        if import.glob && matches_prefix(&import.full, "quire_rs") {
            findings.push(Finding::new(
                path,
                import.line,
                format!(
                    "quire-rs glob import `{}` hides the imported primitive",
                    import.full
                ),
            ));
        }
    }
}

fn audit_assurance(path: &Path, index: &SyntaxIndex, findings: &mut Vec<Finding>) {
    const REQUIRED_PATHS: &[&str] = &[
        "Spec::from_path",
        "extract_tree_scoped",
        "symbols::trace::bind",
        "build_assurance_export",
        "read_assurance_export",
    ];
    for required in REQUIRED_PATHS {
        if !index.has_resolved_suffix(required) {
            findings.push(Finding::new(
                path,
                0,
                format!("assurance dispatch does not delegate through `{required}`"),
            ));
        }
    }
    if !index
        .methods
        .iter()
        .any(|(method, _)| method == "to_json_bytes")
    {
        findings.push(Finding::new(
            path,
            0,
            "assurance dispatch does not delegate through `to_json_bytes`",
        ));
    }

    for (name, line) in index.structs.iter().chain(&index.enums) {
        if matches!(name.as_str(), "AssuranceExport" | "AssuranceRelation") {
            findings.push(Finding::new(
                path,
                *line,
                format!("CLI-owned assurance model `{name}` is forbidden"),
            ));
        }
    }

    const FORBIDDEN: &[&str] = &[
        "jsonschema",
        "quire_rs::parse_document",
        "std::process::Command",
        "Command::new",
        "ASSURANCE_V1_SCHEMA",
    ];
    for reference in index.all_references() {
        if let Some(resolved) = forbidden_reference(index, &reference, FORBIDDEN) {
            findings.push(Finding::new(
                path,
                reference.line,
                format!("assurance dispatch contains forbidden boundary logic `{resolved}`"),
            ));
        }
    }
}

fn audit_self_update(root: &Path, findings: &mut Vec<Finding>) {
    let engine_path = root.join("src/self_update/mod.rs");
    let rel = relative(root, &engine_path);
    if let Some(index) = parse_index(root, &engine_path, findings) {
        const FORBIDDEN: &[&str] = &[
            "crate::io",
            "crate::commands",
            "crate::Ctx",
            "quire_cli::io",
            "quire_cli::commands",
        ];
        for reference in index.all_references() {
            if let Some(resolved) = forbidden_reference(&index, &reference, FORBIDDEN) {
                findings.push(Finding::new(
                    &rel,
                    reference.line,
                    format!("self-update engine reaches CLI-specific path `{resolved}`"),
                ));
            }
        }
        for import in &index.imports {
            if import.glob && matches!(import.full.as_str(), "crate" | "quire_cli") {
                findings.push(Finding::new(
                    &rel,
                    import.line,
                    format!(
                        "root glob import `{}` hides CLI-specific dependencies",
                        import.full
                    ),
                ));
            }
        }
    }

    let glue_path = root.join("src/commands/update.rs");
    let glue_rel = relative(root, &glue_path);
    if let Some(index) = parse_index(root, &glue_path, findings) {
        const FORBIDDEN: &[&str] = &[
            "quire_rs::parse_document",
            "quire_rs::validate",
            "quire_rs::extract",
        ];
        for reference in index.all_references() {
            if let Some(resolved) = forbidden_reference(&index, &reference, FORBIDDEN) {
                findings.push(Finding::new(
                    &glue_rel,
                    reference.line,
                    format!("self-update glue contains document primitive `{resolved}`"),
                ));
            }
        }
    }
}

/// Return every thin-boundary violation in deterministic path/line order.
pub fn thin_boundary_findings(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for path in walk::rust_files(root, &["src"], &[])? {
        let rel = relative(root, &path);
        let Some(index) = parse_index(root, &path, &mut findings) else {
            continue;
        };
        audit_general_file(&rel, &index, &mut findings);
        if rel == Path::new("src/commands/assurance.rs") {
            audit_assurance(&rel, &index, &mut findings);
        }
    }
    audit_self_update(root, &mut findings);

    let checklist = "Does any new logic belong upstream in `quire-rs`?";
    let contributing_path = root.join("CONTRIBUTING.md");
    let contributing = fs::read_to_string(&contributing_path).map_err(|source| Error::Read {
        path: contributing_path,
        source,
    })?;
    if !contributing
        .lines()
        .any(|line| line == format!("- {checklist}"))
    {
        findings.push(Finding::new(
            Path::new("CONTRIBUTING.md"),
            0,
            "missing upstream-ownership review question",
        ));
    }

    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// Enforce the complete Rust-source thin boundary.
pub fn check_thin_boundary(root: &Path) -> Result<()> {
    let findings = thin_boundary_findings(root)?;
    if findings.is_empty() {
        return Ok(());
    }
    Err(Error::ThinBoundary {
        findings: FindingList::from(findings),
    })
}

pub(crate) fn unsafe_lines(source: &str) -> syn::Result<Vec<usize>> {
    Ok(SyntaxIndex::parse(source)?.unsafe_lines)
}

pub(crate) fn safety_comment_lines(source: &str) -> syn::Result<BTreeSet<usize>> {
    let index = SyntaxIndex::parse(source)?;
    let mut comments = BTreeSet::new();
    for (line_index, line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        for (column, _) in line.match_indices("// SAFETY:") {
            let inside_literal = index.literal_spans.iter().any(|(start, end)| {
                let after_start = line_number > start.line
                    || (line_number == start.line && column >= start.column);
                let before_end =
                    line_number < end.line || (line_number == end.line && column < end.column);
                after_start && before_end
            });
            if !inside_literal {
                comments.insert(line_number);
            }
        }
    }
    Ok(comments)
}

/// Locate Rust syntax that imports or references process-command execution.
pub fn process_execution_lines(source: &str) -> Result<Vec<usize>> {
    let index = SyntaxIndex::parse(source).map_err(|source| Error::ParseRustInput { source })?;
    let mut lines = BTreeSet::new();
    for reference in index.all_references() {
        if forbidden_reference(
            &index,
            &reference,
            &["std::process::Command", "Command::new"],
        )
        .is_some()
        {
            lines.insert(reference.line);
        }
    }
    for import in &index.imports {
        if import.glob && matches_prefix(&import.full, "std::process") {
            lines.insert(import.line);
        }
    }
    Ok(lines.into_iter().collect())
}
