use gpui::{
    AnyElement, App, AppContext, Bounds, ClickEvent, Context, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled,
    TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, rgb, size,
};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::config::{self, AppConfigStore};
use crate::app::theme::{Theme, observe_window_appearance, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::divider_h;
use crate::windows::settings::{SettingsSection, SettingsView};
use jayjay_core::shortcuts::{EffectiveBindings, KeyPlatform};

mod entry;
mod section;

use section::ShortcutSection;

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
        let bindings = EffectiveBindings::resolve(
            &config::current(cx).shortcut_overrides(),
            KeyPlatform::current(),
        );
        let [left, right] = ShortcutSection::columns(&bindings);
        div()
            .id("keyboard-shortcuts-window")
            .debug_selector(|| "keyboard-shortcuts-window".to_owned())
            .font_family(crate::app::fonts::interface())
            .track_focus(&self.focus_handle)
            .key_context("KeyboardShortcutsView")
            .on_key_down(|event, window, cx| {
                if event.keystroke.key == "tab" && !event.keystroke.modifiers.modified() {
                    window.focus_next(cx);
                    cx.stop_propagation();
                }
            })
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
                    .child(ShortcutSection::column(left, &bindings, &t, cx))
                    .child(ShortcutSection::column(right, &bindings, &t, cx)),
            )
            .child(
                crate::ui::primitives::button(
                    "shortcuts-customize",
                    jayjay_i18n::shortcuts::customize(),
                    &t,
                    false,
                )
                .focusable()
                .tab_stop(true)
                .on_key_down(|event, _, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space")
                        && !event.keystroke.modifiers.modified()
                    {
                        SettingsView::open_section(SettingsSection::KeyboardShortcuts, cx);
                        cx.stop_propagation();
                    }
                })
                .on_click(|_, _, cx| {
                    SettingsView::open_section(SettingsSection::KeyboardShortcuts, cx)
                }),
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
