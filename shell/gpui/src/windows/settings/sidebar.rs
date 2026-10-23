use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::{SettingsSection, SettingsView};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;
use crate::ui::resizable_pane::ResizablePane;

pub(crate) const SETTINGS_SIDEBAR_DEFAULT: f32 = 230.;
const SIDEBAR_MIN: f32 = 160.;
const SIDEBAR_MAX: f32 = 360.;
const CONTENT_MIN: f32 = 420.;

pub(super) fn sidebar(
    active: SettingsSection,
    width: f32,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .id("settings-sidebar")
        .flex()
        .flex_col()
        .flex_none()
        .w(px(width))
        .h_full()
        .overflow_y_scroll()
        .bg(rgb(t.row_alt_bg))
        .px(px(8.))
        .py(px(12.))
        .children(SettingsSection::ALL.into_iter().map(|section| {
            div()
                .id(SharedString::from(format!("settings-nav-{section:?}")))
                .debug_selector(move || format!("settings-nav-{section:?}"))
                .flex()
                .flex_none()
                .items_center()
                .gap(px(8.))
                .px(px(8.))
                .py(px(8.))
                .rounded(px(6.))
                .bg(rgb(if section == active {
                    t.selected_bg
                } else {
                    t.row_alt_bg
                }))
                .text_size(ui_font_size(13.))
                .text_color(rgb(t.fg))
                .cursor_pointer()
                .hover(|s| {
                    s.bg(rgb(if section == active {
                        t.selected_bg
                    } else {
                        t.toggle_inactive_bg
                    }))
                })
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    if this.section == section {
                        return;
                    }
                    this.section = section;
                    this.open_dropdown = None;
                    this.fonts.pending_dropdown = None;
                    this.scroll.set_offset(Default::default());
                    match section {
                        SettingsSection::Fonts => this.ensure_font_catalog_loaded(cx),
                        SettingsSection::Integrations => this.ensure_tools_loaded(cx),
                        SettingsSection::Jujutsu => this.ensure_jj_config_loaded(cx),
                        SettingsSection::DataPrivacy => this.refresh_review_summary(cx),
                        _ => {}
                    }
                    cx.notify();
                }))
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(px(20.))
                        .rounded(px(5.))
                        .bg(rgb(section.color(t)))
                        .child(icons::icon(section.glyph(), 12., t.detail_bg)),
                )
                .child(div().min_w_0().truncate().child(section.label()))
        }))
        .into_any_element()
}

impl SettingsView {
    pub(super) fn sidebar_pane(cx: &App) -> ResizablePane {
        let width = crate::app::config::current(cx)
            .layout
            .settings_sidebar_width;
        ResizablePane::new(width, SIDEBAR_MIN, SIDEBAR_MAX, CONTENT_MIN)
    }

    pub(super) fn start_sidebar_drag(
        &mut self,
        x: f32,
        viewport_width: f32,
        cx: &mut Context<Self>,
    ) {
        self.sidebar.start_drag(x, viewport_width);
        cx.notify();
    }

    pub(super) fn drag_sidebar_to(&mut self, x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        if self.sidebar.drag_to(x, viewport_width) {
            cx.notify();
        }
    }

    pub(super) fn end_sidebar_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(width) = self.sidebar.end_drag() {
            crate::app::config::update(cx, move |c| c.layout.settings_sidebar_width = width);
            cx.notify();
        }
    }
}
