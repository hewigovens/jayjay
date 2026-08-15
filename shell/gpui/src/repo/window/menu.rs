use gpui::{App, ClipboardItem, Context, Pixels, Point, SharedString};
use jayjay_core::{ChangeInfo, WorkspaceInfo};

use super::RepoWindow;
use crate::repo::revset;
#[cfg(not(target_os = "macos"))]
use crate::ui::app_menu::AppMenuState;
use crate::ui::context_menu::{ContextAction, ContextMenuItem, ContextMenuState};
use crate::ui::icons::glyph;
use crate::windows::evolog::EvologView;
use crate::windows::file_history::FileHistoryView;

impl RepoWindow {
    #[cfg(not(target_os = "macos"))]
    pub(crate) fn open_named_app_menu(
        &mut self,
        menu_name: SharedString,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.context_menu = None;
        self.app_menu = Some(AppMenuState {
            anchor,
            menu_name: Some(menu_name),
        });
        cx.notify();
    }

    #[cfg(not(target_os = "macos"))]
    pub(crate) fn app_menu_open(&self) -> bool {
        self.app_menu.is_some()
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn app_menu_open(&self) -> bool {
        false
    }

    #[cfg(not(target_os = "macos"))]
    pub(crate) fn close_app_menu(&mut self, cx: &mut Context<Self>) {
        if self.app_menu.take().is_some() {
            cx.notify();
        }
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn close_app_menu(&mut self, _cx: &mut Context<Self>) {}

    pub(crate) fn open_context_menu(
        &mut self,
        anchor: Point<Pixels>,
        items: Vec<ContextMenuItem>,
        cx: &mut Context<Self>,
    ) {
        if items.is_empty() {
            return;
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.app_menu = None;
        }
        self.context_menu = Some(ContextMenuState { anchor, items });
        cx.notify();
    }

    pub(crate) fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.take().is_some() {
            cx.notify();
        }
    }

    pub fn dispatch_context_action(&mut self, action: ContextAction, cx: &mut Context<Self>) {
        self.context_menu = None;
        match action {
            ContextAction::Noop => {}
            ContextAction::CopyText(text) => {
                cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
            }
            ContextAction::OpenUrl(url) => {
                cx.open_url(url.as_ref());
            }
            ContextAction::CreateBookmark(rev) => {
                self.open_create_bookmark(rev.to_string(), cx);
            }
            ContextAction::OpenStackedPr(rev) => {
                self.open_stacked_pr(rev.to_string(), cx);
            }
            ContextAction::MoveBookmarkToParent(name) => {
                self.move_bookmark_to_parent(name, cx);
            }
            ContextAction::PushBookmark(name) => {
                self.push_bookmark(name, cx);
            }
            ContextAction::DeleteBookmark(name) => {
                self.delete_bookmark(name, cx);
            }
            ContextAction::OpenPRForBookmark(name) => {
                self.open_pr_for_bookmark(name, cx);
            }
            ContextAction::NewChangeOnTop(parent) => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.new_change_on_top(parent.to_string(), cx));
                task.detach();
            }
            ContextAction::AbandonChange(rev) => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.abandon_change(rev.to_string(), cx));
                task.detach();
            }
            ContextAction::OpenEvologFor(rev) => {
                let vm = self.vm.read(cx);
                let Some(repo) = vm.repo.clone() else {
                    return;
                };
                let title = SharedString::from(rev.chars().take(12).collect::<String>());
                let rev_string = rev.to_string();
                cx.spawn(async move |_, cx| {
                    cx.update(|cx| {
                        EvologView::open(repo, rev_string, title, cx);
                    });
                })
                .detach();
            }
            ContextAction::OpenFileHistoryFor(path) => {
                let vm = self.vm.read(cx);
                let Some(repo) = vm.repo.clone() else {
                    return;
                };
                let path_string = path.to_string();
                let parent = cx.entity();
                cx.spawn(async move |_, cx| {
                    cx.update(|cx| {
                        FileHistoryView::open(repo, path_string, parent, cx);
                    });
                })
                .detach();
            }
            ContextAction::ToggleAnnotateFor(path) => {
                let target_ix =
                    self.vm.read(cx).files.as_ref().and_then(|files| {
                        files.iter().position(|h| h.path.as_str() == path.as_ref())
                    });
                if let Some(ix) = target_ix {
                    self.select_file(ix, cx);
                }
                self.toggle_annotate(cx);
            }
            ContextAction::ShowBookmarkDiff(request) => {
                self.vm
                    .update(cx, |vm, cx| vm.compare_bookmark_diff(request, cx));
            }
            ContextAction::RevealChange(change_id) => {
                self.reveal_change_id(change_id.as_ref(), cx);
            }
            ContextAction::OpenInEditor(path) => {
                let repo_path = self.vm.read(cx).repo_path.to_string();
                crate::app::tools::open_in_editor(&repo_path, path.as_ref(), cx);
            }
            ContextAction::ShowInFileManager(path) => {
                let repo_path = self.vm.read(cx).repo_path.to_string();
                if !crate::app::tools::show_in_file_manager(&repo_path, Some(path.as_ref())) {
                    self.show_toast("File could not be shown in the file manager", cx);
                }
            }
            ContextAction::OpenInTerminal => {
                let repo_path = self.vm.read(cx).repo_path.to_string();
                crate::app::tools::open_in_terminal(&repo_path, cx);
            }
            ContextAction::OpenWorkspaceAt(path) => {
                let path = std::path::PathBuf::from(path.as_ref());
                cx.spawn(async move |_, cx| {
                    cx.update(|cx| {
                        crate::repo::window::open_repo_window(path, cx);
                    });
                })
                .detach();
            }
            ContextAction::ForgetWorkspace(name) => {
                self.forget_workspace(name.to_string(), cx);
            }
            ContextAction::CreateWorkspace => {
                self.open_create_workspace(cx);
            }
            ContextAction::OpenDiffEdit => self.enter_diff_edit(cx),
            ContextAction::AbandonSelectedLines(request) => {
                self.abandon_selected_diff_lines(request, cx);
            }
            ContextAction::FileBatch(action) => {
                self.run_file_batch_action(action, cx);
            }
            ContextAction::OpenAddReviewNote(request) => {
                self.open_add_note_composer(request, cx);
            }
            ContextAction::OpenEditReviewNote(note_id) => {
                self.open_edit_note_composer(note_id.to_string(), cx);
            }
            ContextAction::ResolveReviewNote(note_id) => {
                self.resolve_review_note(note_id.to_string(), cx);
            }
            ContextAction::DeleteReviewNote(note_id) => {
                self.delete_review_note(note_id.to_string(), cx);
            }
        }
        cx.notify();
    }

    pub(crate) fn open_workspace_picker(&mut self, anchor: Point<Pixels>, cx: &mut Context<Self>) {
        let workspaces = self.vm.read(cx).graph.workspaces.clone();
        let items = workspace_menu_items(workspaces.as_ref());
        if items.is_empty() {
            return;
        }
        self.open_context_menu(anchor, items, cx);
    }

    pub(crate) fn open_bookmark_picker(&mut self, anchor: Point<Pixels>, cx: &mut Context<Self>) {
        let bookmarks = self.vm.read(cx).graph.bookmarks.clone();
        if bookmarks.is_empty() {
            return;
        }
        let mut tracked: Vec<_> = bookmarks
            .iter()
            .filter(|b| b.has_local_target && b.is_tracking_remote)
            .cloned()
            .collect();
        let mut local: Vec<_> = bookmarks
            .iter()
            .filter(|b| b.has_local_target && !b.is_tracking_remote)
            .cloned()
            .collect();
        tracked.sort_by(|a, b| a.name.cmp(&b.name));
        local.sort_by(|a, b| a.name.cmp(&b.name));
        let mut items: Vec<ContextMenuItem> = Vec::new();
        for bm in tracked.iter().chain(local.iter()) {
            items.push(ContextMenuItem::new(
                bm.name.clone(),
                glyph::ARROW_CIRCLE_RIGHT,
                ContextAction::RevealChange(bm.change_id.id.clone().into()),
            ));
        }
        self.open_context_menu(anchor, items, cx);
    }

    pub fn build_change_menu(&self, change: &ChangeInfo, cx: &App) -> Vec<ContextMenuItem> {
        let rev = revset::change_revision(change);
        let mut items = vec![
            ContextMenuItem::new(
                "New change on top",
                glyph::PLUS_CIRCLE,
                ContextAction::NewChangeOnTop(rev.clone().into()),
            ),
            ContextMenuItem::new(
                "Copy Change ID",
                glyph::COPY,
                ContextAction::CopyText(change.change_id.id.clone().into()),
            ),
            ContextMenuItem::new(
                "Copy Commit ID",
                glyph::COPY,
                ContextAction::CopyText(change.commit_id.id.clone().into()),
            ),
            ContextMenuItem::new(
                "Show History (evolog)",
                glyph::ARROW_CLOCKWISE,
                ContextAction::OpenEvologFor(rev.clone().into()),
            ),
            ContextMenuItem::new(
                "Create bookmark…",
                glyph::BOOKMARK,
                ContextAction::CreateBookmark(rev.clone().into()),
            ),
        ];
        if let Some(request) = self
            .vm
            .read(cx)
            .selected_change()
            .and_then(|base| revset::bookmark_diff_request(base, change))
        {
            items.push(ContextMenuItem::new(
                "Show Bookmark Diff",
                glyph::ARROWS_LEFT_RIGHT,
                ContextAction::ShowBookmarkDiff(request),
            ));
        }
        if !change.is_immutable {
            items.push(ContextMenuItem::new(
                "Stacked Pull Requests…",
                glyph::GIT_BRANCH,
                ContextAction::OpenStackedPr(rev.clone().into()),
            ));
            let label = if change.is_divergent {
                "Abandon (resolve divergence)"
            } else {
                "Abandon"
            };
            items.push(ContextMenuItem::new(
                label,
                glyph::X_CIRCLE,
                ContextAction::AbandonChange(rev.into()),
            ));
        }
        items
    }

    pub(crate) fn build_file_menu(path: &str, cx: &App) -> Vec<ContextMenuItem> {
        let basename = path.rsplit('/').next().unwrap_or(path).to_owned();
        vec![
            ContextMenuItem::new(
                crate::app::tools::open_in_editor_label(cx),
                glyph::PENCIL_CIRCLE,
                ContextAction::OpenInEditor(path.to_owned().into()),
            ),
            ContextMenuItem::new(
                "Show in File Manager",
                glyph::FOLDER,
                ContextAction::ShowInFileManager(path.to_owned().into()),
            ),
            ContextMenuItem::new(
                "Annotate",
                glyph::FILE_CODE,
                ContextAction::ToggleAnnotateFor(path.to_owned().into()),
            ),
            ContextMenuItem::new(
                "Show History",
                glyph::ARROW_CLOCKWISE,
                ContextAction::OpenFileHistoryFor(path.to_owned().into()),
            ),
            ContextMenuItem::new(
                "Copy Path",
                glyph::COPY,
                ContextAction::CopyText(path.to_owned().into()),
            ),
            ContextMenuItem::new(
                "Copy Filename",
                glyph::COPY,
                ContextAction::CopyText(basename.into()),
            ),
        ]
    }
}

fn workspace_menu_items(workspaces: &[WorkspaceInfo]) -> Vec<ContextMenuItem> {
    if workspaces.len() <= 1 {
        return Vec::new();
    }
    let mut items = Vec::new();
    for ws in workspaces {
        if ws.is_current {
            items.push(ContextMenuItem::new(
                ws.name.clone(),
                glyph::CHECK,
                ContextAction::Noop,
            ));
            continue;
        }
        items.push(ContextMenuItem::new(
            format!("Open {}", ws.name),
            glyph::COLUMNS,
            ContextAction::OpenWorkspaceAt(ws.path.clone().into()),
        ));
        if ws.name != "default" {
            items.push(ContextMenuItem::new(
                format!("Forget {}", ws.name),
                glyph::X_CIRCLE,
                ContextAction::ForgetWorkspace(ws.name.clone().into()),
            ));
        }
    }
    items.push(ContextMenuItem::new(
        "New Workspace…",
        glyph::PLUS_CIRCLE,
        ContextAction::CreateWorkspace,
    ));
    items
}

#[cfg(test)]
mod tests {
    use super::workspace_menu_items;
    use crate::app::config::{AppConfig, AppConfigStore};
    use crate::ui::context_menu::ContextAction;
    use jayjay_core::WorkspaceInfo;

    #[test]
    fn workspace_menu_opens_and_forgets_non_default_workspaces() {
        let items = workspace_menu_items(&[
            WorkspaceInfo::new("default", "/repo", true),
            WorkspaceInfo::new("feature", "/repo-feature", false),
        ]);

        let labels: Vec<_> = items.iter().map(|item| item.label.as_ref()).collect();
        assert_eq!(
            labels,
            vec![
                "default",
                "Open feature",
                "Forget feature",
                "New Workspace…"
            ]
        );
        assert!(matches!(items[0].action, ContextAction::Noop));
        assert!(matches!(
            &items[1].action,
            ContextAction::OpenWorkspaceAt(path) if path.as_ref() == "/repo-feature"
        ));
        assert!(matches!(
            &items[2].action,
            ContextAction::ForgetWorkspace(name) if name.as_ref() == "feature"
        ));
        assert!(matches!(&items[3].action, ContextAction::CreateWorkspace));
    }

    #[gpui::test]
    fn file_menu_uses_configured_editor_name(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let mut cfg = AppConfig::default();
            cfg.tools.external_editor = "zed".to_owned();
            cx.set_global(AppConfigStore::new(cfg));

            let items = crate::repo::window::RepoWindow::build_file_menu("src/main.rs", cx);
            assert_eq!(items[0].label.as_ref(), "Open in Zed");
        });
    }
}
