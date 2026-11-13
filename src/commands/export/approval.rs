//! `--approval <PATH>` (FR-038): the approval block generated from verdict files
//! of `engineering-assurance approval-verify`. The export verifies nothing and
//! reads no repository here; each verdict is data that is presented as it says.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use super::custom::{invalid, read_bounded};
use super::template::escape_markdown;

const RESULT_PROTOCOL: &str = "engineering-assurance.approval-verify-result/v1";
const APPROVED: &str = "approved";
const NOT_AVAILABLE: &str = "not available";
const NOT_RECORDED: &str = "not recorded";
const UNMAPPED: &str = "unmapped";
const COLUMNS: [&str; 5] = [
    "Person",
    "Signed at (signer's claim)",
    "Meaning",
    "Verdict",
    "Record",
];
/// The narrowest and widest relative width a column asks for, in characters.
const MIN_COLUMN_WIDTH: usize = 3;
const MAX_COLUMN_WIDTH: usize = 40;

/// The members of a verdict file that the block shows. The verifier may add
/// others; they are not consulted.
#[derive(Debug, Deserialize)]
struct Verdict {
    status: String,
    reason: Option<String>,
    tag: Option<String>,
    record_id: Option<String>,
    record_digest: Option<String>,
    approver: Option<String>,
    meaning: Option<Vec<String>>,
    signed_at: Option<String>,
    signature: Option<Signature>,
}

#[derive(Debug, Deserialize)]
struct Signature {
    signer: Option<Signer>,
    signed_off_meaning: Option<Meaning>,
}

#[derive(Debug, Deserialize)]
struct Signer {
    principal: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Meaning {
    values: Option<Vec<String>>,
}

/// The text of `in/approval.md` for the verdict files, in the order given, or
/// `None` without any. The first unusable file is an invalid-input error
/// naming it.
pub(super) fn load(paths: &[PathBuf]) -> anyhow::Result<Option<String>> {
    if paths.is_empty() {
        return Ok(None);
    }
    let mut rows = Vec::with_capacity(paths.len());
    for path in paths {
        let bytes = read_bounded("--approval", path)?;
        let text =
            String::from_utf8(bytes).map_err(|_| invalid("--approval", path, "not valid UTF-8"))?;
        rows.push(row(&parse(path, &text)?, path)?);
    }
    Ok(Some(markdown(&rows)))
}

fn parse(path: &Path, text: &str) -> anyhow::Result<Verdict> {
    let name = path.display();
    let value: Value = serde_json::from_str(text)
        .map_err(|error| invalid_error!("--approval '{name}': not valid JSON: {error}"))?;
    if value.get("protocol").and_then(Value::as_str) != Some(RESULT_PROTOCOL) {
        invalid_request!("--approval '{name}': not an {RESULT_PROTOCOL} document");
    }
    if value.get("status").is_none() {
        invalid_request!("--approval '{name}': the verdict lacks `status`");
    }
    serde_json::from_value(value)
        .map_err(|error| invalid_error!("--approval '{name}': unusable verdict: {error}"))
}

/// The five cell texts of one entry, unescaped.
fn row(verdict: &Verdict, path: &Path) -> anyhow::Result<[String; 5]> {
    let approved = verdict.status == APPROVED;
    let signer = verdict.signature.as_ref().and_then(|s| s.signer.as_ref());
    let person = verdict
        .approver
        .as_deref()
        .or_else(|| signer.and_then(|s| s.principal.as_deref()))
        .unwrap_or(UNMAPPED);
    // A time beside a signature the verifier did not accept as an approval
    // would read as attested, so only an `approved` entry shows one.
    let signed_at = match verdict.signed_at.as_deref().filter(|_| approved) {
        Some(instant) => format_instant(instant).ok_or_else(|| {
            invalid_error!(
                "--approval '{}': signed_at '{instant}' is not a UTC instant YYYY-MM-DDTHH:MM:SSZ",
                path.display()
            )
        })?,
        None => NOT_AVAILABLE.to_string(),
    };
    let recorded = verdict
        .meaning
        .as_deref()
        .or_else(|| {
            verdict
                .signature
                .as_ref()
                .and_then(|s| s.signed_off_meaning.as_ref())
                .and_then(|m| m.values.as_deref())
        })
        .filter(|values| !values.is_empty());
    let meaning = match recorded {
        Some(values) if approved => quoted(values),
        Some(values) => format!("claimed: {}", quoted(values)),
        None => NOT_RECORDED.to_string(),
    };
    let verdict_text = match &verdict.reason {
        Some(reason) => format!("{} ({reason})", verdict.status),
        None => verdict.status.clone(),
    };
    let member = |value: &Option<String>| value.as_deref().unwrap_or(NOT_AVAILABLE).to_string();
    let record = format!(
        "id: {}; digest: {}; tag: {}",
        member(&verdict.record_id),
        member(&verdict.record_digest),
        member(&verdict.tag)
    );
    Ok([person.to_string(), signed_at, meaning, verdict_text, record])
}

/// Each value as a JSON string, joined by `, `, so the boundary between two
/// values stays visible whatever a value holds.
fn quoted(values: &[String]) -> String {
    values
        .iter()
        .map(|value| Value::String(flatten(value)).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// `YYYY-MM-DDTHH:MM:SSZ` as `YYYY-MM-DD HH:MM:SS UTC`; `None` for any other
/// shape.
fn format_instant(instant: &str) -> Option<String> {
    let bytes = instant.as_bytes();
    let shaped = bytes.len() == 20
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            10 => *byte == b'T',
            13 | 16 => *byte == b':',
            19 => *byte == b'Z',
            _ => byte.is_ascii_digit(),
        });
    if !shaped {
        return None;
    }
    let number = |range: std::ops::Range<usize>| instant[range].parse::<u32>().ok();
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    let valid = (1..=days).contains(&day) && hour < 24 && minute < 60 && second < 60;
    valid.then(|| format!("{} {} UTC", &instant[..10], &instant[11..19]))
}

/// Line and paragraph separators and the bidirectional controls, which can
/// split a row or visually reorder a cell.
fn is_format_control(ch: char) -> bool {
    matches!(
        ch,
        '\u{2028}' | '\u{2029}' | '\u{061C}' | '\u{200E}' | '\u{200F}'
            | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

/// The text with each control character, line break and bidirectional control
/// replaced by a space.
fn flatten(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_control() || is_format_control(ch) {
                ' '
            } else {
                ch
            }
        })
        .collect()
}

/// A cell the converter shows as literal text on one line, with a break
/// opportunity in each long unbroken run so it wraps inside its column.
fn cell(text: &str) -> String {
    breakable(&escape_markdown(&flatten(text)))
}

/// A cell can break at a zero width space, so a long run such as a record digest
/// or a URL in a meaning wraps instead of running into the next column.
const BREAK_OPPORTUNITY: char = '\u{200B}';
/// The longest unbroken run in a cell, in characters.
const MAX_RUN: usize = 16;

/// `text` with a break opportunity after every `MAX_RUN` characters of any run
/// without a space. The text is already escaped, so a break never follows a
/// backslash: it would separate an escape from the character it escapes.
fn breakable(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run = 0;
    let mut previous = ' ';
    for ch in text.chars() {
        if ch.is_whitespace() {
            run = 0;
        } else {
            if run >= MAX_RUN && previous != '\\' {
                out.push(BREAK_OPPORTUNITY);
                run = 0;
            }
            run += 1;
        }
        out.push(ch);
        previous = ch;
    }
    out
}

/// The dash count of each column: as wide as its longest cell up to
/// `MAX_COLUMN_WIDTH`, and at least 1.5 times its longest unbreakable run (a
/// run between spaces and break opportunities), whose width a proportional font
/// can exceed its character count. The counts are relative, so this keeps a
/// run from overflowing only because every run is short, which `breakable`
/// ensures for the Record cell.
fn column_widths(rows: &[[String; 5]]) -> Vec<usize> {
    (0..COLUMNS.len())
        .map(|index| {
            let cells = || {
                rows.iter()
                    .map(move |row| row[index].as_str())
                    .chain([COLUMNS[index]])
            };
            let longest = cells()
                .map(|text| text.chars().count())
                .max()
                .unwrap_or(MIN_COLUMN_WIDTH);
            let run = cells()
                .flat_map(|text| {
                    text.split(|ch: char| ch.is_whitespace() || ch == BREAK_OPPORTUNITY)
                })
                .map(|run| run.chars().count())
                .max()
                .unwrap_or(MIN_COLUMN_WIDTH);
            longest
                .clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH)
                .max(run + run / 2)
        })
        .collect()
}

/// The level-1 heading `Approval` and the pipe table, one row per entry.
fn markdown(rows: &[[String; 5]]) -> String {
    let rows: Vec<[String; 5]> = rows
        .iter()
        .map(|row| row.each_ref().map(|text| cell(text)))
        .collect();
    let widths = column_widths(&rows);
    let line = |cells: Vec<String>| format!("| {} |\n", cells.join(" | "));
    let mut out = String::from("# Approval\n\n");
    out.push_str(&line(COLUMNS.iter().map(|title| cell(title)).collect()));
    out.push_str(&line(
        widths.iter().map(|width| "-".repeat(*width)).collect(),
    ));
    for row in rows {
        out.push_str(&line(row.into_iter().collect()));
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-038-AC-4")]
    fn an_instant_is_shown_as_utc_and_any_other_shape_is_refused() {
        assert_eq!(
            format_instant("2026-01-02T03:04:05Z").as_deref(),
            Some("2026-01-02 03:04:05 UTC")
        );
        assert_eq!(format_instant("2026-01-02T03:04:05+01:00"), None);
        assert_eq!(format_instant("2026-01-02 03:04:05Z"), None);
        assert_eq!(format_instant("é026-01-02T03:04:05Z"), None);
        for bad in [
            "2026-13-45T99:99:99Z",
            "2026-02-30T00:00:00Z",
            "2025-02-29T00:00:00Z",
            "2026-00-10T00:00:00Z",
            "2026-01-02T24:00:00Z",
            "2026-01-02T03:60:00Z",
            "2026-01-02T03:04:60Z",
        ] {
            assert_eq!(format_instant(bad), None, "{bad}");
        }
        assert!(format_instant("2024-02-29T23:59:59Z").is_some());
    }

    #[test]
    #[trace("FR-038-AC-9")]
    fn a_long_run_in_any_cell_wraps_and_every_column_keeps_its_longest_run() {
        let digest = format!("sha256:{}", "ab".repeat(32));
        assert_eq!(digest.len(), 71);
        let url = "https://reviews.example.org/approvals/2026/11/a_very_long_path_*_x|y/index.html";
        let record = format!("id: REC-1; digest: {digest}; tag: approvals/v1");
        for text in [url, record.as_str()] {
            let shown = cell(text);
            assert!(shown.contains(BREAK_OPPORTUNITY), "{shown}");
            // Removing the break points leaves exactly the escaped text, and no
            // break separates an escape from its character.
            assert_eq!(shown.replace(BREAK_OPPORTUNITY, ""), escape_markdown(text));
            assert!(
                !shown.contains(&format!("\\{BREAK_OPPORTUNITY}")),
                "{shown}"
            );
        }
        let row = |meaning: &str, verdict: &str| {
            [
                "Ada".to_string(),
                "n/a".to_string(),
                cell(meaning),
                verdict.to_string(),
                cell(&record),
            ]
        };
        let widths = column_widths(&[row(url, "untrusted_signer"), row("ok", "approved")]);
        assert!(widths[3] >= "untrusted_signer".len() * 3 / 2, "{widths:?}");
        assert!(widths[2] <= MAX_COLUMN_WIDTH, "{widths:?}");
        assert!(widths[4] < widths.iter().sum::<usize>() / 2, "{widths:?}");
    }

    #[test]
    #[trace("FR-038-AC-5")]
    fn a_cell_has_no_markup_and_no_line_break() {
        let quoted_flat = quoted(&["a\nb\tc".to_string(), "d".to_string()]);
        assert_eq!(quoted_flat, "\"a b c\", \"d\"");
        let bidi = cell("a\u{202E}b\u{2066}c\u{200F}d\u{061C}e");
        assert!(bidi.chars().all(|ch| !is_format_control(ch)), "{bidi}");
        let text = cell("*x* `#pagebreak()`{=typst} | a\nb\u{2028}c");
        assert!(!text.contains('\n'));
        assert!(!text.contains('\u{2028}'));
        assert!(text.contains("\\*x\\*"));
        assert!(text.contains("\\|"));
        assert!(text.contains("\\`"));
    }
}
