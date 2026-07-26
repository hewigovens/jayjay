use std::sync::Arc;

use gpui::Context;
use jayjay_core::diff::FileDiff;
use jayjay_core::{DiffHunk, HunkType};

use super::state::hunk_supports_diff_edit;
use crate::repo::view_model::DiffLoadState;
use crate::repo::window::RepoWindow;

pub(super) struct DiffEditRowModel {
    pub(super) files: Vec<DiffEditCardFile>,
    pub(super) rows: Vec<DiffEditRow>,
    hunks: Arc<Vec<DiffHunk>>,
    loaded_count: usize,
    unsupported_count: usize,
    diff_cache_count: usize,
}

impl DiffEditRowModel {
    pub(super) fn file_index(&self, path: &str) -> Option<usize> {
        self.files
            .iter()
            .position(|file| file.path.as_ref() == path)
    }
}

pub(super) struct DiffEditCardFile {
    pub(super) path: Arc<str>,
    pub(super) hunk_type: HunkType,
    pub(super) supported: bool,
    pub(super) diff: Option<Arc<FileDiff>>,
    pub(super) changed_total: usize,
}

pub(super) enum DiffEditRow {
    Notice,
    Gap,
    HeaderPad {
        top: bool,
    },
    Header(usize),
    Line {
        file: usize,
        line_ix: u32,
        full_line: Option<u32>,
    },
    Placeholder {
        loading: bool,
    },
}

impl RepoWindow {
    pub(super) fn diff_edit_row_model(&mut self, cx: &Context<Self>) -> Arc<DiffEditRowModel> {
        let (hunks, diff_cache_count) = {
            let vm = self.vm.read(cx);
            (vm.files.clone().unwrap_or_default(), vm.diff_cache.len())
        };
        if let Some(model) = self.diff_edit.rows.as_ref()
            && Arc::ptr_eq(&model.hunks, &hunks)
            && model.loaded_count == self.diff_edit.loaded_files.len()
            && model.unsupported_count == self.diff_edit.known_unsupported.len()
            && model.diff_cache_count == diff_cache_count
        {
            return model.clone();
        }
        let model = Arc::new(self.build_diff_edit_rows(hunks, diff_cache_count, cx));
        self.diff_edit.rows = Some(model.clone());
        model
    }

    pub fn diff_edit_preview_line_count(&mut self, path: &str, cx: &Context<Self>) -> usize {
        self.diff_edit_row_model(cx)
            .files
            .iter()
            .find(|file| file.path.as_ref() == path)
            .and_then(|file| file.diff.as_ref())
            .map(|diff| diff.lines.len())
            .unwrap_or(0)
    }

    pub fn diff_edit_line_rows(&mut self, path: &str, cx: &Context<Self>) -> usize {
        let model = self.diff_edit_row_model(cx);
        let Some(file_ix) = model.file_index(path) else {
            return 0;
        };
        model
            .rows
            .iter()
            .filter(|row| matches!(row, DiffEditRow::Line { file, .. } if *file == file_ix))
            .count()
    }

    fn build_diff_edit_rows(
        &self,
        hunks: Arc<Vec<DiffHunk>>,
        diff_cache_count: usize,
        cx: &Context<Self>,
    ) -> DiffEditRowModel {
        let mut files = Vec::with_capacity(hunks.len());
        let mut rows = Vec::new();
        if self.diff_edit_has_known_unsupported(cx) {
            rows.push(DiffEditRow::Notice);
        }
        for (file_ix, hunk) in hunks.iter().enumerate() {
            if !rows.is_empty() {
                rows.push(DiffEditRow::Gap);
            }
            let loaded = self.diff_edit.loaded_files.get(&hunk.path);
            let supported = hunk_supports_diff_edit(hunk) && loaded.is_some();
            // The keyed lookup pins the default projection mode; a path scan can surface the wrong entry and can never match processed rows stored under the projection's virtual path.
            let preview = loaded.map(|file| file.display_diff.clone()).or_else(|| {
                match self.vm.read(cx).diff_load_state(hunk) {
                    DiffLoadState::Loaded(cached) => Some(cached.diff.clone()),
                    _ => None,
                }
            });
            rows.push(DiffEditRow::HeaderPad { top: true });
            rows.push(DiffEditRow::Header(file_ix));
            rows.push(DiffEditRow::HeaderPad { top: false });
            if !self.diff_edit.collapsed.contains(&hunk.path) {
                match &preview {
                    Some(diff) if !diff.lines.is_empty() => {
                        let map = loaded.map(|file| file.display_to_full.clone());
                        for line_ix in 0..diff.lines.len() as u32 {
                            let full_line = map
                                .as_ref()
                                .filter(|_| supported)
                                .and_then(|map| map.get(&(line_ix + 1)).copied());
                            rows.push(DiffEditRow::Line {
                                file: file_ix,
                                line_ix,
                                full_line,
                            });
                        }
                    }
                    Some(_) => rows.push(DiffEditRow::Placeholder { loading: false }),
                    None => rows.push(DiffEditRow::Placeholder {
                        loading: !self.diff_edit.known_unsupported.contains(&hunk.path),
                    }),
                }
            }
            files.push(DiffEditCardFile {
                path: hunk.path.as_str().into(),
                hunk_type: hunk.hunk_type,
                supported,
                diff: preview,
                changed_total: loaded.map(|file| file.changed.len()).unwrap_or(0),
            });
        }
        DiffEditRowModel {
            files,
            rows,
            hunks,
            loaded_count: self.diff_edit.loaded_files.len(),
            unsupported_count: self.diff_edit.known_unsupported.len(),
            diff_cache_count,
        }
    }
}
