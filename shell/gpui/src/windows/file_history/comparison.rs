use jayjay_core::{ChangeInfo, ShortId};

#[derive(Clone, Debug)]
pub(super) struct FileHistoryEndpoint {
    pub(super) commit_id: ShortId,
    pub(super) change_id: ShortId,
}

impl FileHistoryEndpoint {
    pub(super) fn from_change(change: &ChangeInfo) -> Self {
        Self {
            commit_id: change.commit_id.clone(),
            change_id: change.change_id.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct FileHistoryComparison {
    base: Option<FileHistoryEndpoint>,
    target: Option<FileHistoryEndpoint>,
}

impl FileHistoryComparison {
    pub(super) fn base(&self) -> Option<&FileHistoryEndpoint> {
        self.base.as_ref()
    }

    pub(super) fn target(&self) -> Option<&FileHistoryEndpoint> {
        self.target.as_ref()
    }

    pub(super) fn is_pinned(&self) -> bool {
        self.base.is_some()
    }

    pub(super) fn is_same_revision(&self) -> bool {
        match (&self.base, &self.target) {
            (Some(base), Some(target)) => base.commit_id.id == target.commit_id.id,
            _ => false,
        }
    }

    pub(super) fn compare_from(&self) -> Option<&str> {
        self.base
            .as_ref()
            .map(|endpoint| endpoint.commit_id.id.as_str())
    }

    pub(super) fn compare_to(&self) -> Option<&str> {
        self.target
            .as_ref()
            .map(|endpoint| endpoint.commit_id.id.as_str())
    }

    pub(super) fn select(&mut self, endpoint: FileHistoryEndpoint) {
        self.target = Some(endpoint);
    }

    pub(super) fn pin(&mut self, endpoint: FileHistoryEndpoint) {
        self.target = Some(endpoint.clone());
        self.base = Some(endpoint);
    }

    pub(super) fn swap(&mut self) {
        if self.base.is_some() && self.target.is_some() {
            std::mem::swap(&mut self.base, &mut self.target);
        }
    }

    pub(super) fn clear(&mut self) {
        self.base = None;
    }
}
