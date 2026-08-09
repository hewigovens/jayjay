use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{App, Context};
use jayjay_core::{ChangeInfo, DiffHunk};

use super::RepoWindow;
use super::file_actions::SelectedFilesRequest;
use crate::diff::file_status;
use crate::repo::revset;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

pub enum FileBatchAction {
    /// `files` pairs each path with its review identity; an empty identity means the path had no loaded hunk, which can be unmarked but never marked (SwiftUI skips those on mark).
    SetReviewed {
        change_id: String,
        reviewed: bool,
        files: Vec<(String, String)>,
    },
    Split(Arc<SelectedFilesRequest>),
    Commit(Arc<SelectedFilesRequest>),
    MoveToWorkingCopy(Arc<SelectedFilesRequest>),
    /// `rev` is always the selected change (the restore target); `from` names one parent of a merge as the content source and must never be passed as the rev, or the parent itself gets rewritten.
    Restore {
        rev: String,
        from: Option<String>,
        paths: Vec<String>,
    },
    Delete {
        paths: Vec<String>,
    },
    IgnoreAndUntrack {
        paths: Vec<String>,
    },
}

impl RepoWindow {
    /// Batch section of the file context menu, gated per action like SwiftUI's; compare mode gets none because an interdiff's files are not the change's files.
    pub(super) fn batch_file_menu_items(&self, paths: &[String], cx: &App) -> Vec<ContextMenuItem> {
        let vm = self.vm.read(cx);
        let Some(change) = vm.selected_change_for_file_ops() else {
            return Vec::new();
        };
        if paths.is_empty() {
            return Vec::new();
        }
        let hunks = hunks_for_paths(vm.files.as_deref().map(Vec::as_slice), paths);
        if hunks.iter().any(|hunk| file_status::is_submodule(hunk)) {
            return Vec::new();
        }
        let mut items = Vec::new();
        if vm.shows_review_controls()
            && hunks.len() == paths.len()
            && hunks.iter().all(|hunk| !hunk.review_identity.is_empty())
        {
            items.push(self.review_toggle_item(paths, change, &hunks));
        }
        if !change.is_immutable {
            let request = Arc::new(SelectedFilesRequest {
                rev: revset::change_revision(change),
                paths: paths.to_vec(),
            });
            items.push(ContextMenuItem::new(
                plural_label(paths, "Split to New Change", |n| {
                    format!("Split {n} Files to New Change")
                }),
                glyph::GIT_BRANCH,
                batch(FileBatchAction::Split(request.clone())),
            ));
            if change.is_working_copy {
                items.push(ContextMenuItem::new(
                    plural_label(paths, "Commit File", |n| format!("Commit {n} Files")),
                    glyph::CHECK,
                    batch(FileBatchAction::Commit(request)),
                ));
            } else {
                items.push(ContextMenuItem::new(
                    plural_label(paths, "Move to Working Copy", |n| {
                        format!("Move {n} Files to Working Copy")
                    }),
                    glyph::ARROW_DOWN,
                    batch(FileBatchAction::MoveToWorkingCopy(request)),
                ));
            }
        }
        // Restore rewrites the selected change, so immutable changes must not offer it (same gate as the change menu's Abandon); the remaining actions touch only the disk, .gitignore/@, or the review layer.
        if !change.is_immutable {
            items.extend(restore_items(change, paths));
        }
        if change.is_working_copy {
            items.push(ContextMenuItem::new(
                plural_label(paths, "Delete from Disk", |n| {
                    format!("Delete {n} Files from Disk")
                }),
                glyph::X_CIRCLE,
                batch(FileBatchAction::Delete {
                    paths: paths.to_vec(),
                }),
            ));
        }
        items.push(ContextMenuItem::new(
            plural_label(paths, "Ignore & Untrack", |n| {
                format!("Ignore & Untrack {n} Files")
            }),
            glyph::EYE_OFF,
            batch(FileBatchAction::IgnoreAndUntrack {
                paths: paths.to_vec(),
            }),
        ));
        items
    }

    /// SwiftUI parity: the toggle marks every selected file reviewed unless all of them already are, in which case it unmarks them all.
    fn review_toggle_item(
        &self,
        paths: &[String],
        change: &ChangeInfo,
        hunks: &[&DiffHunk],
    ) -> ContextMenuItem {
        let (all_reviewed, action) = self.set_reviewed_action(paths, change, hunks);
        let label = match (all_reviewed, paths.len()) {
            (true, 1) => "Mark as Unreviewed".to_owned(),
            (true, n) => format!("Mark {n} Files as Unreviewed"),
            (false, 1) => "Mark as Reviewed".to_owned(),
            (false, n) => format!("Mark {n} Files as Reviewed"),
        };
        ContextMenuItem::new(
            label,
            if all_reviewed {
                glyph::EYE_OFF
            } else {
                glyph::EYE
            },
            batch(action),
        )
    }

    fn set_reviewed_action(
        &self,
        paths: &[String],
        change: &ChangeInfo,
        hunks: &[&DiffHunk],
    ) -> (bool, FileBatchAction) {
        let change_id = change.change_id.id.clone();
        let identity_of: HashMap<&str, &str> = hunks
            .iter()
            .map(|h| (h.path.as_str(), h.review_identity.as_str()))
            .collect();
        let files: Vec<(String, String)> = paths
            .iter()
            .map(|p| {
                let identity = identity_of.get(p.as_str()).copied().unwrap_or_default();
                (p.clone(), identity.to_owned())
            })
            .collect();
        let all_reviewed = files.iter().all(|(path, identity)| {
            !identity.is_empty() && self.is_reviewed(&change_id, path, identity)
        });
        (
            all_reviewed,
            FileBatchAction::SetReviewed {
                change_id,
                reviewed: !all_reviewed,
                files,
            },
        )
    }

    /// Space in the file column: a multi-selection toggles every selected row together; otherwise the single selected file.
    pub fn toggle_reviewed_for_selected_files(&mut self, cx: &mut Context<Self>) {
        let paths = self.multi_selected_file_paths(cx);
        if paths.len() < 2 {
            self.toggle_reviewed_for_selected_file(cx);
            return;
        }
        let action = {
            let vm = self.vm.read(cx);
            let Some(change) = vm.selected_change_for_file_ops() else {
                return;
            };
            if !vm.shows_review_controls() {
                return;
            }
            let hunks = hunks_for_paths(vm.files.as_deref().map(Vec::as_slice), &paths);
            if hunks.iter().any(|hunk| file_status::is_submodule(hunk)) {
                return;
            }
            if hunks.iter().all(|hunk| hunk.review_identity.is_empty()) {
                return;
            }
            self.set_reviewed_action(&paths, change, &hunks).1
        };
        self.run_file_batch_action(Arc::new(action), cx);
    }

    pub(crate) fn run_file_batch_action(
        &mut self,
        action: Arc<FileBatchAction>,
        cx: &mut Context<Self>,
    ) {
        match action.as_ref() {
            FileBatchAction::SetReviewed {
                change_id,
                reviewed,
                files,
            } => {
                super::review::mutate(&self.review_store, |store| {
                    for (path, identity) in files {
                        if *reviewed && !identity.is_empty() {
                            store.mark_reviewed(change_id, path, identity);
                        } else if !*reviewed {
                            store.mark_unreviewed(change_id, path);
                        }
                    }
                });
                cx.notify();
            }
            FileBatchAction::Split(request) => self.open_split_files_modal(request.clone(), cx),
            FileBatchAction::Commit(request) => self.commit_selected_files(request.clone(), cx),
            FileBatchAction::MoveToWorkingCopy(request) => self
                .vm
                .update(cx, |vm, cx| {
                    vm.move_files_to_working_copy(request.rev.clone(), request.paths.clone(), cx)
                })
                .detach(),
            // No review-store cleanup after restore: marks and notes key on content identity, so a restored file's stale mark can never match again (SwiftUI likewise does nothing).
            FileBatchAction::Restore { rev, from, paths } => self
                .vm
                .update(cx, |vm, cx| {
                    vm.restore_files(rev.clone(), from.clone(), paths.clone(), cx)
                })
                .detach(),
            FileBatchAction::Delete { paths } => self
                .vm
                .update(cx, |vm, cx| vm.delete_files(paths.clone(), cx))
                .detach(),
            FileBatchAction::IgnoreAndUntrack { paths } => self
                .vm
                .update(cx, |vm, cx| vm.ignore_and_untrack(paths.clone(), cx))
                .detach(),
        }
    }
}

/// A merge change cannot restore from the ambiguous auto-merged parents; SwiftUI's parent submenu is flattened into one item per parent, with the parent as the restore SOURCE and the selected change staying the rewrite target (SwiftUI passes the parent as the rev, which wrongly rewrites the parent — a bug we deliberately do not mirror).
fn restore_items(change: &ChangeInfo, paths: &[String]) -> Vec<ContextMenuItem> {
    let label = plural_label(paths, "Restore to Parent", |n| {
        format!("Restore {n} Files to Parent")
    });
    let rev = revset::change_revision(change);
    if change.parents.len() > 1 {
        return change
            .parents
            .iter()
            .enumerate()
            .map(|(ix, parent)| {
                let short: String = parent.chars().take(8).collect();
                ContextMenuItem::new(
                    format!("{label} {}: {short}", ix + 1),
                    glyph::ARROW_CLOCKWISE,
                    batch(FileBatchAction::Restore {
                        rev: rev.clone(),
                        from: Some(parent.clone()),
                        paths: paths.to_vec(),
                    }),
                )
            })
            .collect();
    }
    vec![ContextMenuItem::new(
        label,
        glyph::ARROW_CLOCKWISE,
        batch(FileBatchAction::Restore {
            rev,
            from: None,
            paths: paths.to_vec(),
        }),
    )]
}

fn hunks_for_paths<'a>(files: Option<&'a [DiffHunk]>, paths: &[String]) -> Vec<&'a DiffHunk> {
    let path_set: HashSet<&str> = paths.iter().map(String::as_str).collect();
    files
        .unwrap_or_default()
        .iter()
        .filter(|hunk| path_set.contains(hunk.path.as_str()))
        .collect()
}

pub(super) fn plural_label(
    paths: &[String],
    singular: &str,
    plural: impl Fn(usize) -> String,
) -> String {
    match paths.len() {
        1 => singular.to_owned(),
        n => plural(n),
    }
}

pub(super) fn batch(action: FileBatchAction) -> ContextAction {
    ContextAction::FileBatch(Arc::new(action))
}
