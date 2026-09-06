use super::RepoWindow;
#[cfg(not(target_os = "macos"))]
use crate::ui::app_menu::AppMenuState;
use crate::ui::context_menu::{ContextAction, ContextMenuItem, ContextMenuState};
use crate::ui::icons::glyph;
use crate::windows::evolog::EvologView;
use crate::windows::file_history::FileHistoryView;
use gpui::{App, ClipboardItem, Context, Pixels, Point, SharedString};

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
        self.context_menu = Some(ContextMenuState {
            anchor,
            items,
            submenu_index: None,
        });
        cx.notify();
    }

    pub(crate) fn set_context_submenu(
        &mut self,
        submenu_index: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        let Some(menu) = self.context_menu.as_mut() else {
            return;
        };
        if menu.submenu_index != submenu_index {
            menu.submenu_index = submenu_index;
            cx.notify();
        }
    }

    pub(crate) fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.take().is_some() {
            cx.notify();
        }
    }

    pub fn dispatch_context_action(&mut self, action: ContextAction, cx: &mut Context<Self>) {
        self.context_menu = None;
        // A picker stays up for its menu, but like the SwiftUI panel it dismisses once an action other than a copy runs.
        if !matches!(action, ContextAction::CopyText(_)) {
            self.close_bookmark_picker(cx);
            self.close_repo_switcher(cx);
        }
        match action {
            ContextAction::Noop => {}
            ContextAction::CopyText(text) => {
                cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
            }
            ContextAction::OpenUrl(url) => {
                crate::app::links::open_url(cx, url.as_ref());
            }
            ContextAction::CreateBookmark(rev) => {
                self.open_create_bookmark(rev.to_string(), cx);
            }
            ContextAction::OpenStackedPr(rev) => {
                self.open_stacked_pr(rev.to_string(), cx);
            }
            ContextAction::MoveBookmark { name, to_rev } => {
                self.move_bookmark(name, to_rev, cx);
            }
            ContextAction::PushBookmark(name) => {
                self.push_bookmark(name, cx);
            }
            ContextAction::DeleteBookmark { name, rev } => {
                self.delete_bookmark(name, rev, cx);
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
            ContextAction::Change(action) => self.run_change_action(action, cx),
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
            ContextAction::FilterBookmarkRevset(revset) => self.filter_bookmark_revset(&revset, cx),
            ContextAction::TrackBookmark { name, remote } => {
                self.close_bookmark_picker(cx);
                let task = self.vm.update(cx, |vm, cx| {
                    vm.bookmark_write(move |repo| repo.track_bookmark(&name, &remote), cx)
                });
                cx.spawn(async move |this, cx| {
                    if let Err(error) = task.await {
                        let _ = this.update(cx, |view, cx| {
                            view.show_toast(error.to_string(), cx);
                        });
                    }
                })
                .detach();
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
            ContextAction::OpenWorkspaceAt(path) => {
                let path = std::path::PathBuf::from(path.as_ref());
                cx.spawn(async move |_, cx| {
                    cx.update(|cx| {
                        crate::repo::window::open_repo_window(path, cx);
                    });
                })
                .detach();
            }
            ContextAction::SetRepositoryPinned { path, pinned } => {
                crate::app::repositories::set_pinned(
                    cx,
                    std::path::Path::new(path.as_ref()),
                    pinned,
                );
            }
            ContextAction::ForgetWorkspace { name, path } => {
                self.forget_workspace(name.to_string(), path.map(|path| path.to_string()), cx);
            }
            ContextAction::DeleteWorkspace { name, path } => {
                self.request_workspace_delete(name.to_string(), path.to_string(), cx);
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

#[cfg(test)]
mod tests {
    use crate::app::config::{AppConfig, AppConfigStore};

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
