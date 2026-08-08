use gpui::{
    AnyElement, App, AppContext, Bounds, ClickEvent, Context, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    px, rgb, size,
};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::divider_h;

mod guide;

use guide::{ShortcutEntry, ShortcutSection, display_key};

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
                        title: Some("Keyboard Shortcuts".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let [left, right] = guide::columns();
        div()
            .id("keyboard-shortcuts-window")
            .debug_selector(|| "keyboard-shortcuts-window".to_owned())
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
                    .child(column(left, &t))
                    .child(column(right, &t)),
            )
            .child(divider_h(&t))
            .child(
                div()
                    .px(px(20.))
                    .py(px(10.))
                    .text_size(px(11.))
                    .text_color(rgb(t.fg_faint))
                    .child("Esc closes the palette and sheets · Ctrl+N / Ctrl+P also move the selection"),
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
                .text_size(px(16.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(icons::icon(glyph::INFO, 15., t.fg_dim))
                .child("Keyboard Shortcuts"),
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

fn column(sections: &[ShortcutSection], t: &Theme) -> AnyElement {
    let mut column = div().flex().flex_col().gap(px(20.)).flex_1().min_w_0();
    for section in sections {
        column = column.child(section_block(section, t));
    }
    column.into_any_element()
}

fn section_block(section: &ShortcutSection, t: &Theme) -> AnyElement {
    let title = section.title;
    let mut block = div()
        .id(SharedString::from(format!("shortcut-section-{title}")))
        .debug_selector(move || format!("shortcut-section-{title}"))
        .flex()
        .flex_col()
        .gap(px(9.))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(t.fg_dim))
                .child(title.to_uppercase()),
        );
    for entry in section.entries {
        block = block.child(shortcut_row(entry, t));
    }
    block.into_any_element()
}

fn shortcut_row(entry: &ShortcutEntry, t: &Theme) -> AnyElement {
    let label = entry.label;
    let mut caps = div().flex().items_center().gap(px(4.));
    for (ix, key) in entry.keys.iter().enumerate() {
        caps = caps.child(key_cap(label, ix, display_key(key), t));
    }
    div()
        .id(SharedString::from(format!("shortcut-entry-{label}")))
        .debug_selector(move || format!("shortcut-entry-{label}"))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.))
        .text_size(px(13.))
        .child(div().min_w_0().child(label))
        .child(caps)
        .into_any_element()
}

fn key_cap(label: &'static str, ix: usize, key: &'static str, t: &Theme) -> AnyElement {
    div()
        .id(SharedString::from(format!("shortcut-key-{label}-{ix}")))
        .debug_selector(move || format!("shortcut-key-{label}-{ix}"))
        .min_w(px(18.))
        .px(px(6.))
        .py(px(3.))
        .rounded(px(5.))
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.row_alt_bg))
        .text_size(px(12.))
        .text_color(rgb(t.fg))
        .text_center()
        .child(key)
        .into_any_element()
}
