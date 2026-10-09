use gpui::{
    Anchor, AnyElement, BoxShadow, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Point, SharedString, StatefulInteractiveElement, Styled,
    Window, anchored, deferred, div, hsla, px, rgb,
};

use super::SettingsView;
use crate::app::config::{current as current_cfg, update as update_cfg};
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use crate::ui::icons::{self, glyph};

#[derive(Clone)]
pub(super) struct OpenDropdown {
    pub field_id: SharedString,
    pub anchor: Point<Pixels>,
    options: Vec<DropdownOption>,
}

const MENU_MARGIN: f32 = 6.;

pub(super) fn dropdown_overlay(
    state: OpenDropdown,
    t: &Theme,
    window: &Window,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let cfg = current_cfg(cx);
    let Some(current) = dropdown_current(state.field_id.as_ref(), &cfg) else {
        return div().into_any_element();
    };
    let field_id = state.field_id.clone();

    let backdrop = div()
        .id(SharedString::from("dropdown-backdrop"))
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, _: &MouseDownEvent, _w, cx| view.close_dropdown(cx)),
        );

    let mut panel = div()
        .id("dropdown-panel")
        .debug_selector(|| "dropdown-panel".to_owned())
        .flex()
        .flex_col()
        .max_h(window.viewport_size().height - px(2. * MENU_MARGIN))
        .overflow_y_scroll()
        .gap(px(1.))
        .p(px(8.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded(px(14.))
        .shadow(popup_shadow(t));

    let mut group = None;
    for option in state.options {
        if option.group != group {
            group = option.group;
            panel = panel.children(group.map(|title| dropdown_group_header(title, t)));
        }
        let is_selected = current == option.id;
        panel = panel.child(dropdown_row(
            &field_id,
            option.id,
            option.label,
            is_selected,
            t,
            cx,
        ));
    }

    let menu = anchored()
        .anchor(Anchor::TopLeft)
        .position(state.anchor)
        .snap_to_window_with_margin(px(MENU_MARGIN))
        .child(panel);

    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(backdrop)
            .child(menu),
    )
    .with_priority(2)
    .into_any_element()
}

pub(super) fn dropdown_button(
    field_id: &'static str,
    label: String,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .id(SharedString::from(format!("dd-btn-{field_id}")))
        .debug_selector(move || format!("dd-btn-{field_id}"))
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(t.scaled_control_height(32., 12.)))
        .pl(px(24.))
        .pr(px(40.))
        .rounded_md()
        .bg(rgb(t.toggle_inactive_bg))
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                view.open_dropdown(SharedString::from(field_id), ev.position, cx);
            }),
        )
        .child(
            div()
                .min_w_0()
                .text_center()
                .truncate()
                .child(SharedString::from(label)),
        )
        .child(dropdown_chevrons(t))
        .into_any_element()
}

const INTERFACE_SYSTEM_ID: &str = "system";

#[derive(Debug, Clone)]
struct DropdownOption {
    id: String,
    label: String,
    group: Option<&'static str>,
}

fn dropdown_options(field_id: &str) -> Vec<DropdownOption> {
    match field_id {
        "editor" => static_options(EDITOR_OPTIONS),
        "terminal" => static_options(TERMINAL_OPTIONS),
        "interface-font-family" => {
            let system = DropdownOption {
                id: INTERFACE_SYSTEM_ID.to_owned(),
                label: fonts::interface_preference_label(""),
                group: None,
            };
            let user = fonts::interface_font_choices()
                .into_iter()
                .map(|family| DropdownOption {
                    id: family.clone(),
                    label: family,
                    group: Some("My Fonts"),
                });
            std::iter::once(system).chain(user).collect()
        }
        "code-font-family" => {
            let curated = fonts::mono_font_choices()
                .into_iter()
                .map(|choice| DropdownOption {
                    id: choice.id,
                    label: choice.title,
                    group: None,
                });
            let my_fonts = fonts::my_mono_fonts()
                .into_iter()
                .map(|family| DropdownOption {
                    id: family.clone(),
                    label: family,
                    group: Some("My Fonts"),
                });
            curated.chain(my_fonts).collect()
        }
        _ => Vec::new(),
    }
}

fn dropdown_current(field_id: &str, cfg: &crate::app::config::AppConfig) -> Option<String> {
    match field_id {
        "editor" => Some(cfg.tools.external_editor.clone()),
        "terminal" => Some(cfg.tools.terminal.clone()),
        "interface-font-family" => Some(if cfg.interface_font_family.is_empty() {
            INTERFACE_SYSTEM_ID.to_owned()
        } else {
            cfg.interface_font_family.clone()
        }),
        "code-font-family" => Some(fonts::mono_preference_id(&cfg.code_font_family)),
        _ => None,
    }
}

fn static_options(options: &[(&str, &str)]) -> Vec<DropdownOption> {
    options
        .iter()
        .map(|(id, label)| DropdownOption {
            id: (*id).to_owned(),
            label: (*label).to_owned(),
            group: None,
        })
        .collect()
}

fn dropdown_row(
    field_id: &SharedString,
    id: String,
    label: String,
    is_selected: bool,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let field_for_click = field_id.clone();
    let row_id = format!("dd-{field_id}-{id}");
    let row_bg = if is_selected {
        t.selected_accent
    } else {
        t.detail_bg
    };
    let hover_bg = if is_selected {
        t.selected_accent
    } else {
        t.selected_bg
    };
    let text_color = if is_selected { 0xffffff } else { t.fg };
    div()
        .id(SharedString::from(row_id.clone()))
        .debug_selector(move || row_id)
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .h(px(t.scaled_control_height(34., 13.)))
        .px(px(10.))
        .rounded_lg()
        .bg(rgb(row_bg))
        .text_size(ui_font_size(13.))
        .text_color(rgb(text_color))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, _: &MouseDownEvent, _w, cx| {
                cx.stop_propagation();
                let value = id.clone();
                let field = field_for_click.clone();
                update_cfg(cx, move |c| match field.as_ref() {
                    "editor" => c.tools.external_editor = value.clone(),
                    "terminal" => c.tools.terminal = value.clone(),
                    "interface-font-family" => {
                        c.interface_font_family = if value == INTERFACE_SYSTEM_ID {
                            String::new()
                        } else {
                            value.clone()
                        };
                    }
                    "code-font-family" => c.code_font_family = value.clone(),
                    _ => {}
                });
                view.close_dropdown(cx);
            }),
        )
        .child(selection_checkmark(is_selected))
        .child(
            div()
                .flex_none()
                .truncate()
                .child(SharedString::from(label)),
        )
        .into_any_element()
}

fn dropdown_group_header(title: &'static str, t: &Theme) -> AnyElement {
    div()
        .flex_none()
        .px(px(10.))
        .pt(px(8.))
        .pb(px(2.))
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_dim))
        .child(title)
        .into_any_element()
}

fn selection_checkmark(is_selected: bool) -> AnyElement {
    let checkmark = if is_selected {
        icons::icon(glyph::CHECK, 15., 0xffffff).into_any_element()
    } else {
        div().into_any_element()
    };
    div()
        .flex_none()
        .w(px(20.))
        .flex()
        .items_center()
        .justify_center()
        .child(checkmark)
        .into_any_element()
}

fn popup_shadow(t: &Theme) -> Vec<BoxShadow> {
    let (wide, tight) = if t.is_dark {
        (0.32, 0.26)
    } else {
        (0.14, 0.08)
    };
    vec![
        BoxShadow::new(px(0.), px(10.), hsla(0., 0., 0., wide)).blur_radius(px(28.)),
        BoxShadow::new(px(0.), px(1.), hsla(0., 0., 0., tight)),
    ]
}

fn dropdown_chevrons(t: &Theme) -> AnyElement {
    div()
        .absolute()
        .right(px(6.))
        .top(px(5.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(22.))
        .h(px(22.))
        .rounded_full()
        .bg(rgb(t.row_alt_bg))
        .child(icons::icon(glyph::CARETS_UP_DOWN, 11., t.fg_dim))
        .into_any_element()
}

impl SettingsView {
    pub(super) fn open_dropdown(
        &mut self,
        field_id: SharedString,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let options = dropdown_options(field_id.as_ref());
        self.open_dropdown = Some(OpenDropdown {
            field_id,
            anchor,
            options,
        });
        cx.notify();
    }

    pub(super) fn close_dropdown(&mut self, cx: &mut Context<Self>) {
        if self.open_dropdown.take().is_some() {
            cx.notify();
        }
    }
}
