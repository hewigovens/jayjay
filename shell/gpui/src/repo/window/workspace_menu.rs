use gpui::App;
use jayjay_core::WorkspaceInfo;

use super::RepoWindow;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

pub(super) fn workspace_open_copy_items(workspace: &WorkspaceInfo) -> Vec<ContextMenuItem> {
    let mut items = Vec::new();
    if !workspace.is_current && workspace.is_path_resolved {
        items.push(ContextMenuItem::new(
            "Open in New Window",
            glyph::COLUMNS,
            ContextAction::OpenWorkspaceAt(workspace.path.clone().into()),
        ));
    }
    items.push(ContextMenuItem::new(
        "Copy Workspace Name",
        glyph::COPY,
        ContextAction::CopyText(workspace.name.clone().into()),
    ));
    if workspace.is_path_resolved {
        items.push(ContextMenuItem::new(
            "Copy Path",
            glyph::COPY,
            ContextAction::CopyText(workspace.path.clone().into()),
        ));
    }
    items
}

impl RepoWindow {
    /// No menu when the listing has no workspace of that name; a chip carries only the name.
    pub(super) fn build_workspace_chip_menu(&self, name: &str, cx: &App) -> Vec<ContextMenuItem> {
        self.vm
            .read(cx)
            .graph
            .workspaces
            .iter()
            .find(|workspace| workspace.name == name)
            .map(workspace_open_copy_items)
            .unwrap_or_default()
    }
}
