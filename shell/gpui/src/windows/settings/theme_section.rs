use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, WindowAppearance, div, px, rgb,
};

use super::SettingsView;
use super::font_preview::theme_preview;
use super::shared::{DETAIL_FONT_SIZE, ROW_FONT_SIZE, field_row, form_section};
use crate::app::config::{self, AppConfig, AppearanceMode};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::navigation::{ListNavKeys, list_nav_from_key, move_index};
use jayjay_core::{ColorTheme, color_themes};
use jayjay_i18n::settings;

pub(super) fn theme_section(
    view: &SettingsView,
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let edited_dark = view.edited_theme_dark.unwrap_or(t.is_dark);
    let themes: Vec<ColorTheme> = color_themes()
        .into_iter()
        .filter(|theme| theme.is_dark == edited_dark)
        .collect();
    let current_id = cfg.diff.palette.resolve(edited_dark).id;
    let preview = preview_theme(cfg, edited_dark);
    let footnote = if edited_dark {
        settings::theme_used_dark()
    } else {
        settings::theme_used_light()
    };

    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(8.))
        .child(form_section(
            settings::colors(),
            [
                field_row(
                    glyph::CONTRAST,
                    settings::theme(),
                    theme_side_segmented(edited_dark, t, cx),
                    t,
                ),
                theme_picker_and_preview(&themes, &current_id, edited_dark, &preview, t, cx),
            ],
            t,
        ))
        .child(
            div()
                .px(px(12.))
                .text_size(ui_font_size(DETAIL_FONT_SIZE))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(footnote)),
        )
        .into_any_element()
}

fn theme_side_segmented(
    edited_dark: bool,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .gap(px(1.))
        .p(px(2.))
        .rounded_md()
        .bg(rgb(t.toggle_inactive_bg))
        .child(theme_side_option(
            false,
            settings::appearance_light(),
            !edited_dark,
            "theme-side-light",
            t,
            cx,
        ))
        .child(theme_side_option(
            true,
            settings::appearance_dark(),
            edited_dark,
            "theme-side-dark",
            t,
            cx,
        ))
        .into_any_element()
}

fn theme_side_option(
    is_dark: bool,
    label: String,
    active: bool,
    id: &'static str,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
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
        .text_size(ui_font_size(ROW_FONT_SIZE))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(cx.listener(move |view, _: &ClickEvent, _w, cx| {
            view.edited_theme_dark = Some(is_dark);
            cx.notify();
        }))
        .child(label)
        .into_any_element()
}

fn theme_picker_and_preview(
    themes: &[ColorTheme],
    current_id: &str,
    edited_dark: bool,
    preview: &Theme,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let rows = themes
        .iter()
        .map(|theme| theme_row(theme, theme.id == current_id, edited_dark, t));

    div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .py(px(10.))
        .gap(px(8.))
        .child(
            div()
                .id("settings-theme-list")
                .debug_selector(|| "settings-theme-list".to_owned())
                .flex()
                .flex_col()
                .w_full()
                .p(px(4.))
                .gap(px(2.))
                .rounded(px(8.))
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.detail_bg))
                .children(rows),
        )
        .child(theme_preview(preview, cx))
        .into_any_element()
}

fn theme_row(theme: &ColorTheme, is_selected: bool, edited_dark: bool, t: &Theme) -> AnyElement {
    let id = theme.id.clone();
    let selector = format!("theme-row-{id}");
    let row_bg = if is_selected {
        t.selected_bg
    } else {
        t.detail_bg
    };
    let checkmark = if is_selected {
        icons::icon(glyph::CHECK, 13., t.selected_accent).into_any_element()
    } else {
        div().w(px(13.)).into_any_element()
    };

    div()
        .id(SharedString::from(selector.clone()))
        .debug_selector(move || selector.clone())
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(t.scaled_control_height(30., ROW_FONT_SIZE)))
        .px(px(8.))
        .gap(px(8.))
        .rounded(px(6.))
        .bg(rgb(row_bg))
        .text_size(ui_font_size(ROW_FONT_SIZE))
        .text_color(rgb(t.fg))
        .cursor_pointer()
        .hover(move |s| {
            s.bg(rgb(if is_selected {
                t.selected_bg
            } else {
                t.row_alt_bg
            }))
        })
        .on_click(move |_: &ClickEvent, _w, cx| {
            let value = id.clone();
            config::update(cx, move |c| {
                if edited_dark {
                    c.diff.palette.dark = value.clone();
                } else {
                    c.diff.palette.light = value.clone();
                }
            });
        })
        .child(theme_swatch_tile(theme, t))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .child(SharedString::from(theme.name.clone())),
        )
        .child(checkmark)
        .into_any_element()
}

fn theme_swatch_tile(theme: &ColorTheme, t: &Theme) -> AnyElement {
    let bar = |color: u32| div().w(px(3.)).h(px(10.)).rounded(px(1.)).bg(rgb(color));
    div()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(2.))
        .w(px(22.))
        .h(px(16.))
        .rounded(px(4.))
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(theme.seed.background))
        .child(bar(theme.repo.graph_line))
        .child(bar(theme.colors.gutter_added_fg))
        .child(bar(theme.colors.gutter_removed_fg))
        .into_any_element()
}

fn preview_theme(cfg: &AppConfig, is_dark: bool) -> Theme {
    let mode = if is_dark {
        AppearanceMode::Dark
    } else {
        AppearanceMode::Light
    };
    Theme::for_appearance(mode, WindowAppearance::Light, &cfg.diff.palette).with_typography(cfg)
}

impl SettingsView {
    pub(super) fn handle_appearance_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        if self.section != super::SettingsSection::Appearance || self.open_dropdown.is_some() {
            return;
        }
        let Some(direction) = list_nav_from_key(ev, ListNavKeys::CONTENT_LIST) else {
            return;
        };
        let edited_dark = self
            .edited_theme_dark
            .unwrap_or_else(|| crate::app::theme::theme(cx).is_dark);
        let themes: Vec<ColorTheme> = color_themes()
            .into_iter()
            .filter(|t| t.is_dark == edited_dark)
            .collect();
        let current_id = config::current(cx).diff.palette.resolve(edited_dark).id;
        let current_ix = themes.iter().position(|t| t.id == current_id);
        let Some(next_ix) = move_index(current_ix, themes.len(), direction) else {
            return;
        };
        let next_id = themes[next_ix].id.clone();
        if next_id != current_id {
            config::update(cx, move |c| {
                if edited_dark {
                    c.diff.palette.dark = next_id.clone();
                } else {
                    c.diff.palette.light = next_id.clone();
                }
            });
        }
    }
}
