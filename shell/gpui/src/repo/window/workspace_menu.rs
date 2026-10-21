use gpui::App;
use jayjay_core::{WorkspaceInfo, workspace_filter_revset};

use super::RepoWindow;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

pub(super) fn workspace_menu_items(workspace: &WorkspaceInfo) -> Vec<ContextMenuItem> {
    let mut items = vec![
        workspace_filter_item(&workspace.name),
        ContextMenuItem::separator(),
    ];
    if workspace.is_switch_target() {
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

fn workspace_filter_item(name: &str) -> ContextMenuItem {
    ContextMenuItem::new(
        "Filter by This Workspace",
        glyph::FILTER,
        ContextAction::FilterRevset(workspace_filter_revset(name).into()),
    )
}

impl RepoWindow {
    /// Open and copy need the listing's workspace of that name; a chip carries only the name.
    pub(super) fn build_workspace_chip_menu(&self, name: &str, cx: &App) -> Vec<ContextMenuItem> {
        if let Some(workspace) = self
            .vm
            .read(cx)
            .graph
            .workspaces
            .iter()
            .find(|workspace| workspace.name == name)
        {
            workspace_menu_items(workspace)
        } else {
            vec![workspace_filter_item(name)]
        }
    }
}
