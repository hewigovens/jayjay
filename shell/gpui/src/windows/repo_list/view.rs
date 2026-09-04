use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb,
};

use super::window::RepoListWindow;
use super::{header, sections};
use crate::app::actions::CloseWindow;
use crate::app::config;
use crate::app::repositories;
use crate::ui::primitives::divider_h;

impl Render for RepoListWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let pinned = repositories::current(cx);
        let recent = config::current(cx).recent_repos.clone();
        let has_repositories = !pinned.is_empty() || !recent.is_empty();
        self.show(pinned.clone(), recent, cx);
        let content = if has_repositories {
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .flex_col()
                .child(
                    div()
                        .px(px(30.))
                        .pt(px(30.))
                        .pb(px(22.))
                        .child(header::header(&self.logo, &t)),
                )
                .child(divider_h(&t))
                .child(sections::repository_sections(
                    self.groups.clone(),
                    &pinned,
                    &t,
                ))
        } else {
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .child(header::header(&self.logo, &t))
        };

        div()
            .id("repo-list-window")
            .debug_selector(|| "repo-list-window".to_owned())
            .track_focus(&self.focus_handle)
            .key_context("RepoListWindow")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _| {
                window.remove_window();
            }))
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(content)
    }
}
