use gpui::{
    AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Styled, div, px, rgb,
};
use std::path::Path;

use jayjay_core::WorkspaceInfo;
use jayjay_core::repositories::normalize_repository_path;

use super::sections::{RowContent, SwitcherRow};
use crate::app::repositories;
use crate::app::theme::Theme;
use crate::repo::window::picker::row;
use crate::repo::window::{RepoWindow, format_relative, split_prefix};
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::capsule;

pub(super) fn switcher_row(
    item: SwitcherRow,
    selected: bool,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let mut element = row(item.id, selected, item.height, t);
    if let Some(action) = item.action {
        let view = view.clone();
        element = element.cursor_pointer().on_mouse_down(
            MouseButton::Left,
            move |_: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                let action = action.clone();
                view.update(cx, |view, cx| view.dispatch_repo_switcher(action, cx));
            },
        );
    }
    match item.content {
        RowContent::Workspace(workspace) => {
            let context_workspace = workspace.clone();
            let context_view = view.clone();
            element
                .on_mouse_down(MouseButton::Right, move |event: &MouseDownEvent, _, cx| {
                    let anchor = event.position;
                    context_view.update(cx, |view, cx| {
                        let primary_root = view.vm.read(cx).repo_root_path.to_string();
                        let items = workspace_context_items(&context_workspace, &primary_root);
                        view.open_context_menu(anchor, items, cx);
                    });
                })
                .child(workspace_row(workspace, t))
                .into_any_element()
        }
        RowContent::Repository {
            path,
            glyph,
            current,
        } => element
            .child(repository_row(path, glyph, current, t))
            .into_any_element(),
        RowContent::Action { label, glyph } => element
            .gap(px(5.))
            .text_size(px(13.))
            .child(icons::icon(glyph, 11., t.fg_dim))
            .child(label)
            .into_any_element(),
    }
}

fn workspace_row(workspace: WorkspaceInfo, t: &Theme) -> AnyElement {
    let shown = workspace.change_id.prefix(8);
    let (prefix, rest) = split_prefix(&shown, workspace.change_id.short_len);
    let description = if !workspace.is_path_resolved {
        "Path unavailable — Forget to clean up".to_owned()
    } else if workspace.description.trim().is_empty() {
        "(no description)".to_owned()
    } else {
        workspace.description.clone()
    };
    let description_color = if !workspace.is_path_resolved {
        t.wc_accent
    } else if workspace.description.trim().is_empty() {
        t.fg_faint
    } else {
        t.fg_dim
    };
    let file_label = (workspace.files_changed > 0).then(|| {
        format!(
            "{} file{}",
            workspace.files_changed,
            if workspace.files_changed == 1 {
                ""
            } else {
                "s"
            },
        )
    });
    div()
        .flex()
        .min_w_0()
        .flex_1()
        .flex_col()
        .gap(px(2.))
        .child(
            div()
                .flex()
                .items_center()
                .min_w_0()
                .gap(px(5.))
                .child(icons::icon(
                    if workspace.is_current {
                        glyph::CHECK
                    } else {
                        glyph::FOLDER
                    },
                    11.,
                    if workspace.is_current {
                        t.selected_accent
                    } else {
                        t.fg_dim
                    },
                ))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(format!("{}:", workspace.name)),
                )
                .child(
                    div()
                        .flex()
                        .font_family(crate::app::fonts::mono())
                        .text_size(px(11.))
                        .child(div().text_color(rgb(t.change_id_prefix)).child(prefix))
                        .child(div().text_color(rgb(t.fg_faint)).child(rest)),
                )
                .children(
                    workspace
                        .has_conflict
                        .then(|| capsule("conflict", t.tag_conflict_bg, t.tag_conflict_fg, 9.)),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .flex_none()
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(format_relative(workspace.timestamp)),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .min_w_0()
                .pl(px(19.))
                .gap(px(6.))
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .truncate()
                        .text_size(px(11.))
                        .text_color(rgb(description_color))
                        .child(description),
                )
                .children(file_label.map(|label| {
                    div()
                        .flex_none()
                        .text_size(px(10.))
                        .text_color(rgb(t.fg_dim))
                        .child(label)
                })),
        )
        .into_any_element()
}

fn repository_row(path: String, glyph: &'static str, current: bool, t: &Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .min_w_0()
        .flex_1()
        .gap(px(5.))
        .child(icons::icon(
            glyph,
            11.,
            if current { t.selected_accent } else { t.fg_dim },
        ))
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_size(px(13.))
                .child(repositories::repository_name(&path)),
        )
        .child(div().flex_1())
        .child(
            div()
                .max_w(px(130.))
                .truncate()
                .text_size(px(10.))
                .text_color(rgb(t.fg_faint))
                .child(path),
        )
        .into_any_element()
}

fn workspace_context_items(workspace: &WorkspaceInfo, primary_root: &str) -> Vec<ContextMenuItem> {
    let mut items = Vec::new();
    if !workspace.is_current && workspace.is_path_resolved {
        items.push(ContextMenuItem::new(
            "Open in New Window",
            glyph::COLUMNS,
            ContextAction::OpenWorkspaceAt(workspace.path.clone().into()),
        ));
    }
    if workspace.is_path_resolved {
        items.push(ContextMenuItem::new(
            "Copy Path",
            glyph::COPY,
            ContextAction::CopyText(workspace.path.clone().into()),
        ));
    }
    if !workspace.is_current {
        items.push(ContextMenuItem::new(
            "Forget",
            glyph::X_CIRCLE,
            ContextAction::ForgetWorkspace {
                name: workspace.name.clone().into(),
                path: workspace
                    .is_path_resolved
                    .then(|| workspace.path.clone().into()),
            },
        ));
        // The primary root owns .jj/repo; deleting it would take the repository with it.
        let owns_repository = normalize_repository_path(Path::new(&workspace.path))
            == normalize_repository_path(Path::new(primary_root));
        if workspace.is_path_resolved && !owns_repository {
            items.push(ContextMenuItem::new(
                "Forget & Delete from Disk",
                glyph::WARNING,
                ContextAction::DeleteWorkspace {
                    name: workspace.name.clone().into(),
                    path: workspace.path.clone().into(),
                },
            ));
        }
    }
    items
}
