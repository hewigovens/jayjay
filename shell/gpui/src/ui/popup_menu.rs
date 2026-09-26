use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Point, SharedString, Styled, Window, anchored, deferred,
    div, px, rgb,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::ui::primitives::icon_label;

#[derive(Clone)]
pub(crate) enum PopupMenuEntry<A> {
    Item {
        label: SharedString,
        glyph: &'static str,
        action: A,
    },
    Separator,
}

impl<A> PopupMenuEntry<A> {
    pub(crate) fn item(label: impl Into<SharedString>, glyph: &'static str, action: A) -> Self {
        Self::Item {
            label: label.into(),
            glyph,
            action,
        }
    }
}

#[derive(Clone)]
pub(crate) struct PopupMenu<A> {
    pub(crate) anchor: Point<Pixels>,
    pub(crate) entries: Vec<PopupMenuEntry<A>>,
}

pub(crate) fn render_popup_menu<A: Clone + 'static>(
    menu: &PopupMenu<A>,
    id: &'static str,
    t: &Theme,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
    on_select: impl Fn(A, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let on_dismiss = Rc::new(on_dismiss);
    let on_select = Rc::new(on_select);
    let backdrop = div()
        .id(SharedString::from(format!("{id}-backdrop")))
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(MouseButton::Left, {
            let on_dismiss = on_dismiss.clone();
            move |_: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                on_dismiss(window, cx);
            }
        })
        .on_mouse_down(MouseButton::Right, move |_: &MouseDownEvent, window, cx| {
            cx.stop_propagation();
            on_dismiss(window, cx);
        });

    let mut panel = div()
        .flex()
        .flex_col()
        .min_w(px(180.))
        .py(px(4.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm();
    for (ix, entry) in menu.entries.iter().enumerate() {
        panel = panel.child(match entry {
            PopupMenuEntry::Separator => div()
                .my(px(4.))
                .h(px(1.))
                .bg(rgb(t.border))
                .into_any_element(),
            PopupMenuEntry::Item {
                label,
                glyph,
                action,
            } => {
                let action = action.clone();
                let on_select = on_select.clone();
                let selector = format!("{id}-{label}");
                div()
                    .id(ElementId::named_usize(format!("{id}-row"), ix))
                    .debug_selector(move || selector.clone())
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(5.))
                    .text_size(ui_font_size(12.))
                    .text_color(rgb(t.fg))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(t.selected_bg)))
                    .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        on_select(action.clone(), window, cx);
                    })
                    .child(icon_label(glyph, label.clone(), 12., t.fg_dim))
                    .into_any_element()
            }
        });
    }

    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(backdrop)
            .child(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .position(menu.anchor)
                    .snap_to_window_with_margin(px(6.))
                    .child(panel),
            ),
    )
    .with_priority(2)
    .into_any_element()
}
