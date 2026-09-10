//! AST-backed unsafe-block documentation policy.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::{source, walk, Error, Result};

const BASELINE: &str = "scripts/unsafe_comment_baseline.txt";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Locus {
    path: PathBuf,
    line: usize,
}

impl Locus {
    fn encode(&self) -> String {
        format!("{}:{}", self.path.display(), self.line)
    }
}

fn undocumented_loci(root: &Path) -> Result<BTreeSet<Locus>> {
    let mut loci = BTreeSet::new();
    for path in walk::rust_files(
        root,
        &["src", "tests", "tools"],
        &["fixtures", "target", "node_modules"],
    )? {
        let source = fs::read_to_string(&path).map_err(|source| Error::Read {
            path: path.clone(),
            source,
        })?;
        let comments =
            source::safety_comment_lines(&source).map_err(|source| Error::ParseRust {
                path: path.clone(),
                source,
            })?;
        for line in source::unsafe_lines(&source).map_err(|source| Error::ParseRust {
            path: path.clone(),
            source,
        })? {
            let preceding_start = line.saturating_sub(3);
            let documented = comments.range(preceding_start..line).next().is_some();
            if !documented {
                loci.insert(Locus {
                    path: path.strip_prefix(root).unwrap_or(&path).to_path_buf(),
                    line,
                });
            }
        }
    }
    Ok(loci)
}

fn read_baseline(root: &Path) -> Result<BTreeSet<String>> {
    let path = root.join(BASELINE);
    if !baseline_exists_and_is_regular(&path)? {
        return Ok(BTreeSet::new());
    }
    let text = fs::read_to_string(&path).map_err(|source| Error::Read {
        path: path.clone(),
        source,
    })?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn baseline_exists_and_is_regular(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(Error::SymlinkBaseline {
            path: path.to_path_buf(),
        }),
        Ok(_) => Ok(true),
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(false),
        Err(source) => Err(Error::InspectPath {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Enforce exact unsafe-comment and baseline-lifecycle behavior.
pub fn check(root: &Path) -> Result<()> {
    let missing = undocumented_loci(root)?;
    let missing_encoded: BTreeSet<_> = missing.iter().map(Locus::encode).collect();
    let baseline = read_baseline(root)?;

    let unreviewed: Vec<_> = missing_encoded.difference(&baseline).cloned().collect();
    let stale: Vec<_> = baseline.difference(&missing_encoded).cloned().collect();
    if !unreviewed.is_empty() || !stale.is_empty() {
        return Err(Error::UnsafeBaseline { unreviewed, stale });
    }
    Ok(())
}

/// Explicitly replace the baseline with the current undocumented loci.
pub fn update_baseline(root: &Path) -> Result<usize> {
    let missing = undocumented_loci(root)?;
    let mut rendered = missing
        .iter()
        .map(Locus::encode)
        .collect::<Vec<_>>()
        .join("\n");
    if !rendered.is_empty() {
        rendered.push('\n');
    }
    let path = root.join(BASELINE);
    baseline_exists_and_is_regular(&path)?;
    fs::write(&path, rendered).map_err(|source| Error::Write {
        path: path.clone(),
        source,
    })?;
    Ok(missing.len())
}
