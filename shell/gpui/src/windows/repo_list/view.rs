use gpui::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, MouseButton, MouseMoveEvent,
    MouseUpEvent, ParentElement, Render, Stateful, StatefulInteractiveElement, Styled, Window, div,
    px, rgb,
};

use super::panel::PanelFrame;
use super::window::{DETAIL_WIDTH, RepoListWindow};
use super::{header, sections};
use crate::app::actions::CloseWindow;
use crate::app::config;
use crate::app::repositories;
use crate::app::theme::Theme;
use crate::platform::TOOLBAR_LEADING_INSET;
use crate::ui::icons::glyph;
use crate::ui::primitives::{divider_h, icon_button, text_tooltip};
use crate::ui::resize_handle::resize_handle;

const TOP_INSET: f32 = 38.;

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
            let panel = self
                .panel_frame(panel_shown, window, cx)
                .map(|frame| self.recent_panel(&pinned, frame, &t, cx));
            div()
                .relative()
                .flex()
                .flex_1()
                .flex_row()
                .min_h_0()
                .children(panel.into_iter().flatten())
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
            .on_mouse_move(cx.listener(|view, ev: &MouseMoveEvent, window, cx| {
                let viewport_width = f32::from(window.viewport_size().width);
                view.drag_panel_to(f32::from(ev.position.x), viewport_width, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|view, _: &MouseUpEvent, _, cx| view.end_panel_drag(cx)),
            )
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(content)
    }
}

impl RepoListWindow {
    fn recent_panel(
        &self,
        pinned: &[String],
        frame: PanelFrame,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> [AnyElement; 2] {
        let panel = div()
            .flex_none()
            .overflow_hidden()
            .w(px(frame.visible))
            .child(
                div()
                    .debug_selector(|| "repo-list-recent-panel".to_owned())
                    .flex()
                    .flex_col()
                    .w(px(frame.full))
                    .h_full()
                    .ml(px(frame.visible - frame.full))
                    .pt(px(TOP_INSET))
                    .bg(rgb(t.header_bg))
                    .child(
                        scroll_column("repo-list-recent-scroll").child(
                            sections::recent_section(self.groups.recent.clone(), pinned, t)
                                .px(px(14.))
                                .py(px(18.)),
                        ),
                    ),
            );
        let handle = resize_handle(
            "repo-list-recent-resize-handle",
            t,
            |view: &mut Self, x, viewport_width, cx| view.start_panel_drag(x, viewport_width, cx),
            cx,
        );
        [panel.into_any_element(), handle]
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
