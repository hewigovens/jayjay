use std::sync::OnceLock;

use gpui::{
    AnyElement, Div, InteractiveElement, IntoElement, ParentElement, SharedString, Stateful,
    Styled, Window, WindowControlArea, div, font, prelude::FluentBuilder as _, px, rgb,
};

use crate::app::theme::Theme;

/// Only Windows loses its caption buttons with a transparent title bar.
pub(crate) const DRAWN: bool = cfg!(target_os = "windows");
const HEIGHT: f32 = 32.;

const FLUENT_ICONS: &str = "Segoe Fluent Icons";
const MDL2_ASSETS: &str = "Segoe MDL2 Assets";
const CAPTION_BUTTON_WIDTH: f32 = 46.;
const CLOSE_HOVER_BG: u32 = 0xe81123;

/// Clickable children must `.occlude()`, or they drag the window.
pub(crate) fn drag_strip(id: impl Into<SharedString>) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex()
        .flex_row()
        .flex_none()
        .items_center()
        .w_full()
        .h(px(HEIGHT))
        .window_control_area(WindowControlArea::Drag)
}

pub(crate) fn strip(id: impl Into<SharedString>, t: &Theme, window: &Window) -> Stateful<Div> {
    drag_strip(id)
        .child(div().flex_1())
        .child(caption_buttons(t, window))
}

/// `on_top` paints after the strip so its occluding controls stay clickable over the drag area.
pub(crate) fn overlay(
    content: impl IntoElement,
    on_top: Option<AnyElement>,
    id: impl Into<SharedString>,
    t: &Theme,
    window: &Window,
) -> AnyElement {
    div()
        .relative()
        .flex()
        .flex_col()
        .size_full()
        .child(content)
        .when(DRAWN, |overlay| {
            overlay.child(strip(id, t, window).absolute().top_0().left_0())
        })
        .children(on_top)
        .into_any_element()
}

pub(crate) fn caption_buttons(t: &Theme, window: &Window) -> Div {
    let hover_bg = t.row_alt_bg;
    let (max_id, max_glyph) = if window.is_maximized() {
        ("title-bar-restore", "\u{e923}")
    } else {
        ("title-bar-maximize", "\u{e922}")
    };
    let minimize = caption_button("title-bar-minimize", "\u{e921}", WindowControlArea::Min, t);
    let maximize = caption_button(max_id, max_glyph, WindowControlArea::Max, t);
    div()
        .flex()
        .flex_row()
        .flex_none()
        .h_full()
        .font_family(caption_font(window))
        .child(if window.is_minimizable() {
            minimize.hover(move |s| s.bg(rgb(hover_bg)))
        } else {
            minimize.text_color(rgb(t.fg_faint))
        })
        .child(if window.is_resizable() {
            maximize.hover(move |s| s.bg(rgb(hover_bg)))
        } else {
            maximize.text_color(rgb(t.fg_faint))
        })
        .child(
            caption_button("title-bar-close", "\u{e8bb}", WindowControlArea::Close, t)
                .hover(|s| s.bg(rgb(CLOSE_HOVER_BG)).text_color(rgb(0xffffff))),
        )
}

/// Windows 10 lacks Segoe Fluent Icons; MDL2 has the same glyphs.
fn caption_font(window: &Window) -> &'static str {
    static FONT: OnceLock<&'static str> = OnceLock::new();
    FONT.get_or_init(|| {
        let text_system = window.text_system();
        let resolved = text_system.resolve_font(&font(FLUENT_ICONS));
        let has_fluent = text_system
            .get_font_for_id(resolved)
            .is_some_and(|font| font.family.as_ref() == FLUENT_ICONS);
        if has_fluent {
            FLUENT_ICONS
        } else {
            MDL2_ASSETS
        }
    })
}

fn caption_button(
    id: &'static str,
    glyph: &'static str,
    area: WindowControlArea,
    t: &Theme,
) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .flex()
        .items_center()
        .justify_center()
        .w(px(CAPTION_BUTTON_WIDTH))
        .h_full()
        .occlude()
        .text_size(px(10.))
        .text_color(rgb(t.fg))
        .window_control_area(area)
        .child(glyph)
}
