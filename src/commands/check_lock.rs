//! Read-only Cargo Git source coherence policy (FR-027).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use url::Url;

use super::{failure, Ctx};

#[derive(Debug, Parser)]
pub struct Args {
    /// Explicit Cargo.lock paths (no stdin or glob expansion).
    #[arg(required = true, num_args = 1.., value_name = "Cargo.lock")]
    pub locks: Vec<String>,
}

// Cargo owns the rest of the lock schema. Unknown fields (versions, checksums,
// metadata and dependency lists) are intentionally outside this policy.
#[derive(Deserialize)]
struct Lock {
    #[serde(default)]
    package: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Entry {
    repository: String,
    package: String,
    revision: String,
}

#[derive(Debug)]
struct Violations(Vec<(String, BTreeSet<Entry>)>);

impl fmt::Display for Violations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "agent-ix Git coherence refused (duplicate package name or split repository revision)")?;
        for (path, entries) in &self.0 {
            for entry in entries {
                writeln!(
                    f,
                    "{path}: {} package {} revision {}",
                    entry.repository, entry.package, entry.revision
                )?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Violations {}

fn agent_source(source: &str) -> anyhow::Result<Option<(String, String)>> {
    if source.starts_with("registry+") || source.starts_with("sparse+") {
        return Ok(None);
    }
    let raw = source
        .strip_prefix("git+")
        .ok_or_else(|| failure::invalid(anyhow::anyhow!("unrecognized Cargo source {source:?}")))?;
    let url = Url::parse(raw).map_err(failure::invalid)?;
    let default_port = match url.scheme() {
        "https" => 443,
        "http" => 80,
        "ssh" => 22,
        "git" => 9418,
        _ => invalid_request!("unsupported Git URL {source:?}"),
    };
    if url.host_str().is_none() {
        invalid_request!("malformed Git URL {source:?}");
    }
    let path = url.path();
    let lower_path = path.to_ascii_lowercase();
    if lower_path.contains("%2f") || lower_path.contains("%5c") {
        invalid_request!("encoded path separator in Git URL {source:?}");
    }
    let components: Vec<_> = path.trim_start_matches('/').split('/').collect();
    let is_agent = url
        .host_str()
        .is_some_and(|host| host.eq_ignore_ascii_case("github.com"))
        && components
            .first()
            .is_some_and(|owner| owner.eq_ignore_ascii_case("agent-ix"));
    if !is_agent {
        return Ok(None);
    }
    let repository_path = path.trim_start_matches('/').strip_suffix('/').unwrap_or(path.trim_start_matches('/'));
    let parts: Vec<_> = repository_path.split('/').collect();
    if url.port().is_some_and(|port| port != default_port)
        || parts.len() != 2
        || parts[1].is_empty()
    {
        invalid_request!("invalid agent-ix repository URL {source:?}");
    }
    let name = parts[1].to_ascii_lowercase();
    let name = name.strip_suffix(".git").unwrap_or(&name);
    if name.is_empty() {
        invalid_request!("empty agent-ix repository in {source:?}");
    }
    let revision = url.fragment().unwrap_or_default();
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        invalid_request!("agent-ix source requires a 40-digit resolved commit fragment: {source:?}");
    }
    Ok(Some((
        format!("github.com/agent-ix/{name}"),
        revision.to_ascii_lowercase(),
    )))
}

fn check(packages: Vec<Package>) -> anyhow::Result<BTreeSet<Entry>> {
    let mut by_name: BTreeMap<String, Vec<Entry>> = BTreeMap::new();
    let mut by_repository: BTreeMap<String, Vec<Entry>> = BTreeMap::new();
    for package in packages {
        let Some(source) = package.source else {
            continue;
        };
        let Some((repository, revision)) = agent_source(&source)? else {
            continue;
        };
        let entry = Entry {
            repository,
            package: package.name,
            revision,
        };
        by_name.entry(entry.package.clone()).or_default().push(entry.clone());
        by_repository.entry(entry.repository.clone()).or_default().push(entry);
    }
    let mut failures = BTreeSet::new();
    for entries in by_name.values() {
        if entries.len() > 1 {
            failures.extend(entries.iter().cloned());
        }
    }
    for entries in by_repository.values() {
        if entries.iter().map(|entry| &entry.revision).collect::<BTreeSet<_>>().len() > 1 {
            failures.extend(entries.iter().cloned());
        }
    }
    Ok(failures)
}

pub fn run(_ctx: &Ctx, args: Args) -> anyhow::Result<()> {
    let mut failures = Vec::new();
    for raw in &args.locks {
        if raw == "-" {
            invalid_request!("Cargo.lock path '-': check-lock does not read stdin");
        }
        let path = quire_cli::safety::validate_input_path("Cargo.lock", raw)?;
        let text = std::fs::read_to_string(&path)
            .map_err(|error| failure::input(error.into()))
            .with_context(|| format!("Cargo.lock '{raw}'"))?;
        let lock: Lock = toml::from_str(&text)
            .map_err(failure::invalid)
            .with_context(|| format!("Cargo.lock '{raw}'"))?;
        let entries = check(lock.package).with_context(|| format!("Cargo.lock '{raw}'"))?;
        if !entries.is_empty() {
            failures.push((raw.clone(), entries));
        }
    }
    if !failures.is_empty() {
        failures.sort_by(|left, right| left.0.cmp(&right.0));
        return Err(failure::refused(Violations(failures)));
    }
    primary_line!("Git lock coherence checked: {}", args.locks.join(", "));
    Ok(())
}
