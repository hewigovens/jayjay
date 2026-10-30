use crate::ssh::{ConnectionState, HostPlatform, RemoteLocation, RemoteRepo};
use crate::*;
use jayjay_review::ReviewStore;
use std::path::{Path, PathBuf};

pub struct Repo {
    pub(super) backend: Backend,
}
pub(super) enum Backend {
    Local(Box<crate::repo::Repo>),
    Remote(RemoteRepo),
}

impl Repo {
    pub fn open(path: &Path) -> JayResult<Self> {
        let backend = match RemoteLocation::parse(&path.to_string_lossy())? {
            Some(location) => Backend::Remote(RemoteRepo::open(location)?),
            None => Backend::Local(Box::new(crate::repo::Repo::open(path)?)),
        };
        Ok(Self { backend })
    }
    pub fn path(&self) -> &Path {
        match &self.backend {
            Backend::Local(repo) => repo.path(),
            Backend::Remote(repo) => &repo.path,
        }
    }
    pub fn repository_store_path(&self) -> &Path {
        match &self.backend {
            Backend::Local(repo) => repo.repository_store_path(),
            Backend::Remote(repo) => &repo.store,
        }
    }
    pub fn primary_root(&self) -> PathBuf {
        match &self.backend {
            Backend::Local(repo) => repo.primary_root(),
            Backend::Remote(repo) => repo.primary.clone(),
        }
    }
    pub fn workspace_name(&self) -> &str {
        match &self.backend {
            Backend::Local(repo) => repo.workspace_name(),
            Backend::Remote(repo) => &repo.workspace,
        }
    }
    pub fn is_remote(&self) -> bool {
        matches!(self.backend, Backend::Remote(_))
    }
    pub fn remote_platform(&self) -> Option<HostPlatform> {
        match &self.backend {
            Backend::Local(_) => None,
            Backend::Remote(repo) => Some(repo.platform()),
        }
    }
    pub fn connection_state(&self) -> ConnectionState {
        match &self.backend {
            Backend::Local(_) => ConnectionState::Connected,
            Backend::Remote(repo) => repo.state(),
        }
    }
    pub fn connection_error(&self) -> Option<String> {
        match &self.backend {
            Backend::Local(_) => None,
            Backend::Remote(repo) => repo.error(),
        }
    }
    pub fn reconnect(&self) -> JayResult<()> {
        match &self.backend {
            Backend::Local(_) => Ok(()),
            Backend::Remote(repo) => repo.reconnect(),
        }
    }
    pub fn disconnect(&self) -> JayResult<()> {
        match &self.backend {
            Backend::Local(_) => Ok(()),
            Backend::Remote(repo) => repo.disconnect(),
        }
    }
    pub fn poll_remote_changes(&self) -> JayResult<bool> {
        match &self.backend {
            Backend::Local(_) => Ok(false),
            Backend::Remote(repo) => Ok(repo.changed(self.snapshot_operation()?)),
        }
    }
    pub fn sync_token(&self) -> SyncToken {
        match &self.backend {
            Backend::Local(repo) => repo.sync_token(),
            Backend::Remote(_) => SyncToken::standalone(),
        }
    }
    pub fn cancel_running_jj_processes(&self) {
        match &self.backend {
            Backend::Local(repo) => repo.cancel_running_jj_processes(),
            Backend::Remote(_) => {}
        }
    }
    pub fn review_file_snapshot(
        &self,
        rev: &str,
        path: &str,
        old_path: Option<&str>,
    ) -> JayResult<diff::ReviewFileSnapshot> {
        let hunk = match old_path {
            Some(old) if old != path => self.show_file_rename(rev, old, path)?,
            _ => self.show_file(rev, path)?,
        };
        Ok(review_snapshot_from_hunk(&hunk))
    }
    pub fn review_notes(
        &self,
        rev: &str,
        include_resolved: bool,
    ) -> JayResult<Vec<ReviewNoteStatus>> {
        match &self.backend {
            Backend::Local(repo) => repo.review_notes(rev, include_resolved),
            Backend::Remote(_) => Ok(self
                .review_notes_report(&ReviewStore::load(), rev, include_resolved)?
                .notes),
        }
    }
    pub fn review_notes_report(
        &self,
        store: &ReviewStore,
        rev: &str,
        include_resolved: bool,
    ) -> JayResult<ReviewNotesReport> {
        match &self.backend {
            Backend::Local(repo) => repo.review_notes_report(store, rev, include_resolved),
            Backend::Remote(_) => {
                let report = self.reconcile_review_notes(vec![], rev)?;
                self.reconcile_review_notes(
                    store.list_notes(&report.change_id, include_resolved),
                    rev,
                )
            }
        }
    }
}
