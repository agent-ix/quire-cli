//! The `--repo <ORG/REPO>=<DIR>` corpus input shared by `relations` and
//! `validate --okf` (FR-030).
//!
//! Each occurrence names a member identity and its repository root. The
//! members load as one corpus (quire-rs FR-085); a member's document root is
//! `<DIR>/spec` and its reference root is `<DIR>`, the same pair
//! `validate --okf --scope` derives.

use std::path::{Path, PathBuf};

use anyhow::Context;
use quire_cli::safety;
use quire_rs::corpus::walk::load_repo;
use quire_rs::{CorpusMember, Spec};

/// One parsed `--repo` value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoArg {
    /// The declared `<org>/<repo>` identity.
    pub identity: String,
    /// The repository root as given.
    pub dir: String,
}

/// clap value parser for `<ORG/REPO>=<DIR>`; the error names the value.
pub fn parse_repo(raw: &str) -> Result<RepoArg, String> {
    let bad = || format!("'{raw}' is not <ORG/REPO>=<DIR>");
    let (identity, dir) = raw.split_once('=').ok_or_else(bad)?;
    let mut parts = identity.split('/');
    let (Some(org), Some(repo), None) = (parts.next(), parts.next(), parts.next()) else {
        return Err(bad());
    };
    if org.is_empty() || repo.is_empty() || dir.is_empty() {
        return Err(bad());
    }
    Ok(RepoArg {
        identity: identity.to_string(),
        dir: dir.to_string(),
    })
}

/// A loaded member: where its documents and references resolve.
pub struct Member {
    /// The repository root (reference root, and the root report paths are
    /// relative to).
    pub root: PathBuf,
    /// The document root, `<root>/spec`.
    pub document_root: PathBuf,
}

/// The members of a corpus and the corpus built from them.
pub struct Corpus {
    pub members: Vec<Member>,
    pub spec: Spec,
}

impl Corpus {
    /// A corpus of one member with no identity: `--scope <DIR>`.
    pub fn from_scope(scope: &Path) -> anyhow::Result<Self> {
        Self::load(vec![(None, scope.to_path_buf())])
    }

    /// A corpus of the `--repo` members, in flag order. Each root passes path
    /// safety before anything is read from it.
    pub fn from_repos(repos: &[RepoArg]) -> anyhow::Result<Self> {
        let mut roots = Vec::with_capacity(repos.len());
        for repo in repos {
            let root = safety::validate_dir_path("--repo", &repo.dir)
                .with_context(|| format!("validating --repo '{}={}'", repo.identity, repo.dir))?;
            roots.push((Some(repo.identity.clone()), root));
        }
        Self::load(roots)
    }

    fn load(roots: Vec<(Option<String>, PathBuf)>) -> anyhow::Result<Self> {
        let mut members = Vec::with_capacity(roots.len());
        let mut loaded = Vec::with_capacity(roots.len());
        for (identity, root) in roots {
            let named = identity.as_deref().unwrap_or("--scope");
            let document_root =
                super::spec_root_of(&root).with_context(|| format!("member '{named}'"))?;
            let document_root =
                safety::validate_dir_path("document root", &document_root.display().to_string())
                    .with_context(|| format!("validating document root of member '{named}'"))?;
            loaded.push(CorpusMember {
                identity: identity.clone(),
                load: load_repo(&document_root),
            });
            members.push(Member {
                root,
                document_root,
            });
        }
        let spec = Spec::from_members(loaded).map_err(|e| invalid_error!("{e}"))?;
        Ok(Self { members, spec })
    }

    /// The `(document_root, reference_root)` pair of each member, in order.
    pub fn root_pairs(&self) -> Vec<(&Path, &Path)> {
        self.members
            .iter()
            .map(|m| (m.document_root.as_path(), m.root.as_path()))
            .collect()
    }
}
