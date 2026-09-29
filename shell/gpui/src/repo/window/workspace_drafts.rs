use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gpui::{App, EntityId, Global};
use jayjay_core::repositories::normalize_repository_path;

use super::commit_box::CommitDraft;

#[derive(Default)]
pub(super) struct WorkspaceDrafts(HashMap<(EntityId, PathBuf), CommitDraft>);

impl Global for WorkspaceDrafts {}

impl WorkspaceDrafts {
    pub(super) fn preserve(
        window: EntityId,
        path: PathBuf,
        draft: Option<CommitDraft>,
        cx: &mut App,
    ) {
        let drafts = &mut cx.default_global::<Self>().0;
        match draft {
            Some(draft) => drafts.insert((window, path), draft),
            None => drafts.remove(&(window, path)),
        };
    }

    pub(super) fn take(window: EntityId, path: &Path, cx: &mut App) -> Option<CommitDraft> {
        cx.default_global::<Self>()
            .0
            .remove(&(window, path.to_path_buf()))
    }

    pub(super) fn discard(path: &Path, cx: &mut App) {
        let path = normalize_repository_path(path);
        cx.default_global::<Self>()
            .0
            .retain(|(_, draft_path), _| *draft_path != path);
    }

    pub(super) fn clear(window: EntityId, cx: &mut App) {
        cx.default_global::<Self>()
            .0
            .retain(|(draft_window, _), _| *draft_window != window);
    }
}
