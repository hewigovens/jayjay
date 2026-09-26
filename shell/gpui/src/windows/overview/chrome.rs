use gpui::{
    AnyElement, App, ClickEvent, Context, Focusable, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use gpui::{AppContext, Entity};
use jayjay_core::overview::OverviewSnapshot;

use super::OverviewView;
use super::actions::Confirmation;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::overlay::{confirmation_card, overlay_actions, overlay_layer};
use crate::ui::primitives::{button, icon_button};
use crate::ui::text_area::TextArea;

pub(super) struct LaneFilter {
    pub(super) input: Entity<TextArea>,
    pub(super) shown: bool,
}

impl LaneFilter {
    pub(super) fn new(cx: &mut Context<OverviewView>) -> Self {
        let input = cx.new(|cx| TextArea::new("", "Filter lanes", false, 26., cx));
        TextArea::subscribe_updates(&input, cx);
        Self {
            input,
            shown: false,
        }
    }
}

impl OverviewView {
    pub(super) fn filter_focused(&self, window: &Window, cx: &App) -> bool {
        self.filter.input.focus_handle(cx).is_focused(window)
    }

    pub(super) fn show_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter.shown = true;
        window.focus(&self.filter.input.focus_handle(cx), cx);
        cx.notify();
    }

    pub(super) fn submit_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.filter.input.read(cx).text().is_empty() {
            self.filter.shown = false;
        }
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    pub(super) fn dismiss_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter.input.update(cx, |filter, cx| filter.clear(cx));
        self.filter.shown = false;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    pub(super) fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.menu.take().is_some() || self.confirmation.take().is_some() {
            cx.notify();
        } else if self.filter_focused(window, cx) {
            self.dismiss_filter(window, cx);
        } else if self.clear_change() || std::mem::take(&mut self.selection.lane_panel) {
            cx.notify();
        } else {
            window.remove_window();
        }
    }

    pub(super) fn header(
        &self,
        snapshot: Option<&OverviewSnapshot>,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let lanes = snapshot.map_or(&[][..], |snapshot| snapshot.overview.lanes.as_slice());
        let changes: usize = lanes.iter().map(|lane| lane.changes.len()).sum();
        let behind = lanes.iter().filter(|lane| lane.is_behind_trunk()).count();
        let attention = lanes
            .iter()
            .filter(|lane| !lane.attention.is_empty())
            .count();
        let workspaces = snapshot.map_or(0, |snapshot| snapshot.overview.workspace_count as usize);
        let error = self
            .action_error
            .clone()
            .or_else(|| snapshot.and(self.load.error.clone()));
        let filter: AnyElement = if self.filter.shown {
            div()
                .id("overview-filter")
                .debug_selector(|| "overview-filter".to_owned())
                .w(px(220.))
                .child(self.filter.input.clone())
                .into_any_element()
        } else {
            icon_button(
                "overview-filter-toggle",
                glyph::SEARCH,
                14.,
                26.,
                26.,
                t.fg_dim,
                t,
            )
            .debug_selector(|| "overview-filter-toggle".to_owned())
            .on_click(cx.listener(|view, _: &ClickEvent, window, cx| view.show_filter(window, cx)))
            .into_any_element()
        };
        div()
            .flex()
            .flex_none()
            .flex_row()
            .items_center()
            .gap(px(16.))
            .px(px(14.))
            .h(px(40.))
            .bg(rgb(t.header_bg))
            .border_b_1()
            .border_color(rgb(t.border))
            .child(stat(lanes.len(), "lane", "lanes", None, t))
            .child(stat(changes, "change", "changes", None, t))
            .child(stat(workspaces, "workspace", "workspaces", None, t))
            .child(stat(
                behind,
                "behind trunk",
                "behind trunk",
                (behind > 0).then_some(t.warning_fg),
                t,
            ))
            .child(stat(
                attention,
                "needs attention",
                "need attention",
                (attention > 0).then_some(t.warning_fg),
                t,
            ))
            .child(div().flex_1())
            .children(error.map(|error| {
                div()
                    .min_w_0()
                    .text_size(ui_font_size(11.))
                    .text_color(rgb(t.error_fg))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(error)
            }))
            .child(filter)
            .into_any_element()
    }

    pub(super) fn confirmation_overlay(
        &self,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (title, message, label) = match self.confirmation.as_ref()? {
            Confirmation::Abandon(request) => (
                format!("Abandon \u{201C}{}\u{201D}?", request.title),
                request.message(),
                "Abandon",
            ),
            Confirmation::DeleteWorkspace { name, path } => (
                format!("Delete Workspace {name}?"),
                format!(
                    "This closes its window, forgets the workspace, and deletes its directory from disk:\n{path}"
                ),
                "Delete",
            ),
        };
        Some(
            overlay_layer()
                .child(
                    confirmation_card(title, message, t).child(overlay_actions(
                        button("overview-confirm-cancel", "Cancel", t, false).on_click(
                            cx.listener(|view, _: &ClickEvent, _, cx| {
                                view.confirmation = None;
                                cx.notify();
                            }),
                        ),
                        button("overview-confirm-submit", label, t, true)
                            .debug_selector(|| "overview-confirm-submit".to_owned())
                            .on_click(cx.listener(|view, _: &ClickEvent, _, cx| view.confirm(cx))),
                    )),
                )
                .into_any_element(),
        )
    }
}

fn stat(count: usize, singular: &str, plural: &str, tint: Option<u32>, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .gap(px(4.))
        .text_size(ui_font_size(11.))
        .text_color(rgb(tint.unwrap_or(t.fg_dim)))
        .child(
            div()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(SharedString::from(count.to_string())),
        )
        .child(SharedString::from(
            if count == 1 { singular } else { plural }.to_owned(),
        ))
        .into_any_element()
}
