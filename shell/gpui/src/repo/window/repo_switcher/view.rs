use gpui::{AnyElement, Entity, MouseDownEvent};
use jayjay_core::WorkspaceInfo;

use super::model::RepoSwitcherState;
use super::rows::switcher_row;
use super::sections::switcher_sections;
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::repo::window::picker::{self, picker_items};
use crate::ui::icons::glyph;

pub(crate) fn render_repo_switcher(
    state: &RepoSwitcherState,
    workspaces: &[WorkspaceInfo],
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let close_view = view.clone();
    picker::overlay(
        "repo-switcher-backdrop",
        state.anchor,
        menu_panel(state, workspaces, t, view),
        move |_: &MouseDownEvent, _, cx| {
            close_view.update(cx, |view, cx| view.close_repo_switcher(cx));
        },
    )
}

fn menu_panel(
    state: &RepoSwitcherState,
    workspaces: &[WorkspaceInfo],
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let overview_view = view.clone();
    let new_view = view.clone();
    let header = picker::header(
        "repo-switcher-filter",
        &state.query,
        [
            picker::header_button(
                "repo-switcher-overview",
                glyph::COLUMNS,
                "Overview",
                t,
                move |_, cx| {
                    overview_view.update(cx, |view, cx| {
                        view.close_repo_switcher(cx);
                        view.open_overview(cx);
                    });
                },
            ),
            picker::header_button(
                "repo-switcher-new",
                glyph::PLUS_CIRCLE,
                "New",
                t,
                move |_, cx| {
                    new_view.update(cx, |view, cx| {
                        view.close_repo_switcher(cx);
                        view.open_create_workspace(cx);
                    });
                },
            ),
        ],
        t,
    );

    let row_view = view.clone();
    picker::panel(
        "repo-switcher-panel",
        440.,
        header,
        picker_items(
            switcher_sections(state, workspaces),
            state.query.selected,
            "No matches",
        ),
        &state.query,
        t,
        move |row, selected, t| switcher_row(row, selected, t, &row_view),
    )
}
