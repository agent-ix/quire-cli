//! The JSON report path of `quire export` (FR-031 section B).
//!
//! A JSON report is read as data and rendered as Markdown pipe tables, the one
//! Markdown text handed to the converter (StR-004 bounded exception: output
//! formatting, not Markdown parsing). Every text that comes from the report is
//! escaped so the converter shows it as literal text and a report cannot
//! inject raw Typst, OpenXML, HTML or Markdown structure.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use clap::ValueEnum;
use serde_json::{Map, Value};

use super::{producer, write_output, Args, Kind};

const MATRIX_EMPTY: &str = "No obligations matched this scope.";
const RMM_SINGLE: &str = "requirements-mapping-v1";
const RMM_PACK: &str = "requirements-mapping-pack-v1";
/// Criterion members of `quire matrix --format json`, in column order.
const MATRIX_COLUMNS: [&str; 5] = ["id", "statement", "method", "binders", "status"];
/// `RmmRow` members of `quoin discharge --rmm --json`, in column order.
const RMM_COLUMNS: [&str; 8] = [
    "clauseId",
    "force",
    "applicability",
    "compliance",
    "reason",
    "evidenceRefs",
    "tailoringDescription",
    "tailoringRationale",
];
const RMM_INDEX_COLUMNS: [&str; 2] = ["clauseSet", "rowCount"];
const UNUSED_FACT_COLUMNS: [&str; 3] = ["clauseId", "kind", "reason"];
/// Row members of `quoin soup report --json`, in column order.
const SOUP_COLUMNS: [&str; 14] = [
    "purl",
    "id",
    "path",
    "version",
    "serves",
    "known_anomalies",
    "support_status",
    "end_of_support",
    "unresolved_advisory",
    "past_end_of_support",
    "support_disagreement",
    "disposition_clause",
    "advisories",
    "no_advisory_reported_by",
];
const SOUP_UNMATCHED_COLUMNS: [&str; 5] = ["tool", "rule_id", "package", "severity", "reason"];
const SOUP_SCAN_COLUMNS: [&str; 2] = ["tool", "state"];
/// The narrowest and widest relative width a column asks for, in characters.
const MIN_COLUMN_WIDTH: usize = 3;
const MAX_COLUMN_WIDTH: usize = 48;

/// Renders the report to Markdown and converts it. Shape errors are exit 3 and
/// are raised before any converter is resolved or started.
pub(super) fn export(args: &Args, kind: Kind, text: &str) -> anyhow::Result<()> {
    let name = args.input.display().to_string();
    let markdown = to_markdown(kind, &name, text)?;
    let converters = producer::Converters::resolve(args)?;
    let generated = producer::Generated {
        body: &markdown,
        metadata: "{}\n",
        identity_parts: None,
        epoch: None,
    };
    let rendered = producer::convert(&converters, args.to, &generated)?;
    write_output(&args.output, rendered.as_ref())
}

/// The Markdown text of the report, or an invalid-input error naming `name`.
pub(super) fn to_markdown(kind: Kind, name: &str, text: &str) -> anyhow::Result<String> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| invalid_error!("'{name}': not valid JSON: {error}"))?;
    let Value::Object(document) = &value else {
        invalid_request!("'{name}': a JSON report must be an object");
    };
    let mut out = Markdown::default();
    let shaped = match kind {
        Kind::Matrix => matrix(document, &mut out),
        Kind::Rmm => rmm(document, &mut out),
        Kind::Soup => soup(document, &mut out),
        Kind::Table => table(document, &mut out),
    };
    let kind_name = kind
        .to_possible_value()
        .map_or_else(String::new, |value| value.get_name().to_string());
    shaped.map_err(|reason| {
        invalid_error!("'{name}': does not match --kind {kind_name}: {reason}")
    })?;
    Ok(out.0)
}

/// The Markdown being assembled; every block ends with a blank line.
#[derive(Default)]
struct Markdown(String);

impl Markdown {
    fn line(&mut self, text: &str) {
        self.0.push_str(text);
        self.0.push_str("\n\n");
    }

    fn heading(&mut self, text: &str) {
        self.line(&format!("## {}", escape(text)));
    }

    fn as_of(&mut self, value: &Value) {
        self.line(&format!("As of: {}", escape(&cell_text(value))));
    }

    fn bullets(&mut self, items: &[Value]) {
        for item in items {
            let _ = writeln!(self.0, "- {}", escape(&cell_text(item)));
        }
        self.0.push('\n');
    }

    /// A pipe table over `rows`. The `fixed` columns come first, always, in
    /// order; members outside them follow, sorted by name.
    fn table(&mut self, rows: &[&Map<String, Value>], fixed: &[&str]) {
        let extras: BTreeSet<&str> = rows
            .iter()
            .flat_map(|row| row.keys().map(String::as_str))
            .filter(|key| !fixed.contains(key))
            .collect();
        let columns: Vec<&str> = fixed.iter().copied().chain(extras).collect();
        let cells: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                columns
                    .iter()
                    .map(|column| row.get(*column).map(cell_text).unwrap_or_default())
                    .collect()
            })
            .collect();
        // Relative dash counts steer pandoc's column widths once a row is too
        // long for the page, so long text wraps instead of overflowing.
        let widths: Vec<usize> = columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                cells
                    .iter()
                    .map(|row| row[index].chars().count())
                    .chain([column.chars().count()])
                    .max()
                    .unwrap_or(3)
                    .clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH)
            })
            .collect();
        let line = |texts: Vec<String>| format!("| {} |\n", texts.join(" | "));
        self.0
            .push_str(&line(columns.iter().map(|column| escape(column)).collect()));
        self.0.push_str(&line(
            widths.iter().map(|width| "-".repeat(*width)).collect(),
        ));
        for row in &cells {
            self.0
                .push_str(&line(row.iter().map(|text| escape(text)).collect()));
        }
        self.0.push('\n');
    }
}

/// A scalar as its text, a nested value as its compact JSON text.
fn cell_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Text that the converter shows literally: every ASCII punctuation character
/// is backslash-escaped and every line break or control character becomes a
/// space, so a cell never splits a row and no markup survives.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_ascii_punctuation() {
            out.push('\\');
            out.push(ch);
        } else if ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}') {
            out.push(' ');
        } else {
            out.push(ch);
        }
    }
    out
}

type Shape = Result<(), String>;

fn array_of_objects<'a>(
    value: &'a Value,
    what: &str,
) -> Result<Vec<&'a Map<String, Value>>, String> {
    let Value::Array(items) = value else {
        return Err(format!("'{what}' must be an array"));
    };
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_object()
                .ok_or_else(|| format!("'{what}' element {index} must be an object"))
        })
        .collect()
}

fn required<'a>(document: &'a Map<String, Value>, member: &str) -> Result<&'a Value, String> {
    document
        .get(member)
        .ok_or_else(|| format!("the '{member}' member is missing"))
}

/// A table headed `heading`; an empty array still shows its heading.
fn member_table(out: &mut Markdown, heading: &str, rows: &[&Map<String, Value>], fixed: &[&str]) {
    out.heading(heading);
    if rows.is_empty() {
        out.line("No rows.");
    } else {
        out.table(rows, fixed);
    }
}

fn matrix(document: &Map<String, Value>, out: &mut Markdown) -> Shape {
    let entries = match document.get("coverage_matrix") {
        None => Vec::new(),
        Some(value) => array_of_objects(value, "coverage_matrix")?,
    };
    if entries.is_empty() {
        out.line(MATRIX_EMPTY);
        return Ok(());
    }
    for (index, entry) in entries.iter().enumerate() {
        let name = format!("coverage_matrix[{index}].criteria");
        let rows = array_of_objects(required(entry, "criteria")?, &name)?;
        let heading = required(entry, "document")?;
        member_table(out, &cell_text(heading), &rows, &MATRIX_COLUMNS);
    }
    Ok(())
}

/// The clause set key `authority/id@version`, or its text when not that shape.
fn clause_set_heading(value: &Value) -> String {
    let part = |name: &str| value.get(name).and_then(Value::as_str);
    match (part("authority"), part("id"), part("version")) {
        (Some(authority), Some(id), Some(version)) => format!("{authority}/{id}@{version}"),
        _ => cell_text(value),
    }
}

/// One `requirements-mapping-v1` document: its table, then `unusedFacts`. In a
/// pack, the `unusedFacts` heading names the set.
fn rmm_matrix(document: &Map<String, Value>, in_pack: bool, out: &mut Markdown) -> Shape {
    let rows = array_of_objects(required(document, "rows")?, "rows")?;
    if let Some(as_of) = document.get("asOf") {
        out.as_of(as_of);
    }
    let set = document.get("clauseSet").map(clause_set_heading);
    member_table(out, set.as_deref().unwrap_or("rows"), &rows, &RMM_COLUMNS);
    if let Some(facts) = document.get("unusedFacts") {
        let facts = array_of_objects(facts, "unusedFacts")?;
        if !facts.is_empty() {
            let heading = match (&set, in_pack) {
                (Some(set), true) => format!("unusedFacts: {set}"),
                _ => "unusedFacts".to_string(),
            };
            member_table(out, &heading, &facts, &UNUSED_FACT_COLUMNS);
        }
    }
    Ok(())
}

/// `index` rows with `clauseSet` shown as its key, as the matrix headings do.
fn index_rows(index: &[&Map<String, Value>]) -> Vec<Map<String, Value>> {
    index
        .iter()
        .map(|row| {
            let mut row = (*row).clone();
            if let Some(set) = row.get_mut("clauseSet") {
                *set = Value::String(clause_set_heading(set));
            }
            row
        })
        .collect()
}

/// The pack's `context` as key/value rows.
fn context_rows(document: &Map<String, Value>) -> Result<Vec<Map<String, Value>>, String> {
    let Some(context) = document.get("context") else {
        return Ok(Vec::new());
    };
    let Value::Object(context) = context else {
        return Err("'context' must be an object".to_string());
    };
    Ok(context
        .iter()
        .map(|(key, value)| {
            let mut row = Map::new();
            row.insert("key".to_string(), Value::String(key.clone()));
            row.insert("value".to_string(), value.clone());
            row
        })
        .collect())
}

fn rmm(document: &Map<String, Value>, out: &mut Markdown) -> Shape {
    match document.get("schemaVersion").and_then(Value::as_str) {
        Some(RMM_SINGLE) => rmm_matrix(document, false, out),
        Some(RMM_PACK) => {
            let index = array_of_objects(required(document, "index")?, "index")?;
            let matrices = array_of_objects(required(document, "matrices")?, "matrices")?;
            let context = context_rows(document)?;
            // Shape-check every matrix before writing, so nothing partial is built.
            let mut rendered = Markdown::default();
            for (position, matrix) in matrices.iter().enumerate() {
                rmm_matrix(matrix, true, &mut rendered)
                    .map_err(|reason| format!("matrices[{position}]: {reason}"))?;
            }
            if let Some(pack) = document.get("pack") {
                out.line(&format!("Pack: {}", escape(&cell_text(pack))));
            }
            if let Some(as_of) = document.get("asOf") {
                out.as_of(as_of);
            }
            if document.contains_key("context") {
                let rows: Vec<&Map<String, Value>> = context.iter().collect();
                member_table(out, "context", &rows, &["key", "value"]);
            }
            let index = index_rows(&index);
            let index: Vec<&Map<String, Value>> = index.iter().collect();
            member_table(out, "index", &index, &RMM_INDEX_COLUMNS);
            out.0.push_str(&rendered.0);
            Ok(())
        }
        _ => Err(format!(
            "'schemaVersion' must be '{RMM_SINGLE}' or '{RMM_PACK}'"
        )),
    }
}

fn soup(document: &Map<String, Value>, out: &mut Markdown) -> Shape {
    let rows = array_of_objects(required(document, "rows")?, "rows")?;
    let mut tables = Vec::new();
    for (member, fixed) in [
        ("unmatched", &SOUP_UNMATCHED_COLUMNS[..]),
        ("scans", &SOUP_SCAN_COLUMNS[..]),
    ] {
        if let Some(value) = document.get(member) {
            tables.push((member, fixed, array_of_objects(value, member)?));
        }
    }
    let unreadable = match document.get("unreadable_scans") {
        None => &[][..],
        Some(Value::Array(items)) => items,
        Some(_) => return Err("'unreadable_scans' must be an array".to_string()),
    };
    if let Some(as_of) = document.get("as_of") {
        out.as_of(as_of);
    }
    member_table(out, "rows", &rows, &SOUP_COLUMNS);
    for (member, fixed, rows) in tables.iter().filter(|(_, _, rows)| !rows.is_empty()) {
        member_table(out, member, rows, fixed);
    }
    if !unreadable.is_empty() {
        out.heading("unreadable_scans");
        out.bullets(unreadable);
    }
    Ok(())
}

/// Every array member is a table; a member holding a non-object is refused
/// rather than dropped. Members that are not arrays are not tables.
fn table(document: &Map<String, Value>, out: &mut Markdown) -> Shape {
    let mut tables = Vec::new();
    for (member, value) in document {
        if value.is_array() {
            tables.push((member, array_of_objects(value, member)?));
        }
    }
    if tables.is_empty() {
        return Err("no member is an array of objects".to_string());
    }
    for (member, rows) in tables {
        member_table(out, member, &rows, &[]);
    }
    Ok(())
}
