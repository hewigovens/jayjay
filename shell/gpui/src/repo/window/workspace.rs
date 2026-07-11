//! New Workspace flow: name modal, sibling-directory destination convention, and the add mutation.

use std::path::{Path, PathBuf};

use gpui::{AppContext, Context};

use super::{RepoWindow, TextModalAction, TextModalState};
use crate::ui::text_area::TextArea;

impl RepoWindow {
    pub fn open_create_workspace(&mut self, cx: &mut Context<Self>) {
        let (repo_open, repo_path) = {
            let vm = self.vm.read(cx);
            (vm.repo.is_some(), vm.repo_path.to_string())
        };
        if !repo_open {
            self.show_toast("Repository is not open", cx);
            return;
        }
        let Some(parent) = sibling_parent_dir(&repo_path) else {
            self.show_toast("Repository has no parent directory for a workspace", cx);
            return;
        };
        let input = cx.new(|cx| TextArea::new("", "Workspace name", false, 32., cx));
        self.text_modal = Some(TextModalState {
            title: "New Workspace".into(),
            subtitle: format!("{}{}<name>", parent.display(), std::path::MAIN_SEPARATOR).into(),
            primary_label: "Create".into(),
            action: TextModalAction::CreateWorkspace(parent),
            input,
            focus_pending: true,
            context: None,
            checkbox: None,
            file_list: None,
        });
        cx.notify();
    }

    pub(super) fn submit_create_workspace(
        &mut self,
        parent: PathBuf,
        text: &str,
        cx: &mut Context<Self>,
    ) {
        let name = text.trim().to_owned();
        if name.is_empty() {
            self.show_toast("Workspace name required", cx);
            return;
        }
        if !jayjay_core::is_valid_workspace_name(&name) {
            self.show_toast(format!("Invalid workspace name: {name}"), cx);
            return;
        }
        let dest = parent.join(&name);
        self.text_modal = None;
        let task = {
            let dest = dest.to_string_lossy().into_owned();
            let name = name.clone();
            self.vm
                .update(cx, |vm, cx| vm.workspace_add(dest, name, cx))
        };
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, |view, cx| {
                    view.show_toast(format!("Created workspace {name}"), cx);
                });
                // SwiftUI parity: the new workspace opens in its own window right after creation.
                cx.update(|cx| {
                    crate::repo::window::open_repo_window(dest, cx);
                });
            }
        })
        .detach();
        cx.notify();
    }
}

/// Destination convention shared with SwiftUI: a workspace lives in a sibling directory of the repo, named after the workspace.
fn sibling_parent_dir(repo_path: &str) -> Option<PathBuf> {
    let parent = Path::new(repo_path).parent()?;
    if parent.as_os_str().is_empty() {
        return None;
    }
    Some(parent.to_path_buf())
}
