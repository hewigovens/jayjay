use std::path::PathBuf;

use gpui::{
    AnyElement, Div, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::repositories::RepoGroup;

use super::actions::repository_actions;
use super::sections::RowKind;
use crate::app::repositories;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;

pub(super) fn repository_card(
    index: usize,
    group: RepoGroup,
    kind: RowKind,
    pinned_paths: &[String],
    t: &Theme,
) -> AnyElement {
    let (prefix, pinned) = match kind {
        RowKind::Pinned => ("repo-list-pinned", true),
        RowKind::Recent => ("repo-list", false),
    };
    let root_path = group.path;
    let pin_id = format!("{prefix}-pin-{index}");
    let remove_id = (!pinned).then(|| format!("repo-list-remove-{index}"));

    if group.workspaces.is_empty() {
        let open_path = root_path.clone();
        card_container(format!("{prefix}-row-{index}"), t)
            .items_center()
            .gap(px(8.))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(t.selected_bg)))
            .on_click(move |_, _, cx| {
                crate::repo::open_repo_window(PathBuf::from(&open_path), cx);
            })
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .truncate()
                            .text_size(ui_font_size(12.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(repositories::repository_name(&root_path)),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(ui_font_size(10.))
                            .text_color(rgb(t.fg_dim))
                            .child(root_path.clone()),
                    ),
            )
            .children(repository_actions(root_path, pinned, pin_id, remove_id, t))
            .into_any_element()
    } else {
        let group_prefix = format!("{prefix}-group-{index}");
        let mut card = card_container(group_prefix.clone(), t)
            .flex_col()
            .gap(px(7.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .text_size(ui_font_size(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(repositories::repository_name(&root_path)),
                    )
                    .children(repository_actions(
                        root_path.clone(),
                        pinned,
                        pin_id,
                        remove_id,
                        t,
                    )),
            )
            .child(workspace_entry_row(
                format!("{group_prefix}-default"),
                "default".to_owned(),
                root_path,
                None,
                t,
            ));
        for (workspace_index, path) in group.workspaces.into_iter().enumerate() {
            let workspace_pinned = pinned_paths.contains(&path);
            let actions = WorkspaceActions {
                pinned: workspace_pinned,
                pin_id: format!("{group_prefix}-workspace-pin-{workspace_index}"),
                remove_id: (!workspace_pinned)
                    .then(|| format!("{group_prefix}-workspace-remove-{workspace_index}")),
            };
            card = card.child(workspace_entry_row(
                format!("{group_prefix}-workspace-{workspace_index}"),
                repositories::repository_name(&path),
                path,
                Some(actions),
                t,
            ));
        }
        card.into_any_element()
    }
}

fn card_container(id: String, t: &Theme) -> gpui::Stateful<Div> {
    let id = SharedString::from(id);
    div()
        .id(id.clone())
        .debug_selector(move || id.to_string())
        .flex()
        .p(px(8.))
        .rounded_lg()
        .bg(rgb(t.row_alt_bg))
}

struct WorkspaceActions {
    pinned: bool,
    pin_id: String,
    remove_id: Option<String>,
}

fn workspace_entry_row(
    id: String,
    name: String,
    path: String,
    actions: Option<WorkspaceActions>,
    t: &Theme,
) -> AnyElement {
    let row_id = SharedString::from(id);
    let open_path = path.clone();
    let mut row = div()
        .id(row_id.clone())
        .debug_selector(move || row_id.to_string())
        .flex()
        .items_center()
        .min_w_0()
        .gap(px(6.))
        .pl(px(6.))
        .rounded_md()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(t.selected_bg)))
        .on_click(move |_, _, cx| {
            crate::repo::open_repo_window(PathBuf::from(&open_path), cx);
        })
        .child(icons::icon(icons::glyph::FOLDER, 10., t.fg_dim))
        .child(
            div()
                .flex_none()
                .text_size(ui_font_size(12.))
                .font_weight(FontWeight::MEDIUM)
                .child(name),
        )
        .child(
            div()
                .min_w_0()
                .flex_1()
                .truncate()
                .text_size(ui_font_size(10.))
                .text_color(rgb(t.fg_dim))
                .child(path.clone()),
        );
    if let Some(actions) = actions {
        row = row.children(repository_actions(
            path,
            actions.pinned,
            actions.pin_id,
            actions.remove_id,
            t,
        ));
    }
    row.into_any_element()
}
