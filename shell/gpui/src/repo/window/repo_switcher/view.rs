use gpui::{AnyElement, Entity, MouseDownEvent};
use jayjay_core::WorkspaceInfo;

use super::model::RepoSwitcherState;
use super::rows::switcher_row;
use super::sections::switcher_sections;
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::repo::window::picker::{self, render_sections};
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
    let refresh_view = view.clone();
    let new_view = view.clone();
    let header = picker::header(
        "repo-switcher-filter",
        &state.query,
        [
            picker::header_button(
                "repo-switcher-refresh",
                glyph::ARROW_CLOCKWISE,
                "Refresh",
                t,
                move |window, cx| {
                    let current_window = window.window_handle();
                    refresh_view.update(cx, |view, cx| {
                        view.refresh_repo_switcher(current_window, cx);
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

    let sections = switcher_sections(state, workspaces);
    let rows = if sections.is_empty() {
        vec![picker::empty("No matches", t)]
    } else {
        render_sections(sections, state.query.selected, t, |row, selected| {
            switcher_row(row, selected, t, view)
        })
    };
    picker::panel(
        "repo-switcher-panel",
        360.,
        header,
        rows,
        &state.query.scroll,
        t,
    )
}
