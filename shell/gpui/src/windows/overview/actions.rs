use std::path::{Path, PathBuf};

use gpui::{ClipboardItem, Context, Pixels, Point, Task};
use jayjay_core::overview::OverviewSnapshot;
use jayjay_core::{CoreResult, OverviewChange, OverviewLane, RebaseMode};

use super::OverviewView;
use super::menu::{AbandonRequest, OverviewAction, change_menu, lane_menu};
use crate::app::config;
use crate::repo::window::{RepoWindow, open_repo_window, reveal_in_repo_window};
use crate::ui::popup_menu::PopupMenu;

pub(super) enum Confirmation {
    Abandon(AbandonRequest),
    DeleteWorkspace { name: String, path: String },
}

impl Confirmation {
    pub(super) fn is_current(&self, snapshot: &OverviewSnapshot, lane_ids: &[String]) -> bool {
        match self {
            Self::Abandon(request) => request.is_current(snapshot, lane_ids),
            Self::DeleteWorkspace { name, .. } => snapshot.workspace(name).is_some(),
        }
    }
}

impl OverviewView {
    pub(super) fn target_path(&self, lane: &OverviewLane) -> String {
        let snapshot = self.snapshot.as_deref();
        if lane.has_current_workspace() {
            return self.repo_path.to_string();
        }
        lane.workspaces
            .iter()
            .filter_map(|workspace| snapshot?.workspace(&workspace.name))
            .find(|info| info.is_path_resolved)
            .map(|info| info.path.clone())
            .unwrap_or_else(|| self.repo_path.to_string())
    }

    pub(super) fn show_in_graph(
        &mut self,
        lane_ix: usize,
        change: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(lane) = self.lane(lane_ix) else {
            return;
        };
        let head = lane.head();
        let select = change.unwrap_or_else(|| head.commit_id.id.clone());
        let head = head.change_id.id.clone();
        let target = self.target_path(lane);
        cx.defer(move |cx| reveal_in_repo_window(Path::new(&target), head, select, cx));
    }

    pub(super) fn open_lane_menu(
        &mut self,
        lane: usize,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(snapshot) = self.snapshot.clone() else {
            return;
        };
        self.select_lane(lane, cx);
        self.menu = Some(PopupMenu {
            anchor,
            entries: lane_menu(&snapshot, lane, &self.lane_ids[lane]),
        });
    }

    pub(super) fn open_change_menu(
        &mut self,
        lane: usize,
        change: &OverviewChange,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.select_change(lane, change.commit_id.id.clone(), cx);
        self.menu = Some(PopupMenu {
            anchor,
            entries: change_menu(lane, change),
        });
    }

    pub(super) fn dispatch(&mut self, action: OverviewAction, cx: &mut Context<Self>) {
        self.menu = None;
        match action {
            OverviewAction::ShowInGraph { lane, change } => self.show_in_graph(lane, change, cx),
            OverviewAction::OpenWorkspace(path) => {
                cx.defer(move |cx| open_repo_window(PathBuf::from(path), cx));
            }
            OverviewAction::RevealWorkspace(path) => {
                crate::app::tools::show_in_file_manager(&path, None);
            }
            OverviewAction::RebaseOntoTrunk(root) => {
                let task = self.vm.update(cx, |vm, cx| {
                    vm.rebase_change(root, "trunk()".to_owned(), RebaseMode::Source, cx)
                });
                self.track(task, cx);
            }
            OverviewAction::Abandon(request) => {
                self.confirmation = Some(Confirmation::Abandon(request));
            }
            OverviewAction::ForgetWorkspace { name, path } => {
                self.with_parent(cx, |parent, cx| parent.forget_workspace(name, path, cx));
            }
            OverviewAction::DeleteWorkspace { name, path } => {
                if config::current(cx)
                    .features
                    .skip_workspace_delete_confirmation
                {
                    self.with_parent(cx, |parent, cx| parent.delete_workspace(name, path, cx));
                } else {
                    self.confirmation = Some(Confirmation::DeleteWorkspace { name, path });
                }
            }
            OverviewAction::Copy(text) => cx.write_to_clipboard(ClipboardItem::new_string(text)),
        }
        cx.notify();
    }

    pub(super) fn confirm(&mut self, cx: &mut Context<Self>) {
        match self.confirmation.take() {
            Some(Confirmation::Abandon(request)) => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| match request.commit_ids.as_slice() {
                        [commit_id] => vm.abandon_change(commit_id.clone(), cx),
                        _ => vm.abandon_changes(request.commit_ids, cx),
                    });
                self.track(task, cx);
            }
            Some(Confirmation::DeleteWorkspace { name, path }) => {
                self.with_parent(cx, |parent, cx| parent.delete_workspace(name, path, cx));
            }
            None => {}
        }
        cx.notify();
    }

    pub(super) fn with_parent(
        &self,
        cx: &mut Context<Self>,
        update: impl FnOnce(&mut RepoWindow, &mut Context<RepoWindow>),
    ) {
        if let Some(parent) = self.parent.upgrade() {
            parent.update(cx, update);
        }
    }

    pub(super) fn track(&mut self, task: Task<CoreResult<()>>, cx: &mut Context<Self>) {
        self.action_error = None;
        cx.spawn(async move |this, cx| {
            if let Err(error) = task.await {
                let _ = this.update(cx, |view, cx| {
                    view.action_error = Some(crate::app::error_text(error));
                    cx.notify();
                });
            }
        })
        .detach();
    }
}
