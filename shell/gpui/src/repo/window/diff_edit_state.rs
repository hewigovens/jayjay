use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{AppContext, Context, UniformListScrollHandle};
use jayjay_core::diff::{
    CollapsedDiff, DiffSpanStyle, FileDiff, collapse_context_with_mapping, compute_file_diff_full,
};
use jayjay_core::placeholder::is_editable_text;
use jayjay_core::{DiffHunk, HunkType};

use crate::repo::view_model::DiffLoadState;
use crate::ui::scrollbar::ScrollbarBoundsSlot;

use super::RepoWindow;
use super::diff_edit_rows::DiffEditRowModel;

static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

pub(super) fn next_diff_edit_session() -> u64 {
    NEXT_SESSION.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub(super) struct DiffEditLoadedFile {
    pub(super) old_content: Arc<str>,
    pub(super) new_content: Arc<str>,
    pub(super) display_diff: Arc<FileDiff>,
    pub(super) display_to_full: Arc<HashMap<u32, u32>>,
    pub(super) changed: Arc<BTreeSet<u32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffEditCheckboxState {
    None,
    Some,
    All,
}

pub struct DiffEditState {
    pub active: bool,
    pub selected: HashMap<String, BTreeSet<u32>>,
    pub(super) loaded_files: HashMap<String, DiffEditLoadedFile>,
    pub(super) loading: HashSet<String>,
    pub(super) known_unsupported: HashSet<String>,
    pub(super) select_all_pending: HashSet<String>,
    pub(super) change_id: Option<String>,
    pub(super) working_copy: bool,
    pub(super) focus_pending: bool,
    pub(super) session: u64,
    pub(super) summary: (usize, usize),
    pub(super) rows: Option<Arc<DiffEditRowModel>>,
    pub(super) message: String,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) bounds: ScrollbarBoundsSlot,
}

impl Default for DiffEditState {
    fn default() -> Self {
        Self {
            active: false,
            selected: HashMap::new(),
            loaded_files: HashMap::new(),
            loading: HashSet::new(),
            known_unsupported: HashSet::new(),
            select_all_pending: HashSet::new(),
            change_id: None,
            working_copy: false,
            focus_pending: false,
            session: 0,
            summary: (0, 0),
            rows: None,
            message: String::new(),
            scroll: UniformListScrollHandle::new(),
            bounds: ScrollbarBoundsSlot::default(),
        }
    }
}

pub(super) fn changed_lines(diff: &FileDiff) -> BTreeSet<u32> {
    diff.lines
        .iter()
        .enumerate()
        .filter(|(_, line)| matches!(line.style, DiffSpanStyle::Added | DiffSpanStyle::Removed))
        .map(|(ix, _)| ix as u32 + 1)
        .collect()
}

pub(super) fn checkbox_state(
    selected: Option<&BTreeSet<u32>>,
    all: &BTreeSet<u32>,
) -> DiffEditCheckboxState {
    let count = selected
        .map(|selected| selected.intersection(all).count())
        .unwrap_or(0);
    if count == 0 {
        DiffEditCheckboxState::None
    } else if count == all.len() {
        DiffEditCheckboxState::All
    } else {
        DiffEditCheckboxState::Some
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
            let session = self.diff_edit.session;
            self.diff_edit.loading.insert(path.clone());
            let compute_path = path.clone();
            let (task_old, task_new) = (old.clone(), new.clone());
            cx.spawn(async move |this, cx| {
                let (full, collapsed) = cx
                    .background_spawn(async move {
                        let full = compute_file_diff_full(
                            &compute_path,
                            &task_old,
                            &task_new,
                            ignore_whitespace,
                        );
                        let collapsed = collapse_context_with_mapping(&full);
                        (full, collapsed)
                    })
                    .await;
                let _ = this.update(cx, |view, cx| {
                    view.finish_diff_edit_load(session, path, old, new, full, collapsed, cx);
                });
            })
            .detach();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_diff_edit_load(
        &mut self,
        session: u64,
        path: String,
        old_content: Arc<str>,
        new_content: Arc<str>,
        full: FileDiff,
        collapsed: CollapsedDiff,
        cx: &mut Context<Self>,
    ) {
        if !self.diff_edit.active || self.diff_edit.session != session {
            return;
        }
        self.diff_edit.loading.remove(&path);
        let changed = Arc::new(changed_lines(&full));
        let loaded = DiffEditLoadedFile {
            old_content,
            new_content,
            display_diff: Arc::new(collapsed.diff),
            display_to_full: Arc::new(
                collapsed
                    .display_to_full
                    .into_iter()
                    .map(|mapping| (mapping.display_line, mapping.full_line))
                    .collect(),
            ),
            changed: changed.clone(),
        };
        self.diff_edit.loaded_files.insert(path.clone(), loaded);
        self.diff_edit.rows = None;
        if self.diff_edit.select_all_pending.remove(&path) {
            self.diff_edit.selected.insert(path, (*changed).clone());
            self.refresh_diff_edit_summary();
        }
        cx.notify();
    }

    pub(super) fn mark_diff_edit_unsupported(&mut self, path: String) {
        self.diff_edit.select_all_pending.remove(&path);
        self.diff_edit.known_unsupported.insert(path);
        self.diff_edit.rows = None;
    }

    pub(super) fn refresh_diff_edit_summary(&mut self) {
        let mut files = 0;
        let mut lines = 0;
        for (path, loaded) in &self.diff_edit.loaded_files {
            let count = self
                .diff_edit
                .selected
                .get(path)
                .map(|selected| selected.intersection(&loaded.changed).count())
                .unwrap_or(0);
            if count > 0 {
                files += 1;
                lines += count;
            }
        }
        self.diff_edit.summary = (lines, files);
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
