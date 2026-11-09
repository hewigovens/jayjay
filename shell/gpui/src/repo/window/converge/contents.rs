use std::sync::Arc;

use gpui::{AppContext, Context, SharedString, Task};
use jayjay_core::diff::{FileDiff, compute_file_diff};
use jayjay_core::{ConvergePlan, DiffHunk, DiffProjectionMode, projection};

use super::sheet::ConvergeSheet;
use crate::repo::window::{RepoModal, RepoWindow};

pub(super) type Loaded<T> = Option<Result<Arc<T>, SharedString>>;

#[derive(Default)]
pub(crate) struct ConvergeContents {
    pub(super) target: Option<String>,
    task: Option<Task<()>>,
    pub(super) files: Loaded<Vec<DiffHunk>>,
    pub(super) selected_path: Option<String>,
    pub(super) diff: Loaded<FileDiff>,
}

impl ConvergeContents {
    pub(super) fn comparing(plan: &ConvergePlan) -> Self {
        Self {
            target: plan
                .versions
                .get(1)
                .map(|version| version.commit_id.clone()),
            ..Self::default()
        }
    }
}

impl ConvergeSheet {
    fn endpoints(&self) -> Option<(String, String)> {
        let from = self.review.as_ref()?.versions().first()?.commit_id.clone();
        Some((from, self.contents.target.clone()?))
    }
}

impl RepoWindow {
    fn converge_contents(&mut self) -> Option<&mut ConvergeContents> {
        match &mut self.modal {
            Some(RepoModal::Converge(sheet)) => Some(&mut sheet.contents),
            _ => None,
        }
    }

    pub(super) fn load_converge_contents(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            return;
        };
        let Some(RepoModal::Converge(sheet)) = &mut self.modal else {
            return;
        };
        let Some((from, to)) = sheet.endpoints() else {
            return;
        };
        let contents = &mut sheet.contents;
        contents.files = None;
        contents.diff = None;
        contents.task = Some(cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.interdiff_summary(&from, &to) })
                .await;
            let _ = this.update(cx, |view, cx| {
                let Some(contents) = view.converge_contents() else {
                    return;
                };
                let files = result.map(|detail| Arc::new(detail.diff));
                let keep = contents.selected_path.take().filter(|path| {
                    files
                        .as_ref()
                        .is_ok_and(|files| files.iter().any(|file| &file.path == path))
                });
                contents.files = Some(files.map_err(crate::app::error_text));
                if let Some(path) = keep {
                    view.select_converge_file(path, cx);
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(super) fn compare_converge_with(&mut self, commit_id: String, cx: &mut Context<Self>) {
        if let Some(RepoModal::Converge(sheet)) = &mut self.modal {
            sheet.contents.target = Some(commit_id);
            self.load_converge_contents(cx);
        }
    }

    pub(super) fn toggle_converge_file(&mut self, path: String, cx: &mut Context<Self>) {
        let Some(contents) = self.converge_contents() else {
            return;
        };
        if contents.selected_path.as_ref() == Some(&path) {
            contents.selected_path = None;
            contents.diff = None;
            cx.notify();
        } else {
            self.select_converge_file(path, cx);
        }
    }

    fn select_converge_file(&mut self, path: String, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            return;
        };
        let Some(RepoModal::Converge(sheet)) = &mut self.modal else {
            return;
        };
        let Some((from, to)) = sheet.endpoints() else {
            return;
        };
        let contents = &mut sheet.contents;
        let Some(file) = contents
            .files
            .as_ref()
            .and_then(|files| files.as_ref().ok())
            .and_then(|files| files.iter().find(|file| file.path == path))
        else {
            return;
        };
        let old_path = file.old_path.clone();
        let mode =
            projection::default_mode(file.projection.as_ref()).unwrap_or(DiffProjectionMode::Raw);
        contents.selected_path = Some(path.clone());
        contents.diff = None;
        contents.task = Some(cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let hunk = match old_path.as_deref() {
                        Some(old_path) => {
                            repo.interdiff_file_rename_with_mode(&from, &to, old_path, &path, mode)
                        }
                        None => repo.interdiff_file_with_mode(&from, &to, &path, mode),
                    }?;
                    let old = hunk.old.content.unwrap_or_default();
                    let new = hunk.new.content.unwrap_or_default();
                    Ok::<_, jayjay_core::JayError>(compute_file_diff(&hunk.path, &old, &new, false))
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                if let Some(contents) = view.converge_contents() {
                    contents.diff = Some(result.map(Arc::new).map_err(crate::app::error_text));
                    cx.notify();
                }
            });
        }));
        cx.notify();
    }
}
