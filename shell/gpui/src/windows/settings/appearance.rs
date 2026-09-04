use crate::app::config::{AppConfig, AppearanceMode, ShortcutModifier};
use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::SettingsView;
use super::dropdown::dropdown_button;
use super::shared::{current_value, field_row, section_title, subsection_title};
use crate::app::config;
use crate::app::fonts;
use crate::app::theme::Theme;

pub(super) fn appearance_section(
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut gpui::Context<SettingsView>,
) -> AnyElement {
    let mut section = div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(section_title("Appearance", t))
        .child(field_row(
            "Theme",
            appearance_segmented(cfg.appearance, t),
            "Light, Dark, or System.",
            t,
        ))
        .child(subsection_title("Font", t))
        .child(field_row(
            "Family",
            dropdown_button(
                "font-family",
                fonts::mono_preference_label(&cfg.font_family),
                t,
                cx,
            ),
            "Monospace font for diff and code.",
            t,
        ))
        .child(field_row(
            "Size",
            font_size_value(cfg.font_size, t),
            "Used by diff and editors.",
            t,
        ));
    if cfg!(target_os = "linux") {
        section = section
            .child(subsection_title("Keyboard", t))
            .child(field_row(
                "Modifier",
                modifier_segmented(cfg.shortcut_modifier, t),
                "Combinations the window manager reserves never reach JayJay.",
                t,
            ));
    }
    section.into_any_element()
}

fn font_size_value(size: f32, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .child(current_value(&format!("{size:.0}pt"), t))
        .into_any_element()
}

fn appearance_segmented(current: AppearanceMode, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .gap(px(4.))
        .child(segmented_option(
            AppearanceMode::System,
            "System",
            current,
            "appearance-system",
            |c, mode| c.appearance = mode,
            t,
        ))
        .child(segmented_option(
            AppearanceMode::Light,
            "Light",
            current,
            "appearance-light",
            |c, mode| c.appearance = mode,
            t,
        ))
        .child(segmented_option(
            AppearanceMode::Dark,
            "Dark",
            current,
            "appearance-dark",
            |c, mode| c.appearance = mode,
            t,
        ))
        .into_any_element()
}

fn modifier_segmented(current: ShortcutModifier, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .gap(px(4.))
        .child(segmented_option(
            ShortcutModifier::Ctrl,
            "Ctrl",
            current,
            "shortcut-modifier-ctrl",
            |c, modifier| c.shortcut_modifier = modifier,
            t,
        ))
        .child(segmented_option(
            ShortcutModifier::Super,
            "Super",
            current,
            "shortcut-modifier-super",
            |c, modifier| c.shortcut_modifier = modifier,
            t,
        ))
        .into_any_element()
}

fn segmented_option<T: Copy + PartialEq + 'static>(
    value: T,
    label: &'static str,
    current: T,
    id: &'static str,
    select: fn(&mut AppConfig, T),
    t: &Theme,
) -> AnyElement {
    let active = value == current;
    let (bg, fg) = if active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    div()
        .id(SharedString::from(id))
        .debug_selector(move || id.to_owned())
        .px(px(10.))
        .py(px(3.))
        .rounded_md()
        .bg(rgb(bg))
        .text_size(px(11.))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(move |_ev: &ClickEvent, _w, cx| {
            config::update(cx, move |c| select(c, value));
        })
        .child(label)
        .into_any_element()
}
