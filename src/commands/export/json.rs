//! The JSON report path of `quire export` (FR-031 section B).
//!
//! A JSON report is read as data and rendered as Markdown pipe tables, the one
//! Markdown text handed to the converter (StR-004 bounded exception: output
//! formatting, not Markdown parsing). Every text that comes from the report is
//! escaped so the converter shows it as literal text and a report cannot
//! inject raw Typst, OpenXML, HTML or Markdown structure.

use std::fmt::Write as _;

use serde_json::{Map, Value};

use super::{producer, write_output, Args, Kind};

const MATRIX_EMPTY: &str = "No obligations matched this scope.";
const RMM_SINGLE: &str = "requirements-mapping-v1";
const RMM_PACK: &str = "requirements-mapping-pack-v1";
/// The widest relative width a column asks for, in characters.
const MAX_COLUMN_WIDTH: usize = 48;

/// Renders the report to Markdown and converts it. Shape errors are exit 3 and
/// are raised before any converter is resolved or started.
pub(super) fn export(args: &Args, kind: Kind, text: &str) -> anyhow::Result<()> {
    let name = args.input.display().to_string();
    let markdown = to_markdown(kind, &name, text)?;
    let converters = producer::Converters::resolve(args)?;
    let rendered = producer::convert(&converters, args.to, &markdown, "{}\n")?;
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
    shaped
        .map_err(|reason| invalid_error!("'{name}': does not match --kind {kind:?}: {reason}"))?;
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

    /// A pipe table over `rows`, columns in first-seen member order.
    fn table(&mut self, rows: &[&Map<String, Value>]) {
        let mut columns: Vec<&str> = Vec::new();
        for row in rows {
            for key in row.keys() {
                if !columns.iter().any(|known| known == key) {
                    columns.push(key);
                }
            }
        }
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
                    .clamp(3, MAX_COLUMN_WIDTH)
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

/// A table headed by `member`; an empty array still shows its heading.
fn member_table(out: &mut Markdown, heading: &str, rows: &[&Map<String, Value>]) {
    out.heading(heading);
    if rows.is_empty() {
        out.line("No rows.");
    } else {
        out.table(rows);
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
        member_table(out, &cell_text(heading), &rows);
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

/// One `requirements-mapping-v1` document: its table, then `unusedFacts`.
fn rmm_matrix(document: &Map<String, Value>, out: &mut Markdown) -> Shape {
    let rows = array_of_objects(required(document, "rows")?, "rows")?;
    if let Some(as_of) = document.get("asOf") {
        out.as_of(as_of);
    }
    let heading = document
        .get("clauseSet")
        .map_or_else(|| "rows".to_string(), clause_set_heading);
    member_table(out, &heading, &rows);
    if let Some(facts) = document.get("unusedFacts") {
        let facts = array_of_objects(facts, "unusedFacts")?;
        if !facts.is_empty() {
            member_table(out, "unusedFacts", &facts);
        }
    }
    Ok(())
}

fn rmm(document: &Map<String, Value>, out: &mut Markdown) -> Shape {
    match document.get("schemaVersion").and_then(Value::as_str) {
        Some(RMM_SINGLE) => rmm_matrix(document, out),
        Some(RMM_PACK) => {
            let index = array_of_objects(required(document, "index")?, "index")?;
            let matrices = array_of_objects(required(document, "matrices")?, "matrices")?;
            // Shape-check every matrix before writing, so nothing partial is built.
            let mut rendered = Markdown::default();
            for (position, matrix) in matrices.iter().enumerate() {
                rmm_matrix(matrix, &mut rendered)
                    .map_err(|reason| format!("matrices[{position}]: {reason}"))?;
            }
            if let Some(as_of) = document.get("asOf") {
                out.as_of(as_of);
            }
            member_table(out, "index", &index);
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
    for member in ["unmatched", "scans"] {
        if let Some(value) = document.get(member) {
            tables.push((member, array_of_objects(value, member)?));
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
    member_table(out, "rows", &rows);
    for (member, rows) in tables.iter().filter(|(_, rows)| !rows.is_empty()) {
        member_table(out, member, rows);
    }
    if !unreadable.is_empty() {
        out.heading("unreadable_scans");
        out.bullets(unreadable);
    }
    Ok(())
}

fn table(document: &Map<String, Value>, out: &mut Markdown) -> Shape {
    let mut found = false;
    for (member, value) in document {
        if let Ok(rows) = array_of_objects(value, member) {
            found = true;
            member_table(out, member, &rows);
        }
    }
    if found {
        Ok(())
    } else {
        Err("no member is an array of objects".to_string())
    }
}
