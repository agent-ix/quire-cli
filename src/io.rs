//! Stdin / stdout / stderr wiring (FR-006, FR-007, FR-008).
//!
//! - Subcommand primary output goes to stdout; nothing else does.
//! - Diagnostics + errors go to stderr.
//! - JSON output is compact by default; `--pretty` switches to indented.
//! - `--diagnostics-format=human|json` selects the stderr encoding.

use std::io::{Read, Write};
use std::path::Path;

use serde::Serialize;

/// Format selector for stderr diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagnosticsFormat {
    #[default]
    Human,
    Json,
}

impl std::str::FromStr for DiagnosticsFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            other => Err(format!("unknown diagnostics format: '{other}'")),
        }
    }
}

/// Read a `--data` argument: a filesystem path or `-` for stdin.
///
/// Returns the parsed `serde_json::Value`. JSON parse errors surface as
/// a user error (exit 1) at the caller.
pub fn read_data(arg: &str) -> anyhow::Result<serde_json::Value> {
    let bytes = if arg == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    } else {
        std::fs::read(Path::new(arg))?
    };
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    Ok(value)
}

/// Read a `<doc|->` positional argument as a UTF-8 string.
pub fn read_text(arg: &str) -> anyhow::Result<String> {
    if arg == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        Ok(std::fs::read_to_string(Path::new(arg))?)
    }
}

/// Write the primary subcommand output to stdout.
pub fn write_primary_stdout(bytes: &[u8]) -> std::io::Result<()> {
    let mut out = std::io::stdout().lock();
    out.write_all(bytes)?;
    out.flush()
}

/// Write the primary subcommand output either to stdout or to `--out`.
pub fn write_primary(out_path: Option<&Path>, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(p) = out_path {
        std::fs::write(p, bytes)?;
        Ok(())
    } else {
        write_primary_stdout(bytes)
    }
}

/// Encode a `Serialize` value as JSON, respecting `pretty`.
pub fn encode_json<T: Serialize>(value: &T, pretty: bool) -> serde_json::Result<String> {
    if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
}

/// Write a single diagnostic line to stderr.
pub fn write_diagnostic_human(msg: &str) {
    eprintln!("{msg}");
}

/// Write a diagnostic as a JSON line on stderr.
pub fn write_diagnostic_json(kind: &str, message: &str) {
    let line = serde_json::json!({
        "kind": kind,
        "message": message,
    });
    eprintln!("{line}");
}

/// Emit a diagnostic according to the configured format.
pub fn emit_diagnostic(format: DiagnosticsFormat, kind: &str, message: &str) {
    match format {
        DiagnosticsFormat::Human => write_diagnostic_human(message),
        DiagnosticsFormat::Json => write_diagnostic_json(kind, message),
    }
}

/// Emit every diagnostic in `iter` according to the configured format.
pub fn emit_quire_diagnostics<'a, I>(format: DiagnosticsFormat, iter: I)
where
    I: IntoIterator<Item = &'a quire_rs::Diagnostic>,
{
    for d in iter {
        let json = d.to_json();
        let kind = json
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("Diagnostic");
        emit_diagnostic(format, kind, &d.to_string());
    }
}

/// Exit codes per FR-007. Anything else is a bug.
pub mod exit {
    pub const OK: i32 = 0;
    pub const USER_ERROR: i32 = 1;
    pub const ARGV_ERROR: i32 = 2;
    // 134 is reserved for SIGABRT panics; we don't emit it ourselves.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_format_parses() {
        use std::str::FromStr;
        assert_eq!(
            DiagnosticsFormat::from_str("human").unwrap(),
            DiagnosticsFormat::Human
        );
        assert_eq!(
            DiagnosticsFormat::from_str("json").unwrap(),
            DiagnosticsFormat::Json
        );
        assert!(DiagnosticsFormat::from_str("yaml").is_err());
    }

    #[test]
    fn encode_json_compact_default() {
        let v = serde_json::json!({"a": 1, "b": 2});
        let s = encode_json(&v, false).unwrap();
        assert!(!s.contains('\n'));
    }

    #[test]
    fn encode_json_pretty_indents() {
        let v = serde_json::json!({"a": 1});
        let s = encode_json(&v, true).unwrap();
        assert!(s.contains('\n'));
    }

    #[test]
    fn read_data_from_file() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "{{\"x\": 1}}").unwrap();
        let v = read_data(f.path().to_str().unwrap()).unwrap();
        assert_eq!(v["x"], serde_json::json!(1));
    }
}
