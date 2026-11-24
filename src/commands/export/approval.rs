//! `--approval <PATH>` (FR-038): the approval block generated from verdict files
//! of `engineering-assurance approval-verify`. The export verifies nothing and
//! reads no repository here; each verdict is data that is presented as it says.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::custom::{invalid, read_bounded};
use super::template::escape_markdown;

const APPROVAL_PROTOCOL: &str = "engineering-assurance.approval-verify-result/v1";
const RELEASE_PROTOCOL: &str = "engineering-assurance.release-verify-result/v1";
const APPROVED: &str = "approved";
const RELEASED: &str = "released";
const NOT_RELEASED: &str = "not_released";
const NOT_AVAILABLE: &str = "not available";
const NOT_RECORDED: &str = "not recorded";
const UNMAPPED: &str = "unmapped";
const COLUMNS: [&str; 6] = [
    "Person",
    "Signed at (signer's claim)",
    "Meaning",
    "Verdict",
    "Verdict file",
    "Record",
];
/// The start of every Verdict cell: the export verifies nothing, so no entry is
/// shown as a verified approval, whatever its status.
const UNVERIFIED: &str = "unverified claim: ";
/// The fixed statement under the `Approval` heading.
const STATEMENT: &str = "The verdicts below were computed elsewhere and are not re-verified by \
this export. An approval covers the record named in its Record column, not this document.";

/// The cells of one entry, unescaped.
type Row = [String; 6];
/// The narrowest and widest relative width a column asks for, in characters.
const MIN_COLUMN_WIDTH: usize = 3;
const MAX_COLUMN_WIDTH: usize = 28;

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
    tag_object_id: Option<String>,
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

/// A release-verify result, which may contain a nested approval verdict.
#[derive(Debug, Deserialize)]
struct ReleaseResult {
    status: String,
    reasons: Option<Value>,
    approval: Option<Value>,
}

/// An entry for the approval block: either a standalone verdict or a release result.
#[derive(Debug)]
enum Entry {
    Approval(Verdict),
    Release(ReleaseResult, Option<Box<Verdict>>),
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
        // The digest is of the bytes as read, computed in process.
        let digest = format!("sha256:{:x}", Sha256::digest(&bytes));
        let text =
            String::from_utf8(bytes).map_err(|_| invalid("--approval", path, "not valid UTF-8"))?;
        rows.push(row(&parse(path, &text)?, &digest, path)?);
    }
    Ok(Some(markdown(&rows)))
}

/// Validates a release result before any converter starts (FR-040).
fn validate_release(result: &ReleaseResult, path: &Path) -> anyhow::Result<()> {
    let name = path.display();
    if result.status != RELEASED && result.status != NOT_RELEASED {
        invalid_request!(
            "--approval '{name}': release result status must be 'released' or 'not_released', found '{}'",
            result.status
        );
    }
    if result.status == RELEASED {
        if result.approval.is_none() {
            invalid_request!("--approval '{name}': 'released' result must have 'approval' member");
        }
        // Released must not have non-empty reasons (any value other than None or empty array)
        match &result.reasons {
            None => {}
            Some(Value::Array(arr)) if arr.is_empty() => {}
            _ => {
                invalid_request!(
                    "--approval '{name}': 'released' result must not have non-empty 'reasons'"
                );
            }
        }
    }
    if result.status == NOT_RELEASED {
        match &result.reasons {
            None => {
                invalid_request!("--approval '{name}': 'not_released' result must have 'reasons'")
            }
            Some(Value::Array(arr)) => {
                if !arr.iter().all(|v| v.is_string()) {
                    invalid_request!("--approval '{name}': 'reasons' must be a list of strings");
                }
            }
            Some(_) => invalid_request!("--approval '{name}': 'reasons' must be a list of strings"),
        }
    }
    if let Some(approval) = &result.approval {
        if !approval.is_object() {
            invalid_request!("--approval '{name}': 'approval' member must be an object");
        }
        if approval.get("protocol").and_then(Value::as_str) != Some(APPROVAL_PROTOCOL) {
            invalid_request!(
                "--approval '{name}': nested 'approval' must have protocol '{APPROVAL_PROTOCOL}'"
            );
        }
        if approval.get("status").is_none() {
            invalid_request!("--approval '{name}': nested 'approval' must have 'status'");
        }
        // For released, the nested approval must have status "approved"
        if result.status == RELEASED
            && approval.get("status").and_then(Value::as_str) != Some(APPROVED)
        {
            invalid_request!(
                "--approval '{name}': 'released' result must have nested approval with status 'approved'"
            );
        }
    }
    Ok(())
}

fn parse(path: &Path, text: &str) -> anyhow::Result<Entry> {
    let name = path.display();
    let value: Value = serde_json::from_str(text)
        .map_err(|error| invalid_error!("--approval '{name}': not valid JSON: {error}"))?;

    let protocol = value.get("protocol").and_then(Value::as_str);
    match protocol {
        Some(APPROVAL_PROTOCOL) => {
            if value.get("status").is_none() {
                invalid_request!("--approval '{name}': the verdict lacks `status`");
            }
            let verdict: Verdict = serde_json::from_value(value).map_err(|error| {
                invalid_error!("--approval '{name}': unusable verdict: {error}")
            })?;
            Ok(Entry::Approval(verdict))
        }
        Some(RELEASE_PROTOCOL) => {
            let result: ReleaseResult = serde_json::from_value(value).map_err(|error| {
                invalid_error!("--approval '{name}': unusable release result: {error}")
            })?;
            validate_release(&result, path)?;

            let nested_verdict = if let Some(approval) = &result.approval {
                let verdict = Verdict::deserialize(approval).map_err(|error| {
                    invalid_error!("--approval '{name}': unusable nested approval: {error}")
                })?;
                // Validate signed_at for nested approved verdicts
                if verdict.status == APPROVED {
                    if let Some(instant) = &verdict.signed_at {
                        if format_instant(instant).is_none() {
                            invalid_request!(
                                "--approval '{name}': signed_at '{instant}' is not a UTC instant YYYY-MM-DDTHH:MM:SSZ"
                            );
                        }
                    }
                }
                Some(Box::new(verdict))
            } else {
                None
            };

            Ok(Entry::Release(result, nested_verdict))
        }
        Some(other) => {
            invalid_request!("--approval '{name}': not an {APPROVAL_PROTOCOL} or {RELEASE_PROTOCOL} document, found protocol '{other}'");
        }
        None => {
            invalid_request!(
                "--approval '{name}': not an {APPROVAL_PROTOCOL} or {RELEASE_PROTOCOL} document"
            );
        }
    }
}

/// The cells of one entry. `digest` is the `sha256:<hex>` of the file's bytes.
fn row(entry: &Entry, digest: &str, path: &Path) -> anyhow::Result<Row> {
    match entry {
        Entry::Approval(verdict) => {
            let cells = StandaloneCells::read(verdict, path)?;
            Ok(cells.row(UNVERIFIED.to_string() + &cells.verdict, digest))
        }
        Entry::Release(result, nested) => row_release(result, nested.as_deref(), digest, path),
    }
}

/// Extracts the recorded meaning from a verdict, trying verdict.meaning first, then
/// signature.signed_off_meaning.values, filtering out empty results.
fn extract_meaning(verdict: &Verdict) -> Option<&[String]> {
    verdict
        .meaning
        .as_deref()
        .or_else(|| {
            verdict
                .signature
                .as_ref()
                .and_then(|s| s.signed_off_meaning.as_ref())
                .and_then(|m| m.values.as_deref())
        })
        .filter(|values| !values.is_empty())
}

/// The members of a verdict (a standalone file or the `approval` of a release
/// result) as the entry fields of FR-038 read them.
struct StandaloneCells {
    person: String,
    signed_at: String,
    /// `claimed: ` prefixed unless the verdict is `approved`.
    meaning: String,
    /// The status text with the `reason` when present.
    verdict: String,
    record: String,
    tag_object_id: Option<String>,
}

impl StandaloneCells {
    fn read(verdict: &Verdict, path: &Path) -> anyhow::Result<Self> {
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
        let meaning = match extract_meaning(verdict) {
            Some(values) if approved => quoted(values),
            Some(values) => format!("claimed: {}", quoted(values)),
            None => NOT_RECORDED.to_string(),
        };
        let member = |value: &Option<String>| value.as_deref().unwrap_or(NOT_AVAILABLE).to_string();
        let record = format!(
            "id: {}; digest: {}; tag: {}",
            member(&verdict.record_id),
            member(&verdict.record_digest),
            member(&verdict.tag)
        );
        Ok(Self {
            person: person.to_string(),
            signed_at,
            meaning,
            verdict: status_text(verdict),
            record,
            tag_object_id: verdict.tag_object_id.clone(),
        })
    }

    /// The row of these cells, whose Record is read from the verdict and so
    /// carries the tag object the verdict names.
    fn row(&self, verdict: String, digest: &str) -> Row {
        [
            self.person.clone(),
            self.signed_at.clone(),
            self.meaning.clone(),
            verdict,
            verdict_file(digest, self.tag_object_id.as_deref()),
            self.record.clone(),
        ]
    }
}

/// `status`, or `status (reason)` when the verdict holds a `reason`.
fn status_text(verdict: &Verdict) -> String {
    match &verdict.reason {
        Some(reason) => format!("{} ({reason})", verdict.status),
        None => verdict.status.clone(),
    }
}

/// The Verdict file cell: the digest and, when known, ` tag object: <id>`.
fn verdict_file(digest: &str, tag_object_id: Option<&str>) -> String {
    match tag_object_id {
        Some(id) => format!("{digest} tag object: {id}"),
        None => digest.to_string(),
    }
}

/// The row of a release-verify result (FR-038, Release-verify input). Only the
/// nested approval of a result that is not a `not_released` with an `approved`
/// approval is a source of Person, Signed at, Meaning and Record.
fn row_release(
    result: &ReleaseResult,
    nested: Option<&Verdict>,
    digest: &str,
    path: &Path,
) -> anyhow::Result<Row> {
    let not_available = || NOT_AVAILABLE.to_string();
    if result.status == RELEASED {
        // Validation guarantees an `approved` nested verdict.
        let Some(verdict) = nested else {
            invalid_request!(
                "--approval '{}': 'released' result has no 'approval'",
                path.display()
            );
        };
        let cells = StandaloneCells::read(verdict, path)?;
        return Ok(cells.row(UNVERIFIED.to_string() + &cells.verdict, digest));
    }

    let mut segments = vec![NOT_RELEASED.to_string()];
    if let Some(Value::Array(reasons)) = &result.reasons {
        let reasons: Vec<String> = reasons
            .iter()
            .filter_map(Value::as_str)
            .map(|reason| Value::String(flatten(reason)).to_string())
            .collect();
        segments.push(format!("reasons: [{}]", reasons.join(", ")));
    }
    let verdict_cell = |segments: &[String]| format!("{UNVERIFIED}{}", segments.join("; "));

    let Some(verdict) = nested else {
        return Ok([
            not_available(),
            not_available(),
            NOT_RECORDED.to_string(),
            verdict_cell(&segments),
            verdict_file(digest, None),
            not_available(),
        ]);
    };
    if verdict.status == APPROVED {
        // Nothing is taken from an `approved` approval that the result does not
        // release, except the recorded meaning, which is a claim.
        let meaning = match extract_meaning(verdict) {
            Some(values) => format!("claimed: {}", quoted(values)),
            None => NOT_RECORDED.to_string(),
        };
        return Ok([
            not_available(),
            not_available(),
            meaning,
            verdict_cell(&segments),
            verdict_file(digest, None),
            not_available(),
        ]);
    }
    segments.push(format!("nested: {}", status_text(verdict)));
    let cells = StandaloneCells::read(verdict, path)?;
    Ok(cells.row(verdict_cell(&segments), digest))
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
fn column_widths(rows: &[Row]) -> Vec<usize> {
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
fn markdown(rows: &[Row]) -> String {
    let rows: Vec<Row> = rows
        .iter()
        .map(|row| row.each_ref().map(|text| cell(text)))
        .collect();
    let widths = column_widths(&rows);
    let line = |cells: Vec<String>| format!("| {} |\n", cells.join(" | "));
    let mut out = format!("# Approval\n\n{STATEMENT}\n\n");
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
                cell(&format!("sha256:{}", "cd".repeat(32))),
                cell(&record),
            ]
        };
        let widths = column_widths(&[row(url, "untrusted_signer"), row("ok", "approved")]);
        assert!(widths[3] >= "untrusted_signer".len() * 3 / 2, "{widths:?}");
        assert!(widths[2] <= MAX_COLUMN_WIDTH, "{widths:?}");
        assert!(widths[5] < widths.iter().sum::<usize>() / 2, "{widths:?}");
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
