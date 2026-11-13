//! Document identity for an export (FR-034): the frontmatter fields of the
//! input plus the commit hash and committer date of the source revision.
//!
//! The commit, tag, blob and attributes are read in process from the git
//! object store with read-only access; no `git` process is started and the
//! wall clock is never consulted.

use std::fs;
use std::path::{Path, PathBuf};

use bstr::{BStr, ByteSlice};
use gix_hash::ObjectId;
use gix_object::{Find, ObjectRef};
use gix_ref::file::ReferenceExt;
use serde_json::{Map, Value};

use crate::commands::failure;

/// The source-selection flags of FR-034.
pub(super) struct SourceOptions<'a> {
    pub(super) repo: Option<&'a Path>,
    pub(super) rev: Option<&'a str>,
    pub(super) release: Option<&'a str>,
}

/// The six identity fields of FR-034.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Identity {
    pub(super) title: String,
    pub(super) document_id: String,
    pub(super) version: Option<String>,
    pub(super) release: Option<String>,
    pub(super) commit: String,
    pub(super) date: String,
    /// The committer time as seconds since the Unix epoch.
    pub(super) epoch: i64,
}

/// The identity and the committed body of one input.
pub(super) struct Source {
    pub(super) identity: Identity,
    /// The committed blob after its frontmatter, the converter input.
    pub(super) body: String,
    /// The whole committed blob, the text that is validated.
    pub(super) committed: String,
    /// The shared git directory of the source repository.
    pub(super) repository: PathBuf,
}

/// Reads the committed revision of `input` and assembles its identity.
/// `working` is the text of the working file, compared with the committed blob.
pub(super) fn resolve(
    input: &Path,
    working: &str,
    options: &SourceOptions<'_>,
) -> anyhow::Result<Source> {
    let repo = Repo::open(input, options.repo)?;
    let commit = repo.resolve_commit(options.rev.unwrap_or("HEAD"))?;
    if let Some(tag) = options.release {
        repo.check_release(tag, &commit.id)?;
    }
    let blob = repo.blob_at(&commit, input)?;
    let name = input.display();
    if let Some(filter) = repo.filter_attribute(&commit, &repo.relative)? {
        return Err(failure::invalid(anyhow::anyhow!(
            "'{name}' has the git filter attribute 'filter={filter}'; the committed bytes are not \
             the bytes of the file, so it cannot be exported"
        )));
    }
    let committed = String::from_utf8(blob).map_err(|_| {
        failure::invalid(anyhow::anyhow!("the committed '{name}' is not valid UTF-8"))
    })?;
    if committed.replace("\r\n", "\n") != working.replace("\r\n", "\n") {
        return Err(failure::invalid(anyhow::anyhow!(
            "'{name}' has changes that are not committed at {}; commit it or pass an older \
             --source-rev",
            commit.id
        )));
    }

    let parsed = quire_rs::extract_frontmatter(&committed);
    let frontmatter = parsed.frontmatter.as_ref();
    let title = text_member(frontmatter, "title");
    let document_id = text_member(frontmatter, "id");
    let version = text_member(frontmatter, "version");
    let release = options.release.map(str::to_string);
    let (Some(title), Some(document_id), true) = (
        title.clone(),
        document_id.clone(),
        version.is_some() || release.is_some(),
    ) else {
        let mut missing = Vec::new();
        if title.is_none() {
            missing.push("title");
        }
        if document_id.is_none() {
            missing.push("id");
        }
        if version.is_none() && release.is_none() {
            missing.push("version (or pass --release)");
        }
        return Err(failure::invalid(anyhow::anyhow!(
            "'{name}' lacks the identity field(s): {}",
            missing.join(", ")
        )));
    };
    Ok(Source {
        identity: Identity {
            title,
            document_id,
            version,
            release,
            commit: commit.id.to_string(),
            date: utc_date(commit.seconds),
            epoch: commit.seconds,
        },
        body: parsed.body.to_string(),
        committed,
        repository: repo.common.clone(),
    })
}

/// A non-empty frontmatter member written as a string or a number.
fn text_member(frontmatter: Option<&Map<String, Value>>, key: &str) -> Option<String> {
    let text = match frontmatter?.get(key)? {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        _ => return None,
    };
    (!text.trim().is_empty()).then_some(text)
}

/// The committer date as `YYYY-MM-DD` in UTC (civil-from-days).
fn utc_date(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

struct CommitInfo {
    id: ObjectId,
    tree: ObjectId,
    seconds: i64,
}

struct Repo {
    dir: PathBuf,
    odb: gix_odb::Handle,
    refs: gix_ref::file::Store,
    /// The input path relative to the work tree, `/`-separated.
    relative: String,
    /// The shared repository directory (objects, refs, config, info).
    common: PathBuf,
}

impl Repo {
    fn open(input: &Path, explicit: Option<&Path>) -> anyhow::Result<Self> {
        let input = fs::canonicalize(input)
            .map_err(|error| failure::invalid(anyhow::anyhow!("'{}': {error}", input.display())))?;
        let start = match explicit {
            Some(dir) => dir.to_path_buf(),
            None => input.parent().map(Path::to_path_buf).unwrap_or_default(),
        };
        if explicit.is_some()
            && !start.join(".git").exists()
            && gix_discover::is_git(&start).is_err()
        {
            return Err(failure::invalid(anyhow::anyhow!(
                "source commit unavailable: --source-repo '{}' is not a git repository",
                start.display()
            )));
        }
        let (found, _trust) = gix_discover::upwards(&start).map_err(|error| {
            failure::invalid(anyhow::anyhow!(
                "source commit unavailable: no git repository contains '{}' (looked at '{}'): {error}",
                input.display(),
                start.display()
            ))
        })?;
        let (git_dir, work_tree) = found.into_repository_and_work_tree_directories();
        let git_dir = fs::canonicalize(&git_dir).unwrap_or(git_dir);
        let Some(work_tree) = work_tree.and_then(|dir| fs::canonicalize(dir).ok()) else {
            return Err(failure::invalid(anyhow::anyhow!(
                "source commit unavailable: '{}' is a bare repository with no work tree",
                git_dir.display()
            )));
        };
        let relative = input.strip_prefix(&work_tree).map_err(|_| {
            failure::invalid(anyhow::anyhow!(
                "source commit unavailable: '{}' is outside the repository '{}'",
                input.display(),
                work_tree.display()
            ))
        })?;
        let relative = relative
            .components()
            .map(|part| part.as_os_str().to_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                failure::invalid(anyhow::anyhow!("'{}' is not a UTF-8 path", input.display()))
            })?
            .join("/");
        let common = common_dir(&git_dir);
        let unreadable = |what: &str, error: String| {
            failure::invalid(anyhow::anyhow!(
                "source commit unavailable: the object store of '{}' is unreadable ({what}: {error})",
                git_dir.display()
            ))
        };
        let odb = gix_odb::at(common.join("objects"), gix_hash::Kind::Sha1)
            .map_err(|error| unreadable("objects", error.to_string()))?;
        let refs = if common == git_dir {
            gix_ref::file::Store::at(git_dir.clone(), gix_hash::Kind::Sha1)
        } else {
            gix_ref::file::Store::for_linked_worktree(
                git_dir.clone(),
                common.clone(),
                gix_hash::Kind::Sha1,
            )
        };
        Ok(Self {
            dir: git_dir,
            odb,
            refs,
            relative,
            common,
        })
    }

    fn unavailable(&self, what: &str, cause: impl std::fmt::Display) -> anyhow::Error {
        failure::invalid(anyhow::anyhow!(
            "source commit unavailable in '{}': {what}: {cause}",
            self.dir.display()
        ))
    }

    /// An object peeled through any tag objects to its commit.
    fn resolve_commit(&self, rev: &str) -> anyhow::Result<CommitInfo> {
        let mut id = self.resolve_rev(rev)?;
        let mut buffer = Vec::new();
        loop {
            let data = self
                .odb
                .try_find(&id, &mut buffer)
                .map_err(|error| self.unavailable(rev, error))?
                .ok_or_else(|| self.unavailable(rev, format!("object {id} is missing")))?;
            let decoded = data
                .decode()
                .map_err(|error| self.unavailable(rev, error))?;
            match decoded {
                ObjectRef::Commit(commit) => {
                    let time = commit
                        .time()
                        .map_err(|error| self.unavailable(rev, error))?;
                    return Ok(CommitInfo {
                        id,
                        tree: commit.tree(),
                        seconds: time.seconds,
                    });
                }
                ObjectRef::Tag(tag) => id = tag.target(),
                _ => return Err(self.unavailable(rev, "it does not name a commit")),
            }
        }
    }

    /// A full object hash, or a reference name resolved as git does.
    fn resolve_rev(&self, rev: &str) -> anyhow::Result<ObjectId> {
        if rev.len() == 40 && rev.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return ObjectId::from_hex(rev.as_bytes())
                .map_err(|error| self.unavailable(rev, error));
        }
        let mut reference = self
            .refs
            .try_find(rev)
            .map_err(|error| self.unavailable(rev, error))?
            .ok_or_else(|| self.unavailable(rev, "it does not resolve (only a full 40-digit hash or a reference name is accepted)"))?;
        reference
            .peel_to_id(&self.refs, &self.odb)
            .map_err(|error| self.unavailable(rev, error))
    }

    /// `--release` must be a tag that resolves to the source commit.
    fn check_release(&self, tag: &str, commit: &ObjectId) -> anyhow::Result<()> {
        let name = format!("refs/tags/{tag}");
        let mut reference = self
            .refs
            .try_find(name.as_str())
            .map_err(|error| self.unavailable(tag, error))?
            .ok_or_else(|| self.unavailable(tag, "the release tag does not resolve"))?;
        let id = reference
            .peel_to_id(&self.refs, &self.odb)
            .map_err(|error| self.unavailable(tag, error))?;
        let tagged = self.resolve_commit(&id.to_string())?;
        if tagged.id != *commit {
            return Err(self.unavailable(
                tag,
                format!(
                    "the release tag names {}, not the source commit {commit}",
                    tagged.id
                ),
            ));
        }
        Ok(())
    }

    /// The tree entry for `path` below `tree`, as its blob id.
    fn lookup(&self, tree: &ObjectId, path: &str) -> anyhow::Result<Option<(ObjectId, bool)>> {
        let mut current = *tree;
        let parts: Vec<&str> = path.split('/').collect();
        for (index, part) in parts.iter().enumerate() {
            let mut buffer = Vec::new();
            let data = self
                .odb
                .try_find(&current, &mut buffer)
                .map_err(|error| self.unavailable(path, error))?
                .ok_or_else(|| self.unavailable(path, format!("object {current} is missing")))?;
            let ObjectRef::Tree(entries) = data
                .decode()
                .map_err(|error| self.unavailable(path, error))?
            else {
                return Ok(None);
            };
            let Some(entry) = entries
                .entries
                .iter()
                .find(|entry| entry.filename == BStr::new(part.as_bytes()))
            else {
                return Ok(None);
            };
            let last = index + 1 == parts.len();
            if last {
                return Ok(Some((entry.oid.to_owned(), entry.mode.is_blob())));
            }
            if !entry.mode.is_tree() {
                return Ok(None);
            }
            current = entry.oid.to_owned();
        }
        Ok(None)
    }

    fn blob_at(&self, commit: &CommitInfo, input: &Path) -> anyhow::Result<Vec<u8>> {
        let missing = || {
            failure::invalid(anyhow::anyhow!(
                "'{}' is not committed at {}; commit it or pass an older --source-rev",
                input.display(),
                commit.id
            ))
        };
        let (id, is_blob) = self
            .lookup(&commit.tree, &self.relative)?
            .ok_or_else(missing)?;
        if !is_blob {
            return Err(missing());
        }
        let mut buffer = Vec::new();
        let data = self
            .odb
            .try_find(&id, &mut buffer)
            .map_err(|error| self.unavailable(&self.relative, error))?
            .ok_or_else(|| self.unavailable(&self.relative, format!("blob {id} is missing")))?;
        Ok(data.data.to_vec())
    }

    /// The `filter` attribute value that git applies to `path` at `commit`:
    /// the committed `.gitattributes` files from the root down, then the
    /// repository's `info/attributes`, which has the highest precedence. Deeper
    /// files and later lines win. Attribute macros are expanded.
    fn filter_attribute(&self, commit: &CommitInfo, path: &str) -> anyhow::Result<Option<String>> {
        let parts: Vec<&str> = path.split('/').collect();
        // (bytes, path below the file's directory)
        let mut files: Vec<(Vec<u8>, String)> = Vec::new();
        for depth in 0..parts.len() {
            let directory = parts[..depth].join("/");
            let attributes = if directory.is_empty() {
                ".gitattributes".to_string()
            } else {
                format!("{directory}/.gitattributes")
            };
            let Some((id, true)) = self.lookup(&commit.tree, &attributes)? else {
                continue;
            };
            let mut buffer = Vec::new();
            let Some(data) = self
                .odb
                .try_find(&id, &mut buffer)
                .map_err(|error| self.unavailable(&attributes, error))?
            else {
                continue;
            };
            files.push((data.data.to_vec(), parts[depth..].join("/")));
        }
        let info = self.common.join("info/attributes");
        if let Ok(bytes) = fs::read(&info) {
            files.push((bytes, path.to_string()));
        }
        let mut state = Attributes::default();
        // Macros are honoured from the top-level file and `info/attributes` only.
        if let Some((bytes, _)) = files.iter().find(|(_, relative)| relative == path) {
            state.define_macros(bytes);
        }
        if let Some((bytes, _)) = files.last().filter(|_| info.is_file()) {
            state.define_macros(bytes);
        }
        for (bytes, relative) in &files {
            state.apply(bytes, relative);
        }
        Ok(state.filter)
    }
}

/// An attribute state, owned.
#[derive(Clone)]
enum State {
    Set,
    Unset,
    Unspecified,
    Value(String),
}

impl State {
    fn from_ref(state: gix_attributes::StateRef<'_>) -> Self {
        match state {
            gix_attributes::StateRef::Set => Self::Set,
            gix_attributes::StateRef::Unset => Self::Unset,
            gix_attributes::StateRef::Unspecified => Self::Unspecified,
            gix_attributes::StateRef::Value(value) => Self::Value(value.as_bstr().to_string()),
        }
    }
}

/// The attribute evaluation that matters to an export: the `filter` attribute
/// and the macros that can set it.
#[derive(Default)]
struct Attributes {
    macros: std::collections::HashMap<String, Vec<(String, State)>>,
    filter: Option<String>,
}

/// A macro may expand to another; a chain longer than this is not followed.
const MACRO_DEPTH: usize = 8;

impl Attributes {
    fn define_macros(&mut self, bytes: &[u8]) {
        for line in gix_attributes::parse(bytes).flatten() {
            let (gix_attributes::parse::Kind::Macro(name), assignments, _) = line else {
                continue;
            };
            let body = assignments
                .flatten()
                .map(|assignment| {
                    (
                        assignment.name.as_str().to_string(),
                        State::from_ref(assignment.state),
                    )
                })
                .collect();
            self.macros.insert(name.as_str().to_string(), body);
        }
    }

    /// Applies one attributes file to `relative` (the path below the file's
    /// directory).
    fn apply(&mut self, bytes: &[u8], relative: &str) {
        let basename_start = relative.rfind('/').map(|position| position + 1);
        for line in gix_attributes::parse(bytes).flatten() {
            let (gix_attributes::parse::Kind::Pattern(pattern), assignments, _) = line else {
                continue;
            };
            let matched = pattern.matches_repo_relative_path(
                relative.as_bytes().as_bstr(),
                basename_start,
                Some(false),
                gix_glob::pattern::Case::Sensitive,
                gix_glob::wildmatch::Mode::NO_MATCH_SLASH_LITERAL,
            );
            if !matched {
                continue;
            }
            for assignment in assignments.flatten() {
                let state = State::from_ref(assignment.state);
                self.assign(assignment.name.as_str(), &state, 0);
            }
        }
    }

    fn assign(&mut self, name: &str, state: &State, depth: usize) {
        if name == "filter" {
            self.filter = match state {
                State::Value(value) => Some(value.clone()),
                _ => None,
            };
        } else if matches!(state, State::Set) && depth < MACRO_DEPTH {
            if let Some(body) = self.macros.get(name).cloned() {
                for (inner, inner_state) in &body {
                    self.assign(inner, inner_state, depth + 1);
                }
            }
        }
    }
}

/// The directory that holds the shared repository state (objects, refs,
/// config, info): the `commondir` of a linked work tree's git dir, otherwise
/// the git dir itself.
fn common_dir(git_dir: &Path) -> PathBuf {
    match fs::read_to_string(git_dir.join("commondir")) {
        Ok(text) => {
            let relative = git_dir.join(text.trim());
            fs::canonicalize(&relative).unwrap_or(relative)
        }
        Err(_) => git_dir.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-034-AC-3")]
    fn the_date_is_the_utc_calendar_day() {
        // 2001-02-03T23:30:00-05:00 is 2001-02-04T04:30:00Z.
        assert_eq!(utc_date(981_261_000), "2001-02-04");
        assert_eq!(utc_date(0), "1970-01-01");
        assert_eq!(utc_date(951_782_400), "2000-02-29");
    }

    #[test]
    #[trace("FR-034-AC-11")]
    fn a_filter_attribute_is_found_by_pattern() {
        let filter_of = |files: &[(&str, &str)], path: &str| {
            let mut attributes = Attributes::default();
            for (bytes, _) in files {
                attributes.define_macros(bytes.as_bytes());
            }
            for (bytes, relative) in files {
                let relative = if relative.is_empty() { path } else { relative };
                attributes.apply(bytes.as_bytes(), relative);
            }
            attributes.filter
        };
        assert_eq!(
            filter_of(&[("*.md filter=lfs diff=lfs\n", "")], "docs/a.md").as_deref(),
            Some("lfs")
        );
        assert_eq!(filter_of(&[("*.txt filter=lfs\n", "")], "a.md"), None);
        assert_eq!(
            filter_of(&[("*.md filter=lfs\n", ""), ("a.md -filter\n", "")], "a.md"),
            None
        );
        // A macro that sets the filter is expanded.
        assert_eq!(
            filter_of(
                &[("[attr]mylfs filter=lfs -text\na.md mylfs\n", "")],
                "a.md"
            )
            .as_deref(),
            Some("lfs")
        );
        assert_eq!(
            filter_of(&[("[attr]mylfs filter=lfs\nb.md mylfs\n", "")], "a.md"),
            None
        );
    }
}
