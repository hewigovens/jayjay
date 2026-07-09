use std::sync::Arc;

use gpui::Context;
use jayjay_core::diff::FileDiff;
use jayjay_core::{DiffHunk, DiffPreview, DiffProjectionMode};

use super::super::{LoadedDiff, RepoViewModel};
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
        if let Some(cached) = self.diff_cache.get(&cache_key).cloned() {
            self.current_diff = Some(cached.diff);
            self.current_projection = cached.projection;
            self.current_svg_preview = cached.svg_preview;
            self.current_markdown_preview = cached.markdown_preview;
            self.current_diff_old_content = cached.old_content;
            self.current_diff_new_content = cached.new_content;
            self.loading.diff = false;
            if matches!(self.detail_mode, DetailMode::Annotate) {
                self.load_annotate(cx);
            }
            cx.notify();
            return;
        }

        self.loading.diff_gen = self.loading.diff_gen.wrapping_add(1);
        let generation = self.loading.diff_gen;
        self.current_diff = None;
        self.current_projection = None;
        self.current_svg_preview = None;
        self.current_markdown_preview = None;
        self.current_diff_old_content = None;
        self.current_diff_new_content = None;
        self.loading.diff = true;

        let Some(repo) = self.repo.clone() else {
            self.loading.diff = false;
            cx.notify();
            return;
        };
        let fallback_path = hunk.path.clone();
        let ignore_whitespace = self.ignore_whitespace;
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
                        vm.diff_cache.insert(
                            cache_key,
                            LoadedDiff {
                                diff: file_diff.clone(),
                                projection: loaded.projection.clone(),
                                svg_preview: loaded.svg_preview.clone().map(Arc::new),
                                markdown_preview: loaded.markdown_preview.clone().map(Arc::new),
                                old_content: Some(loaded.old_content.clone()),
                                new_content: Some(loaded.new_content.clone()),
                            },
                        );
                        vm.current_diff = Some(file_diff);
                        vm.current_projection = loaded.projection;
                        vm.current_svg_preview = loaded.svg_preview.map(Arc::new);
                        vm.current_markdown_preview = loaded.markdown_preview.map(Arc::new);
                        vm.current_diff_old_content = Some(loaded.old_content);
                        vm.current_diff_new_content = Some(loaded.new_content);
                        vm.apply_hunk_previews(
                            &fallback_path,
                            loaded.old_preview,
                            loaded.new_preview,
                        );
                    }
                    Err(error) => {
                        vm.current_diff = Some(Arc::new(FileDiff {
                            path: fallback_path,
                            language: String::new(),
                            lines: Vec::new(),
                            whitespace_only_hidden: false,
                        }));
                        vm.current_projection = None;
                        vm.current_svg_preview = None;
                        vm.current_markdown_preview = None;
                        vm.current_diff_old_content = None;
                        vm.current_diff_new_content = None;
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
        let Some(rev) = self
            .selected
            .and_then(|i| self.graph.changes.get(i))
            .map(|c| c.change_id.clone())
        else {
            return;
        };
        let ignore_whitespace = self.ignore_whitespace;
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
            .filter(|(key, _, _)| !self.diff_cache.contains_key(key))
            .collect();

        if pending.is_empty() {
            return;
        }

        for (cache_key, hunk, projection_mode) in pending {
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
                    )
                },
                move |vm, result, _cx| {
                    let Ok(loaded) = result else {
                        return;
                    };
                    vm.diff_cache.entry(cache_key).or_insert(LoadedDiff {
                        diff: Arc::new(loaded.file_diff),
                        projection: loaded.projection,
                        svg_preview: loaded.svg_preview.map(Arc::new),
                        markdown_preview: loaded.markdown_preview.map(Arc::new),
                        // `or_insert` never overwrites, so planting `None` here would permanently starve "Abandon Selected Lines" for any file later selected via this cache entry.
                        old_content: Some(loaded.old_content),
                        new_content: Some(loaded.new_content),
                    });
                    vm.apply_hunk_previews(&hunk_path, loaded.old_preview, loaded.new_preview);
                },
            );
        }
    }
}
