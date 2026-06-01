pub mod about;
pub mod appearance;
pub mod config;
pub mod diff;
pub mod features;
pub mod shared;
pub mod tools;

use gpui::{
    Anchor, AnyElement, App, AppContext, Bounds, ClickEvent, Context, FocusHandle, Focusable,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement, Pixels, Point,
    Render, SharedString, StatefulInteractiveElement, Styled, TitlebarOptions, Window,
    WindowBounds, WindowOptions, anchored, deferred, div, px, rgb, size,
};

use crate::app::config::{AppConfigStore, current as current_cfg, update as update_cfg};
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use crate::ui::icons::{self, glyph};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    Diff,
    Tools,
    Features,
    Config,
    About,
}

const SECTIONS: &[(SettingsSection, &str, &str)] = &[
    (SettingsSection::Appearance, "Appearance", glyph::WHITESPACE),
    (SettingsSection::Diff, "Diff", glyph::COLUMNS),
    (SettingsSection::Tools, "Tools", glyph::GEAR),
    (SettingsSection::Features, "Features", glyph::SPARKLE),
    (SettingsSection::Config, "Config", glyph::FILE_CODE),
    (SettingsSection::About, "About", glyph::INFO),
];

#[derive(Clone)]
pub(super) struct OpenDropdown {
    pub field_id: SharedString,
    pub anchor: Point<Pixels>,
}

pub struct SettingsView {
    section: SettingsSection,
    focus_handle: FocusHandle,
    open_dropdown: Option<OpenDropdown>,
}

impl SettingsView {
    pub fn open(cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(680.), px(520.)), cx);
        let window_handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("JayJay Settings".into()),
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
                            section: SettingsSection::Appearance,
                            focus_handle: cx.focus_handle(),
                            open_dropdown: None,
                        }
                    })
                },
            )
            .ok();
        if let Some(handle) = window_handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                let h = view.focus_handle(cx);
                window.focus(&h, cx);
            });
        }
    }

    pub(super) fn open_dropdown(
        &mut self,
        field_id: SharedString,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.open_dropdown = Some(OpenDropdown { field_id, anchor });
        cx.notify();
    }

    pub(super) fn close_dropdown(&mut self, cx: &mut Context<Self>) {
        if self.open_dropdown.take().is_some() {
            cx.notify();
        }
    }
}

impl Focusable for SettingsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cfg = current_cfg(cx);
        let t = theme(cx).clone();
        let active = self.section;
        let dropdown = self.open_dropdown.clone();

        let mut root = div()
            .track_focus(&self.focus_handle)
            .key_context("SettingsView")
            .on_action(
                cx.listener(|_, _: &crate::app::actions::CloseWindow, window, _cx| {
                    window.remove_window();
                }),
            )
            .relative()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(sidebar(active, &t, cx))
            .child(crate::ui::primitives::divider_v(&t))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .px(px(28.))
                    .py(px(20.))
                    .child(section_body(active, &cfg, &t, cx)),
            );
        if let Some(d) = dropdown {
            root = root.child(dropdown_overlay(d, &t, cx));
        }
        root
    }
}

fn dropdown_overlay(state: OpenDropdown, t: &Theme, cx: &mut Context<SettingsView>) -> AnyElement {
    let options: &[(&str, &str)] = match state.field_id.as_ref() {
        "editor" => EDITOR_OPTIONS,
        "terminal" => TERMINAL_OPTIONS,
        _ => return div().into_any_element(),
    };
    let cfg = current_cfg(cx);
    let current = match state.field_id.as_ref() {
        "editor" => cfg.tools.external_editor.clone(),
        "terminal" => cfg.tools.terminal.clone(),
        _ => String::new(),
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
        .flex()
        .flex_col()
        .min_w(px(180.))
        .py(px(4.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm();

    for (id, label) in options {
        let is_selected = current == *id;
        let field_for_click: SharedString = field_id.clone();
        let id_owned: &'static str = id;
        panel = panel.child(
            div()
                .id(SharedString::from(format!("dd-{field_id}-{id}")))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .px(px(10.))
                .py(px(5.))
                .text_size(px(12.))
                .text_color(rgb(if is_selected {
                    t.toggle_active_fg
                } else {
                    t.fg
                }))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(t.selected_bg)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |view, _: &MouseDownEvent, _w, cx| {
                        let value = id_owned.to_owned();
                        let field = field_for_click.clone();
                        update_cfg(cx, move |c| match field.as_ref() {
                            "editor" => c.tools.external_editor = value.clone(),
                            "terminal" => c.tools.terminal = value.clone(),
                            _ => {}
                        });
                        view.close_dropdown(cx);
                    }),
                )
                .child(if is_selected {
                    icons::icon(glyph::CHECK, 12., t.toggle_active_fg)
                } else {
                    icons::icon(glyph::DOT, 12., t.fg_faint)
                })
                .child(SharedString::from(*label)),
        );
    }

    let menu = anchored()
        .anchor(Anchor::TopLeft)
        .position(state.anchor)
        .snap_to_window_with_margin(px(6.))
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

fn sidebar(active: SettingsSection, t: &Theme, cx: &mut Context<SettingsView>) -> AnyElement {
    let mut col = div()
        .flex()
        .flex_col()
        .w(px(180.))
        .h_full()
        .bg(rgb(t.sidebar_bg))
        .py(px(12.));

    for (sect, label, glyph_str) in SECTIONS {
        col = col.child(nav_button(*sect, label, glyph_str, *sect == active, t, cx));
    }
    col.into_any_element()
}

fn nav_button(
    sect: SettingsSection,
    label: &'static str,
    glyph_str: &'static str,
    active: bool,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let (bg, fg) = if active {
        (t.selected_bg, t.fg)
    } else {
        (t.sidebar_bg, t.fg_dim)
    };
    div()
        .id(SharedString::from(format!("settings-nav-{label}")))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(14.))
        .py(px(8.))
        .bg(rgb(bg))
        .text_size(px(12.))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _ev: &ClickEvent, _w, cx| {
            this.section = sect;
            cx.notify();
        }))
        .child(icons::icon(glyph_str, 14., fg))
        .child(label)
        .into_any_element()
}

fn section_body(
    sect: SettingsSection,
    cfg: &crate::app::config::AppConfig,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    match sect {
        SettingsSection::Appearance => appearance::appearance_section(cfg, t),
        SettingsSection::Diff => diff::diff_section(cfg, t),
        SettingsSection::Tools => tools::tools_section(cfg, t, cx),
        SettingsSection::Features => features::features_section(cfg, t),
        SettingsSection::Config => config::config_section(t),
        SettingsSection::About => about::about_section(t).into_any_element(),
    }
}
