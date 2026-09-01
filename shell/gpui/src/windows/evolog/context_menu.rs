use gpui::{
    Anchor, AnyElement, ClipboardItem, Entity, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Point, SharedString, Styled, anchored, deferred, div,
    px, rgb,
};

use super::EvologView;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::primitives::icon_label;

pub(super) struct EvologContextMenuState {
    pub anchor: Point<Pixels>,
    pub commit_id: String,
}

pub(super) fn render_context_menu(
    state: &EvologContextMenuState,
    theme: &Theme,
    view: &Entity<EvologView>,
) -> AnyElement {
    let backdrop_view = view.clone();
    let backdrop = div()
        .id("evolog-menu-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(MouseButton::Left, {
            let view = backdrop_view.clone();
            move |_: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                view.update(cx, |this, cx| this.close_context_menu(cx));
            }
        })
        .on_mouse_down(MouseButton::Right, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            backdrop_view.update(cx, |this, cx| this.close_context_menu(cx));
        });

    let menu = anchored()
        .anchor(Anchor::TopLeft)
        .position(state.anchor)
        .snap_to_window_with_margin(px(6.))
        .child(menu_panel(&state.commit_id, theme, view));

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

fn menu_panel(commit_id: &str, theme: &Theme, view: &Entity<EvologView>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .min_w(px(210.))
        .py(px(4.))
        .bg(rgb(theme.detail_bg))
        .border_1()
        .border_color(rgb(theme.border))
        .rounded_sm()
        .child(menu_row(
            0,
            "evolog-context-copy-commit",
            "Copy Commit ID",
            glyph::COPY,
            commit_id.to_owned(),
            theme,
            view,
        ))
        .child(menu_row(
            1,
            "evolog-context-copy-restore",
            "Copy ‘jj restore’ command",
            glyph::TERMINAL,
            format!("jj restore --from {commit_id} --into @"),
            theme,
            view,
        ))
        .into_any_element()
}

fn menu_row(
    index: usize,
    selector: &'static str,
    label: &'static str,
    icon: &'static str,
    value: String,
    theme: &Theme,
    view: &Entity<EvologView>,
) -> AnyElement {
    let view = view.clone();
    div()
        .id(("evolog-context-menu-row", index))
        .debug_selector(move || selector.to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(5.))
        .text_size(px(12.))
        .text_color(rgb(theme.fg))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.selected_bg)))
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
            view.update(cx, |this, cx| this.close_context_menu(cx));
        })
        .child(icon_label(
            icon,
            SharedString::from(label),
            12.,
            theme.fg_dim,
        ))
        .into_any_element()
}
