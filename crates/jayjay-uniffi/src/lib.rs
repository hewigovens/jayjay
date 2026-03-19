uniffi::setup_scaffolding!();

use std::path::PathBuf;
use std::sync::Arc;

use jayjay_core::{self as core, CoreError};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum JayJayError {
    #[error("repository not found: {path}")]
    RepoNotFound { path: String },
    #[error("revision not found: {rev}")]
    RevNotFound { rev: String },
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl From<CoreError> for JayJayError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::RepoNotFound { path } => Self::RepoNotFound { path },
            CoreError::RevNotFound { rev } => Self::RevNotFound { rev },
            CoreError::Internal { message } => Self::Internal { message },
        }
    }
}

#[derive(uniffi::Record)]
pub struct ChangeInfo {
    pub change_id: String,
    pub commit_id: String,
    pub description: String,
    pub author: String,
    pub email: String,
    pub timestamp_millis: i64,
    pub parents: Vec<String>,
    pub bookmarks: Vec<String>,
    pub is_working_copy: bool,
    pub has_conflict: bool,
    pub is_empty: bool,
}

impl From<core::ChangeInfo> for ChangeInfo {
    fn from(c: core::ChangeInfo) -> Self {
        Self {
            change_id: c.change_id,
            commit_id: c.commit_id,
            description: c.description,
            author: c.author,
            email: c.email,
            timestamp_millis: c.timestamp_millis,
            parents: c.parents,
            bookmarks: c.bookmarks,
            is_working_copy: c.is_working_copy,
            has_conflict: c.has_conflict,
            is_empty: c.is_empty,
        }
    }
}

#[derive(uniffi::Record)]
pub struct GraphEntry {
    pub change: ChangeInfo,
    pub edges: Vec<GraphEdge>,
}

impl From<core::GraphEntry> for GraphEntry {
    fn from(g: core::GraphEntry) -> Self {
        Self {
            change: g.change.into(),
            edges: g.edges.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct GraphEdge {
    pub target: String,
    pub edge_type: EdgeType,
}

impl From<core::GraphEdge> for GraphEdge {
    fn from(e: core::GraphEdge) -> Self {
        Self {
            target: e.target,
            edge_type: e.edge_type.into(),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum EdgeType {
    Direct,
    Indirect,
    Missing,
}

impl From<core::EdgeType> for EdgeType {
    fn from(e: core::EdgeType) -> Self {
        match e {
            core::EdgeType::Direct => Self::Direct,
            core::EdgeType::Indirect => Self::Indirect,
            core::EdgeType::Missing => Self::Missing,
        }
    }
}

#[derive(uniffi::Record)]
pub struct DiffHunk {
    pub path: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub hunk_type: HunkType,
}

impl From<core::DiffHunk> for DiffHunk {
    fn from(h: core::DiffHunk) -> Self {
        Self {
            path: h.path.display().to_string(),
            old_content: h.old_content,
            new_content: h.new_content,
            hunk_type: h.hunk_type.into(),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum HunkType {
    Added,
    Removed,
    Modified,
}

impl From<core::HunkType> for HunkType {
    fn from(h: core::HunkType) -> Self {
        match h {
            core::HunkType::Added => Self::Added,
            core::HunkType::Removed => Self::Removed,
            core::HunkType::Modified => Self::Modified,
        }
    }
}

#[derive(uniffi::Record)]
pub struct ChangeDetail {
    pub info: ChangeInfo,
    pub diff: Vec<DiffHunk>,
}

impl From<core::ChangeDetail> for ChangeDetail {
    fn from(d: core::ChangeDetail) -> Self {
        Self {
            info: d.info.into(),
            diff: d.diff.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct BookmarkInfo {
    pub name: String,
    pub change_id: String,
    pub is_tracking_remote: bool,
}

impl From<core::BookmarkInfo> for BookmarkInfo {
    fn from(b: core::BookmarkInfo) -> Self {
        Self {
            name: b.name,
            change_id: b.change_id,
            is_tracking_remote: b.is_tracking_remote,
        }
    }
}

#[derive(uniffi::Object)]
pub struct JayJayRepo {
    inner: core::Repo,
}

#[uniffi::export]
impl JayJayRepo {
    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, JayJayError> {
        let repo = core::Repo::open(&PathBuf::from(&path))?;
        Ok(Arc::new(Self { inner: repo }))
    }

    pub fn path(&self) -> String {
        self.inner.path().display().to_string()
    }

    pub fn refresh_working_copy(&self) -> Result<(), JayJayError> {
        Ok(self.inner.refresh_working_copy()?)
    }

    pub fn log(&self, revset: String) -> Result<Vec<ChangeInfo>, JayJayError> {
        Ok(self.inner.log(&revset)?.into_iter().map(Into::into).collect())
    }

    pub fn log_graph(&self, revset: String) -> Result<Vec<GraphEntry>, JayJayError> {
        Ok(self.inner.log_graph(&revset)?.into_iter().map(Into::into).collect())
    }

    pub fn show(&self, rev: String) -> Result<ChangeDetail, JayJayError> {
        Ok(self.inner.show(&rev)?.into())
    }

    pub fn restore_files(&self, rev: String, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.restore_files(&rev, &paths)?)
    }

    pub fn ignore_and_untrack(&self, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.ignore_and_untrack(&paths)?)
    }

    pub fn split(&self, rev: String, paths: Vec<String>) -> Result<(), JayJayError> {
        Ok(self.inner.split(&rev, &paths)?)
    }

    pub fn describe(&self, rev: String, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.describe(&rev, &message)?)
    }

    pub fn new_change(&self, parent: String, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.new_change(&parent, &message)?)
    }

    pub fn squash(&self, rev: String, into_rev: Option<String>) -> Result<(), JayJayError> {
        Ok(self.inner.squash(&rev, into_rev.as_deref())?)
    }

    pub fn abandon(&self, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.abandon(&rev)?)
    }

    pub fn rebase(&self, rev: String, dest: String) -> Result<(), JayJayError> {
        Ok(self.inner.rebase(&rev, &dest)?)
    }

    pub fn list_bookmarks(&self) -> Result<Vec<BookmarkInfo>, JayJayError> {
        Ok(self.inner.list_bookmarks()?.into_iter().map(Into::into).collect())
    }

    pub fn create_bookmark(&self, name: String, rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.create_bookmark(&name, &rev)?)
    }

    pub fn move_bookmark(&self, name: String, to_rev: String) -> Result<(), JayJayError> {
        Ok(self.inner.move_bookmark(&name, &to_rev)?)
    }

    pub fn delete_bookmark(&self, name: String) -> Result<(), JayJayError> {
        Ok(self.inner.delete_bookmark(&name)?)
    }

    pub fn git_push(&self, bookmark: String) -> Result<(), JayJayError> {
        Ok(self.inner.git_push(&bookmark)?)
    }

    pub fn git_fetch(&self, remote: String) -> Result<(), JayJayError> {
        Ok(self.inner.git_fetch(&remote)?)
    }

    pub fn jj_commit(&self, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.jj_commit(&message)?)
    }

    pub fn commit_with_submodules(&self, message: String) -> Result<(), JayJayError> {
        Ok(self.inner.commit_with_submodules(&message)?)
    }

    pub fn dirty_submodules(&self) -> Result<Vec<String>, JayJayError> {
        Ok(self.inner.dirty_submodules()?)
    }

    pub fn diff_summary(&self) -> Result<String, JayJayError> {
        Ok(self.inner.diff_summary()?)
    }
}
