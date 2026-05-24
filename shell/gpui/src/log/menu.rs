use gpui::{App, ClipboardItem, Context, Pixels, Point, SharedString};
use jayjay_core::ChangeInfo;

use super::LogView;
use crate::ui::context_menu::{ContextAction, ContextMenuItem, ContextMenuState};
use crate::ui::icons::glyph;

impl LogView {
    pub fn open_context_menu(
        &mut self,
        anchor: Point<Pixels>,
        items: Vec<ContextMenuItem>,
        cx: &mut Context<Self>,
    ) {
        if items.is_empty() {
            return;
        }
        self.context_menu = Some(ContextMenuState { anchor, items });
        cx.notify();
    }

    pub fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.take().is_some() {
            cx.notify();
        }
    }

    pub fn dispatch_context_action(&mut self, action: ContextAction, cx: &mut Context<Self>) {
        self.context_menu = None;
        match action {
            ContextAction::CopyText(text) => {
                cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
            }
            ContextAction::OpenUrl(url) => {
                cx.open_url(url.as_ref());
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
                        crate::windows::evolog::EvologView::open(repo, rev_string, title, cx);
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
                        crate::windows::file_history::FileHistoryView::open(
                            repo,
                            path_string,
                            parent,
                            cx,
                        );
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
            ContextAction::RevealChange(change_id) => {
                self.reveal_change_id(change_id.as_ref(), cx);
            }
            ContextAction::OpenInEditor(path) => {
                let repo_path = self.vm.read(cx).repo_path.to_string();
                crate::app::tools::open_in_editor(&repo_path, path.as_ref(), cx);
            }
            ContextAction::OpenInTerminal => {
                let repo_path = self.vm.read(cx).repo_path.to_string();
                crate::app::tools::open_in_terminal(&repo_path, cx);
            }
            ContextAction::OpenWorkspaceAt(path) => {
                let path = std::path::PathBuf::from(path.as_ref());
                cx.spawn(async move |_, cx| {
                    cx.update(|cx| {
                        crate::log::open_repo_window(path, cx);
                    });
                })
                .detach();
            }
        }
        cx.notify();
    }

    pub fn open_workspace_picker(&mut self, anchor: Point<Pixels>, cx: &mut Context<Self>) {
        let workspaces = self.vm.read(cx).graph.workspaces.clone();
        if workspaces.len() <= 1 {
            return;
        }
        let mut items: Vec<ContextMenuItem> = Vec::new();
        for ws in workspaces.iter() {
            let label = if ws.is_current {
                format!("✓ {}", ws.name)
            } else {
                ws.name.clone()
            };
            items.push(ContextMenuItem::new(
                label,
                if ws.is_current {
                    glyph::CHECK
                } else {
                    glyph::COLUMNS
                },
                ContextAction::OpenWorkspaceAt(ws.path.clone().into()),
            ));
        }
        self.open_context_menu(anchor, items, cx);
    }

    pub fn open_bookmark_picker(&mut self, anchor: Point<Pixels>, cx: &mut Context<Self>) {
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
                ContextAction::RevealChange(bm.change_id.clone().into()),
            ));
        }
        self.open_context_menu(anchor, items, cx);
    }

    pub(super) fn build_change_menu(change: &ChangeInfo) -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::new(
                "Copy Change ID",
                glyph::COPY,
                ContextAction::CopyText(change.change_id.clone().into()),
            ),
            ContextMenuItem::new(
                "Copy Commit ID",
                glyph::COPY,
                ContextAction::CopyText(change.commit_id.clone().into()),
            ),
            ContextMenuItem::new(
                "Show History (evolog)",
                glyph::ARROW_CLOCKWISE,
                ContextAction::OpenEvologFor(change.change_id.clone().into()),
            ),
        ]
    }

    pub fn build_file_menu(path: &str) -> Vec<ContextMenuItem> {
        let basename = path.rsplit('/').next().unwrap_or(path).to_owned();
        vec![
            ContextMenuItem::new(
                "Open in Editor",
                glyph::PENCIL_CIRCLE,
                ContextAction::OpenInEditor(path.to_owned().into()),
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

    pub(super) fn build_bookmark_menu(&self, name: &str, cx: &App) -> Vec<ContextMenuItem> {
        let mut items = vec![ContextMenuItem::new(
            "Copy Bookmark Name",
            glyph::COPY,
            ContextAction::CopyText(name.to_owned().into()),
        )];
        if let Some(repo) = self.vm.read(cx).repo.clone()
            && let Some(url) = repo.gh_pr_open_url(name)
        {
            items.push(ContextMenuItem::new(
                "Open PR on GitHub",
                glyph::ARROW_CIRCLE_RIGHT,
                ContextAction::OpenUrl(url.into()),
            ));
        }
        items
    }
}
