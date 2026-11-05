use super::view::ShortcutSettings;
use crate::app::config;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};
use gpui::{
    Context, Div, InteractiveElement, ParentElement, Stateful, Styled, Window, div, px, rgb,
};
use jayjay_i18n::shortcuts;
use std::rc::Rc;

const SEGMENT_WIDTH: f32 = 86.;

impl ShortcutSettings {
    pub(super) fn filter_bar(&self, t: &Theme, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .items_center()
            .gap(px(10.))
            .child(
                div()
                    .id("shortcuts-query")
                    .debug_selector(|| "shortcuts-query".to_owned())
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .child(self.query.clone())
                    .child(
                        div()
                            .absolute()
                            .left(px(9.))
                            .top_0()
                            .bottom_0()
                            .flex()
                            .items_center()
                            .child(icons::icon(glyph::SEARCH, 13., t.fg_dim)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .gap(px(1.))
                    .p(px(2.))
                    .rounded_md()
                    .bg(rgb(t.toggle_inactive_bg))
                    .child(self.segment("shortcuts-all", shortcuts::all(), false, t, cx))
                    .child(self.segment("shortcuts-modified", shortcuts::modified(), true, t, cx)),
            )
    }

    fn segment(
        &self,
        id: &'static str,
        label: String,
        modified_only: bool,
        t: &Theme,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let (bg, fg) = if self.modified_only == modified_only {
            (t.toggle_active_bg, t.toggle_active_fg)
        } else {
            (t.toggle_inactive_bg, t.toggle_inactive_fg)
        };
        Self::control(
            id.into(),
            label.clone(),
            t,
            cx,
            Rc::new(
                move |this: &mut Self, _: &mut Window, cx: &mut Context<Self>| {
                    this.modified_only = modified_only;
                    cx.notify();
                },
            ),
        )
        .w(px(SEGMENT_WIDTH))
        .py(px(3.))
        .flex()
        .justify_center()
        .rounded_md()
        .border_color(gpui::transparent_black())
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .child(label)
    }

    pub(super) fn reset_all_button(t: &Theme, cx: &Context<Self>) -> Stateful<Div> {
        Self::button(
            "shortcuts-reset-all",
            shortcuts::reset_all(),
            t,
            cx,
            |this, _, cx| {
                let mut overrides = config::current(cx).shortcut_overrides();
                overrides.reset_all();
                Self::save(&overrides, cx);
                this.cancel(cx);
            },
        )
    }
}
