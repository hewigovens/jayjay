use std::time::{Duration, Instant};

use gpui::{
    Context, Div, InteractiveElement, IntoElement, ParentElement, Render, Stateful,
    StatefulInteractiveElement, Styled, Window, div, ease_in_out, px, rgb,
};

use super::window::{DETAIL_WIDTH, RepoListWindow};
use super::{header, sections};
use crate::app::actions::CloseWindow;
use crate::app::config;
use crate::app::repositories;
use crate::app::theme::Theme;
use crate::platform::TOOLBAR_LEADING_INSET;
use crate::ui::icons::glyph;
use crate::ui::primitives::{divider_h, icon_button, text_tooltip};

const PANEL_WIDTH: f32 = 270.;
const PANEL_SLIDE: Duration = Duration::from_millis(180);
const TOP_INSET: f32 = 38.;

#[derive(Clone, Copy)]
pub(super) struct PanelSlide {
    started: Instant,
    from: f32,
    to: f32,
}

impl PanelSlide {
    fn width_at(&self, now: Instant) -> f32 {
        let progress = (now - self.started).as_secs_f32() / PANEL_SLIDE.as_secs_f32();
        self.from + (self.to - self.from) * ease_in_out(progress.min(1.))
    }
}

impl Render for RepoListWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let pinned = repositories::current(cx);
        let cfg = config::current(cx);
        self.show(pinned.clone(), cfg.recent_repos.clone(), cx);
        let content = if let Some(onboarding) = self.onboarding.as_ref() {
            div().flex().flex_1().min_h_0().child(onboarding.clone())
        } else {
            let panel_shown = cfg.layout.recent_repos_panel;
            let panel_width = self.panel_width(panel_shown, window, cx);
            div()
                .relative()
                .flex()
                .flex_1()
                .flex_row()
                .min_h_0()
                .children(panel_width.map(|width| self.recent_panel(&pinned, width, &t)))
                .child(self.detail(&pinned, &t))
                .child(panel_toggle(panel_shown, &t))
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

impl RepoListWindow {
    fn panel_width(
        &mut self,
        shown: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<f32> {
        let now = cx.background_executor().now();
        let to = if shown { PANEL_WIDTH } else { 0. };
        if self
            .panel_shown
            .replace(shown)
            .is_some_and(|was| was != shown)
            && !cx.reduce_motion()
        {
            let from = self
                .panel_slide
                .map_or(PANEL_WIDTH - to, |slide| slide.width_at(now));
            self.panel_slide = Some(PanelSlide {
                started: now,
                from,
                to,
            });
        }
        if self
            .panel_slide
            .is_some_and(|slide| now - slide.started >= PANEL_SLIDE)
        {
            self.panel_slide = None;
        }
        match self.panel_slide {
            Some(slide) => {
                window.request_animation_frame();
                Some(slide.width_at(now))
            }
            None => shown.then_some(PANEL_WIDTH),
        }
    }

    fn recent_panel(&self, pinned: &[String], width: f32, t: &Theme) -> Div {
        div().flex_none().overflow_hidden().w(px(width)).child(
            div()
                .debug_selector(|| "repo-list-recent-panel".to_owned())
                .flex()
                .flex_col()
                .w(px(PANEL_WIDTH))
                .h_full()
                .ml(px(width - PANEL_WIDTH))
                .pt(px(TOP_INSET))
                .bg(rgb(t.header_bg))
                .border_r_1()
                .border_color(rgb(t.border))
                .child(
                    scroll_column("repo-list-recent-scroll").child(
                        sections::recent_section(self.groups.recent.clone(), pinned, t)
                            .px(px(14.))
                            .py(px(18.)),
                    ),
                ),
        )
    }

    fn detail(&self, pinned: &[String], t: &Theme) -> Div {
        let column = div()
            .flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex_col()
            .pt(px(TOP_INSET));
        let content = div()
            .debug_selector(|| "repo-list-detail".to_owned())
            .w_full()
            .max_w(px(DETAIL_WIDTH))
            .flex()
            .flex_col();
        if self.groups.pinned.is_empty() {
            return column
                .items_center()
                .justify_center()
                .child(content.px(px(30.)).child(header::header(&self.logo, t)));
        }
        column.child(
            scroll_column("repo-list-scroll").items_center().child(
                content
                    .child(
                        div()
                            .px(px(30.))
                            .pt(px(14.))
                            .pb(px(22.))
                            .child(header::header(&self.logo, t)),
                    )
                    .child(divider_h(t))
                    .child(
                        sections::repository_section(
                            "Pinned",
                            self.groups.pinned.clone(),
                            sections::RowKind::Pinned,
                            pinned,
                            t,
                        )
                        .px(px(30.))
                        .py(px(18.)),
                    ),
            ),
        )
    }
}

fn panel_toggle(panel_shown: bool, t: &Theme) -> Stateful<Div> {
    icon_button(
        "repo-list-toggle-recent",
        glyph::COLUMNS,
        14.,
        28.,
        24.,
        t.fg_dim,
        t,
    )
    .debug_selector(|| "repo-list-toggle-recent".to_owned())
    .absolute()
    .top(px(8.))
    .left(px(TOOLBAR_LEADING_INSET))
    .tooltip(text_tooltip(if panel_shown {
        "Hide Recent Repositories"
    } else {
        "Show Recent Repositories"
    }))
    .on_click(|_, _, cx| {
        config::update(cx, |cfg| {
            cfg.layout.recent_repos_panel = !cfg.layout.recent_repos_panel;
        });
    })
}

fn scroll_column(id: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .flex_1()
        .min_h_0()
        .flex_col()
        .overflow_y_scroll()
        .scrollbar_width(px(0.))
}
