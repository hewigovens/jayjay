use gpui::{
    AnyElement, App, AppContext, Bounds, ClickEvent, Context, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    px, rgb, size,
};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, observe_window_appearance, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::divider_h;

mod guide;

use guide::{ShortcutEntry, ShortcutSection};

pub struct KeyboardShortcutsView {
    focus_handle: FocusHandle,
}

impl KeyboardShortcutsView {
    pub fn open(cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(720.), px(560.)), cx);
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(jayjay_i18n::menu::keyboard_shortcuts().into()),
                        ..Default::default()
                    }),
                    ..crate::app::window_options()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        Self {
                            focus_handle: cx.focus_handle(),
                        }
                    })
                },
            )
            .ok();
        if let Some(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                window.focus(&view.focus_handle, cx);
            });
        }
    }
}

impl Focusable for KeyboardShortcutsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for KeyboardShortcutsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let [left, right] = guide::columns();
        div()
            .id("keyboard-shortcuts-window")
            .debug_selector(|| "keyboard-shortcuts-window".to_owned())
            .font_family(crate::app::fonts::interface())
            .track_focus(&self.focus_handle)
            .key_context("KeyboardShortcutsView")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _| window.remove_window()))
            .on_action(cx.listener(|_, _: &Dismiss, window, _| window.remove_window()))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(header(&t, cx))
            .child(divider_h(&t))
            .child(
                div()
                    .id("keyboard-shortcuts-scroll")
                    .debug_selector(|| "keyboard-shortcuts-scroll".to_owned())
                    .flex()
                    .flex_row()
                    .gap(px(32.))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .scrollbar_width(px(0.))
                    .p(px(20.))
                    .child(column(left, &t, cx))
                    .child(column(right, &t, cx)),
            )
            .child(divider_h(&t))
            .child(
                div()
                    .px(px(20.))
                    .py(px(10.))
                    .text_size(ui_font_size(11.))
                    .text_color(rgb(t.fg_faint))
                    .child(jayjay_i18n::shortcuts::esc_closes_the_palette_and_sheets()),
            )
    }
}

fn header(t: &Theme, cx: &mut Context<KeyboardShortcutsView>) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .px(px(20.))
        .py(px(14.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .text_size(ui_font_size(16.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(icons::icon(glyph::INFO, 15., t.fg_dim))
                .child(jayjay_i18n::menu::keyboard_shortcuts()),
        )
        .child(
            div()
                .id("keyboard-shortcuts-close")
                .debug_selector(|| "keyboard-shortcuts-close".to_owned())
                .flex()
                .items_center()
                .justify_center()
                .size(px(26.))
                .rounded_full()
                .bg(rgb(t.row_alt_bg))
                .cursor_pointer()
                .on_click(cx.listener(|_, _: &ClickEvent, window, _| window.remove_window()))
                .child(icons::icon(glyph::X, 11., t.fg_dim)),
        )
        .into_any_element()
}

fn column(sections: &[ShortcutSection], t: &Theme, cx: &App) -> AnyElement {
    let mut column = div().flex().flex_col().gap(px(20.)).flex_1().min_w_0();
    for section in sections {
        column = column.child(section_block(section, t, cx));
    }
    column.into_any_element()
}

fn section_block(section: &ShortcutSection, t: &Theme, cx: &App) -> AnyElement {
    let id = section.id;
    let mut block = div()
        .id(SharedString::from(format!("shortcut-section-{id}")))
        .debug_selector(move || format!("shortcut-section-{id}"))
        .flex()
        .flex_col()
        .gap(px(9.))
        .child(
            div()
                .text_size(ui_font_size(12.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(t.fg_dim))
                .child((section.title)().to_uppercase()),
        );
    for (ix, entry) in section.entries.iter().enumerate() {
        block = block.child(shortcut_row(&format!("{id}-{ix}"), entry, t, cx));
    }
    block.into_any_element()
}

fn shortcut_row(id: &str, entry: &ShortcutEntry, t: &Theme, cx: &App) -> AnyElement {
    let label = (entry.label)();
    let mut caps = div().flex().items_center().gap(px(4.));
    for (ix, key) in entry.key_caps(cx).into_iter().enumerate() {
        caps = caps.child(key_cap(id, ix, key, t));
    }
    let selector = format!("shortcut-entry-{id}");
    div()
        .id(SharedString::from(selector.clone()))
        .debug_selector(move || selector)
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.))
        .text_size(ui_font_size(13.))
        .child(div().min_w_0().child(label))
        .child(caps)
        .into_any_element()
}

fn key_cap(entry_id: &str, ix: usize, key: String, t: &Theme) -> AnyElement {
    let selector = format!("shortcut-key-{entry_id}-{ix}");
    div()
        .id(SharedString::from(selector.clone()))
        .debug_selector(move || selector)
        .min_w(px(18.))
        .px(px(6.))
        .py(px(3.))
        .rounded(px(5.))
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.row_alt_bg))
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg))
        .text_center()
        .child(key)
        .into_any_element()
}
