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
            jayjay_i18n::menu::open_in_new_window(),
            glyph::COLUMNS,
            ContextAction::OpenWorkspaceAt(workspace.path.clone().into()),
        ));
    }
    items.push(ContextMenuItem::new(
        jayjay_i18n::menu::copy_workspace_name(),
        glyph::COPY,
        ContextAction::CopyText(workspace.name.clone().into()),
    ));
    if workspace.is_path_resolved {
        items.push(ContextMenuItem::new(
            jayjay_i18n::menu::copy_path(),
            glyph::COPY,
            ContextAction::CopyText(workspace.path.clone().into()),
        ));
    }
    items
}

fn workspace_filter_item(name: &str) -> ContextMenuItem {
    ContextMenuItem::new(
        jayjay_i18n::menu::filter_by_this_workspace(),
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
