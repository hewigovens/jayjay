use std::sync::Arc;

use gpui::Context;
use jayjay_core::diff::FileDiff;
use jayjay_core::{DiffHunk, DiffPreview, DiffProjectionMode};

use super::super::{DiffLoadState, LoadedDiff, RepoViewModel};
use super::diff_compute::{compute_diff_blocking, diff_cache_key};
use crate::diff::{DetailMode, projection};

impl RepoViewModel {
    pub(in crate::repo) fn load_diff_async(
        &mut self,
        rev: String,
        hunk: DiffHunk,
        cx: &mut Context<Self>,
    ) {
        let projection_mode = projection::request_mode(hunk.projection.as_ref(), false);
        self.load_diff_async_with_projection(rev, hunk, projection_mode, cx);
    }

    pub(in crate::repo) fn load_diff_async_with_projection(
        &mut self,
        rev: String,
        hunk: DiffHunk,
        projection_mode: Option<DiffProjectionMode>,
        cx: &mut Context<Self>,
    ) {
        // Every selected-file request supersedes the previous one, including a cache hit; otherwise an older in-flight miss can finish after the cached selection and overwrite it.
        self.loading.diff_gen = self.loading.diff_gen.wrapping_add(1);
        let generation = self.loading.diff_gen;
        let compare_from_rev = self
            .compare
            .as_ref()
            .map(|compare| compare.from_rev.clone());
        let cache_key = diff_cache_key(
            compare_from_rev.as_deref(),
            &rev,
            &hunk,
            projection_mode,
            self.ignore_whitespace,
        );
        self.diff_cache.load_failures.remove(&cache_key);
        if let Some(cached) = self.diff_cache.loaded.get(&cache_key).cloned() {
            self.shown.diff = Some(cached.diff);
            self.shown.projection = cached.projection;
            self.shown.svg_preview = cached.svg_preview;
            self.shown.markdown_preview = cached.markdown_preview;
            self.shown.old_content = cached.old_content;
            self.shown.new_content = cached.new_content;
            self.shown.supports_file_editor = cached.supports_file_editor;
            self.loading.diff = false;
            if matches!(self.detail_mode, DetailMode::Annotate) {
                self.load_annotate(cx);
            }
            cx.notify();
            return;
        }

        self.shown.diff = None;
        self.shown.projection = None;
        self.shown.svg_preview = None;
        self.shown.markdown_preview = None;
        self.shown.old_content = None;
        self.shown.new_content = None;
        self.shown.supports_file_editor = false;
        self.loading.diff = true;

        let Some(repo) = self.repo.clone() else {
            self.loading.diff = false;
            cx.notify();
            return;
        };
        let fallback_path = hunk.path.clone();
        let ignore_whitespace = self.ignore_whitespace;
        let reviewable = self.shows_review_controls();
        cx.notify();

        Self::background_update(
            cx,
            async move {
                compute_diff_blocking(
                    &repo,
                    &rev,
                    &hunk,
                    compare_from_rev.as_deref(),
                    projection_mode,
                    ignore_whitespace,
                    reviewable,
                )
            },
            move |vm, file_diff, cx| {
                if vm.loading.diff_gen != generation {
                    return;
                }
                vm.loading.diff = false;
                match file_diff {
                    Ok(loaded) => {
                        let file_diff = Arc::new(loaded.file_diff);
                        vm.diff_cache.loaded.insert(
                            cache_key,
                            LoadedDiff {
                                diff: file_diff.clone(),
                                projection: loaded.projection.clone(),
                                svg_preview: loaded.svg_preview.clone().map(Arc::new),
                                markdown_preview: loaded.markdown_preview.clone().map(Arc::new),
                                old_content: Some(loaded.old_content.clone()),
                                new_content: Some(loaded.new_content.clone()),
                                supports_file_editor: loaded.supports_file_editor,
                                review: loaded.review,
                            },
                        );
                        vm.shown.diff = Some(file_diff);
                        vm.shown.projection = loaded.projection;
                        vm.shown.svg_preview = loaded.svg_preview.map(Arc::new);
                        vm.shown.markdown_preview = loaded.markdown_preview.map(Arc::new);
                        vm.shown.old_content = Some(loaded.old_content);
                        vm.shown.new_content = Some(loaded.new_content);
                        vm.shown.supports_file_editor = loaded.supports_file_editor;
                        vm.apply_hunk_previews(
                            &fallback_path,
                            loaded.old_preview,
                            loaded.new_preview,
                        );
                    }
                    Err(error) => {
                        vm.diff_cache.load_failures.insert(cache_key);
                        vm.shown.diff = Some(Arc::new(FileDiff {
                            path: fallback_path,
                            language: String::new(),
                            lines: Vec::new(),
                            whitespace_only_hidden: false,
                        }));
                        vm.shown.projection = None;
                        vm.shown.svg_preview = None;
                        vm.shown.markdown_preview = None;
                        vm.shown.old_content = None;
                        vm.shown.new_content = None;
                        vm.shown.supports_file_editor = false;
                        vm.present_error(error);
                    }
                }
                if matches!(vm.detail_mode, DetailMode::Annotate) {
                    vm.load_annotate(cx);
                }
                cx.notify();
            },
        );
    }

    fn apply_hunk_previews(
        &mut self,
        path: &str,
        old_preview: Option<DiffPreview>,
        new_preview: Option<DiffPreview>,
    ) {
        if old_preview.is_none() && new_preview.is_none() {
            return;
        }
        if let Some(files) = self.files.as_mut()
            && let Some(h) = Arc::make_mut(files).iter_mut().find(|h| h.path == path)
        {
            h.old.preview = old_preview;
            h.new.preview = new_preview;
        }
    }

    pub(in crate::repo) fn preload_diffs_async(
        &mut self,
        hunks: Arc<Vec<DiffHunk>>,
        cx: &mut Context<Self>,
    ) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let Some(rev) = self.selected_revision() else {
            return;
        };
        let generation = self.loading.change_gen;
        let ignore_whitespace = self.ignore_whitespace;
        let reviewable = self.shows_review_controls();
        let pending: Vec<_> = hunks
            .iter()
            .enumerate()
            .filter(|(ix, _)| Some(*ix) != self.selected_file_ix)
            .map(|(_, hunk)| {
                let projection_mode = projection::request_mode(hunk.projection.as_ref(), false);
                (
                    diff_cache_key(None, &rev, hunk, projection_mode, ignore_whitespace),
                    hunk.clone(),
                    projection_mode,
                )
            })
            .filter(|(key, _, _)| {
                !self.diff_cache.loaded.contains_key(key)
                    && !self.diff_cache.preloads_in_flight.contains(key)
                    && !self.diff_cache.load_failures.contains(key)
            })
            .collect();

        if pending.is_empty() {
            return;
        }

        for (cache_key, hunk, projection_mode) in pending {
            self.diff_cache.preloads_in_flight.insert(cache_key.clone());
            let repo = repo.clone();
            let rev = rev.clone();
            let hunk_path = hunk.path.clone();
            Self::background_update(
                cx,
                async move {
                    compute_diff_blocking(
                        &repo,
                        &rev,
                        &hunk,
                        None,
                        projection_mode,
                        ignore_whitespace,
                        reviewable,
                    )
                },
                move |vm, result, cx| {
                    if vm.loading.change_gen != generation {
                        return;
                    }
                    vm.diff_cache.preloads_in_flight.remove(&cache_key);
                    match result {
                        Ok(loaded) => {
                            vm.diff_cache.load_failures.remove(&cache_key);
                            vm.diff_cache.loaded.entry(cache_key).or_insert(LoadedDiff {
                                diff: Arc::new(loaded.file_diff),
                                projection: loaded.projection,
                                svg_preview: loaded.svg_preview.map(Arc::new),
                                markdown_preview: loaded.markdown_preview.map(Arc::new),
                                // `or_insert` never overwrites, so planting `None` here would permanently starve "Abandon Selected Lines" for any file later selected via this cache entry.
                                old_content: Some(loaded.old_content),
                                new_content: Some(loaded.new_content),
                                supports_file_editor: loaded.supports_file_editor,
                                review: loaded.review,
                            });
                            vm.apply_hunk_previews(
                                &hunk_path,
                                loaded.old_preview,
                                loaded.new_preview,
                            );
                        }
                        Err(_) => {
                            vm.diff_cache.load_failures.insert(cache_key);
                        }
                    }
                    cx.notify();
                },
            );
        }
    }

    pub(in crate::repo) fn diff_load_state(&self, hunk: &DiffHunk) -> DiffLoadState {
        let Some(rev) = self.selected_revision() else {
            return DiffLoadState::Missing;
        };
        let projection_mode = projection::request_mode(hunk.projection.as_ref(), false);
        let key = diff_cache_key(None, &rev, hunk, projection_mode, self.ignore_whitespace);
        if let Some(loaded) = self.diff_cache.loaded.get(&key) {
            DiffLoadState::Loaded(loaded.clone())
        } else if self.diff_cache.load_failures.contains(&key) {
            DiffLoadState::Failed
        } else {
            DiffLoadState::Missing
        }
    }
}
