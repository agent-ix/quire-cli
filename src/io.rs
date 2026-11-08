//! CLI-specific input and output placement around ix-cli-kit's stream contract.

use std::io::Read;
use std::path::Path;

use ix_cli_kit::streams::{Diagnostics, Record};

/// Read a `--data` argument: a filesystem path or `-` for stdin.
///
/// JSON parse errors surface as an invalid-request outcome at the caller.
pub fn read_data(arg: &str) -> anyhow::Result<serde_json::Value> {
    let bytes = if arg == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    } else {
        std::fs::read(Path::new(arg))?
    };
    Ok(serde_json::from_slice(&bytes)?)
}

/// Read a `<doc|->` positional argument as a UTF-8 string.
pub fn read_text(arg: &str) -> anyhow::Result<String> {
    if arg == "-" {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        Ok(text)
    } else {
        Ok(std::fs::read_to_string(Path::new(arg))?)
    }
}

/// Write primary subcommand output either to stdout or to `--out`.
pub fn write_primary(out_path: Option<&Path>, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(path) = out_path {
        std::fs::write(path, bytes)
    } else {
        ix_cli_kit::streams::write_primary_stdout(bytes)
    }
}

/// Emit engine diagnostics through the shared record renderer.
pub fn emit_quire_diagnostics<'a, I>(out: Diagnostics, iter: I)
where
    I: IntoIterator<Item = &'a quire_rs::Diagnostic>,
{
    for diagnostic in iter {
        let json = diagnostic.to_json();
        let kind = json
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Diagnostic");
        let message = diagnostic.to_string();
        Record::error(kind, &message).emit(out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_data_from_file() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "{{\"x\": 1}}").unwrap();
        let value = read_data(file.path().to_str().unwrap()).unwrap();
        assert_eq!(value["x"], serde_json::json!(1));
    }
}
