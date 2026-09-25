use gpui::Context;
use jayjay_core::{ChangeInfo, RebaseMode};

use super::payload::DagDrag;
use super::state::DagRebaseRequest;
use crate::app::config;
use crate::repo::window::RepoWindow;

impl RepoWindow {
    pub(in crate::repo::window) fn drop_dag_drag_on_change(
        &mut self,
        drag: DagDrag,
        destination: ChangeInfo,
        cx: &mut Context<Self>,
    ) {
        if !drag.can_drop_on(&destination) {
            return;
        }
        match &drag {
            DagDrag::WorkingCopy => self.drop_working_copy_on_change(destination, cx),
            DagDrag::Bookmark { name, .. } => {
                let message = format!("Moved {name}");
                self.move_bookmark_to_rev(
                    name.clone(),
                    destination.selection_revision().to_owned(),
                    message,
                    cx,
                );
            }
            DagDrag::Change { .. } => {
                let Some(source) = drag.source_change() else {
                    return;
                };
                let request = DagRebaseRequest {
                    source_rev: source.selection_revision().to_owned(),
                    source_change_id: source.change_id.clone(),
                    source_commit_id: source.commit_id.clone(),
                    source_label: drag.source_label(),
                    dest_rev: destination.selection_revision().to_owned(),
                    dest_change_id: destination.change_id.clone(),
                    dest_commit_id: destination.commit_id.clone(),
                    dest_label: DagDrag::label_for_change(&destination),
                    selection_commit_ids: drag
                        .selection()
                        .map(|selection| selection.commit_ids.clone())
                        .unwrap_or_default(),
                };
                if config::current(cx).features.confirm_drag_rebase {
                    self.pending_rebase = Some(request);
                    cx.notify();
                } else {
                    self.run_drag_rebase(request, cx);
                }
            }
        }
    }

    pub(crate) fn confirm_drag_rebase(&mut self, cx: &mut Context<Self>) {
        let Some(request) = self.pending_rebase.take() else {
            return;
        };
        let all_shown = {
            let shown: std::collections::HashSet<&str> = self
                .vm
                .read(cx)
                .graph
                .changes
                .iter()
                .map(|change| change.commit_id.as_str())
                .collect();
            [
                request.source_commit_id.as_str(),
                request.dest_commit_id.as_str(),
            ]
            .into_iter()
            .chain(request.selection_commit_ids.iter().map(String::as_str))
            .all(|id| shown.contains(id))
        };
        if !all_shown {
            self.show_toast("Rebase cancelled: the changes moved while confirming", cx);
            return;
        }
        self.run_drag_rebase(request, cx);
    }

    pub(crate) fn cancel_drag_rebase(&mut self, cx: &mut Context<Self>) {
        if self.pending_rebase.take().is_some() {
            cx.notify();
        }
    }

    fn run_drag_rebase(&mut self, request: DagRebaseRequest, cx: &mut Context<Self>) {
        let source_label = request.source_label.clone();
        let dest_label = request.dest_label.clone();
        let task = self.vm.update(cx, |vm, cx| {
            if request.selection_commit_ids.is_empty() {
                vm.rebase_change(request.source_rev, request.dest_rev, RebaseMode::Source, cx)
            } else {
                vm.rebase_changes(request.selection_commit_ids, request.dest_rev, cx)
            }
        });
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, move |view, cx| {
                    view.show_toast(format!("Rebased {source_label} onto {dest_label}"), cx);
                });
            }
        })
        .detach();
    }

    fn drop_working_copy_on_change(&mut self, change: ChangeInfo, cx: &mut Context<Self>) {
        if change.is_immutable {
            return;
        }
        let task = self.vm.update(cx, |vm, cx| {
            vm.edit_change(change.selection_revision().to_owned(), cx)
        });
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, |view, cx| {
                    view.show_toast("Moved working copy", cx);
                });
            }
        })
        .detach();
    }
}
