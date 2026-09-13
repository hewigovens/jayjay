use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{AppContext, Context, UniformListScrollHandle};
use jayjay_core::diff::FileDiff;
use jayjay_core::diff_edit::{DiffEditFile, DiffEditFileDiff, DiffEditSession};
use jayjay_core::placeholder::is_editable_text;
use jayjay_core::{DiffHunk, FileDiffStats, HunkType};

use crate::repo::view_model::DiffLoadState;
use crate::ui::scrollbar::ScrollbarBoundsSlot;

use super::rows::DiffEditRowModel;
use crate::repo::window::RepoWindow;

static NEXT_EPOCH: AtomicU64 = AtomicU64::new(1);

pub(super) fn next_diff_edit_epoch() -> u64 {
    NEXT_EPOCH.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub(super) struct DiffEditLoadedFile {
    pub(super) display_diff: Arc<FileDiff>,
    pub(super) display_to_full: Arc<HashMap<u32, u32>>,
}

struct DiffEditLoadResult {
    hunk: DiffHunk,
    old_content: String,
    new_content: String,
    computed: DiffEditFileDiff,
}

pub struct DiffEditState {
    pub(crate) active: bool,
    pub(super) session: DiffEditSession,
    pub(super) loaded_files: HashMap<String, DiffEditLoadedFile>,
    pub(super) loading: HashSet<String>,
    pub(super) known_unsupported: HashSet<String>,
    pub(super) change_id: Option<String>,
    pub(super) working_copy: bool,
    pub(super) focus_pending: bool,
    /// Supersession token: a bump abandons every in-flight card load and stats query.
    pub(super) epoch: u64,
    pub(super) stats_commit: Option<String>,
    pub(super) loaded_ignore_whitespace: bool,
    pub(super) loaded_commit: Option<String>,
    pub(super) stats: Option<HashMap<String, FileDiffStats>>,
    pub(super) rows: Option<Arc<DiffEditRowModel>>,
    pub(super) message: String,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) bounds: ScrollbarBoundsSlot,
}

impl Default for DiffEditState {
    fn default() -> Self {
        Self {
            active: false,
            session: DiffEditSession::default(),
            loaded_files: HashMap::new(),
            loading: HashSet::new(),
            known_unsupported: HashSet::new(),
            change_id: None,
            working_copy: false,
            focus_pending: false,
            epoch: 0,
            stats_commit: None,
            loaded_ignore_whitespace: false,
            loaded_commit: None,
            stats: None,
            rows: None,
            message: String::new(),
            scroll: UniformListScrollHandle::new(),
            bounds: ScrollbarBoundsSlot::default(),
        }
    }
}

pub(super) fn hunk_supports_diff_edit(hunk: &DiffHunk) -> bool {
    hunk.projection.is_none() && hunk.hunk_type != HunkType::Renamed
}

impl RepoWindow {
    /// Render and vm updates both drive this, so it must be a cheap no-op when nothing new arrived; the full-diff compute itself always runs off the main thread.
    pub(super) fn ensure_diff_edit_files(&mut self, cx: &mut Context<Self>) {
        if !self.diff_edit.active {
            return;
        }
        let Some(hunks) = self.vm.read(cx).files.clone() else {
            return;
        };
        let paths: Vec<String> = hunks.iter().map(|hunk| hunk.path.clone()).collect();
        self.diff_edit.session.prune_focus(&paths);
        for hunk in hunks.iter() {
            let path = hunk.path.clone();
            if self.diff_edit.loaded_files.contains_key(&path)
                || self.diff_edit.loading.contains(&path)
                || self.diff_edit.known_unsupported.contains(&path)
            {
                continue;
            }
            if !hunk_supports_diff_edit(hunk) {
                self.mark_diff_edit_unsupported(path);
                continue;
            }
            let (load_state, ignore_whitespace) = {
                let vm = self.vm.read(cx);
                (vm.diff_load_state(hunk), vm.ignore_whitespace)
            };
            let cached = match load_state {
                DiffLoadState::Missing => continue,
                DiffLoadState::Failed => {
                    self.mark_diff_edit_unsupported(path);
                    continue;
                }
                DiffLoadState::Loaded(cached) => cached,
            };
            let (Some(old), Some(new)) = (cached.old_content.clone(), cached.new_content.clone())
            else {
                self.mark_diff_edit_unsupported(path);
                continue;
            };
            if !is_editable_text(&old) || !is_editable_text(&new) {
                self.mark_diff_edit_unsupported(path);
                continue;
            }
            let epoch = self.diff_edit.epoch;
            self.diff_edit.loading.insert(path);
            let hunk = hunk.clone();
            cx.spawn(async move |this, cx| {
                let result = cx
                    .background_spawn(async move {
                        let computed = DiffEditFileDiff::compute(
                            &hunk.path,
                            &old,
                            &new,
                            ignore_whitespace,
                            true,
                        );
                        DiffEditLoadResult {
                            hunk,
                            old_content: old.to_string(),
                            new_content: new.to_string(),
                            computed,
                        }
                    })
                    .await;
                let _ = this.update(cx, |view, cx| {
                    view.finish_diff_edit_load(epoch, result, cx);
                });
            })
            .detach();
        }
    }

    fn finish_diff_edit_load(
        &mut self,
        epoch: u64,
        result: DiffEditLoadResult,
        cx: &mut Context<Self>,
    ) {
        let DiffEditLoadResult {
            hunk,
            old_content,
            new_content,
            computed,
        } = result;
        if !self.diff_edit.active || self.diff_edit.epoch != epoch {
            return;
        }
        self.diff_edit.loading.remove(&hunk.path);
        self.diff_edit.loaded_files.insert(
            hunk.path.clone(),
            DiffEditLoadedFile {
                display_diff: Arc::new(computed.display),
                display_to_full: Arc::new(
                    computed
                        .display_to_full
                        .into_iter()
                        .map(|mapping| (mapping.display_line, mapping.full_line))
                        .collect(),
                ),
            },
        );
        self.diff_edit.session.load(DiffEditFile {
            path: hunk.path,
            old_path: hunk.old_path,
            hunk_type: hunk.hunk_type,
            old_content: (hunk.hunk_type != HunkType::Added).then_some(old_content),
            new_content: (hunk.hunk_type != HunkType::Removed).then_some(new_content),
            changed_lines: computed.changed_lines,
        });
        self.invalidate_diff_edit_rows(cx);
    }

    fn mark_diff_edit_unsupported(&mut self, path: String) {
        self.diff_edit.session.skip(&path);
        self.diff_edit.known_unsupported.insert(path);
        self.diff_edit.rows = None;
    }

    pub fn diff_edit_file_supported(&self, hunk: &DiffHunk) -> bool {
        hunk_supports_diff_edit(hunk) && self.diff_edit.loaded_files.contains_key(&hunk.path)
    }

    pub fn diff_edit_has_known_unsupported(&self, cx: &Context<Self>) -> bool {
        if !self.diff_edit.known_unsupported.is_empty() {
            return true;
        }
        self.vm
            .read(cx)
            .files
            .as_ref()
            .is_some_and(|files| files.iter().any(|hunk| !hunk_supports_diff_edit(hunk)))
    }
}
