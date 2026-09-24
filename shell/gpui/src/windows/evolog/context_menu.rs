use gpui::{
    Anchor, AnyElement, ClipboardItem, Entity, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Point, SharedString, StatefulInteractiveElement, Styled,
    anchored, deferred, div, px, rgb,
};

use super::EvologView;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::primitives::{icon_label, text_tooltip};

pub(super) struct EvologContextMenuState {
    pub anchor: Point<Pixels>,
    pub commit_id: String,
    pub into_rev: String,
    pub is_immutable: bool,
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
        .child(menu_panel(state, theme, view));

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

#[derive(Clone)]
enum EvologMenuAction {
    Copy(String),
    Restore(String),
}

struct EvologMenuItem {
    selector: &'static str,
    label: &'static str,
    icon: &'static str,
    action: EvologMenuAction,
    tooltip: Option<SharedString>,
    enabled: bool,
}

fn menu_panel(
    state: &EvologContextMenuState,
    theme: &Theme,
    view: &Entity<EvologView>,
) -> AnyElement {
    let restore_help: SharedString = if state.is_immutable {
        "Immutable changes cannot be restored".into()
    } else {
        format!("Restore this version into {}", state.into_rev).into()
    };
    let items = [
        EvologMenuItem {
            selector: "evolog-context-restore",
            label: "Restore this version",
            icon: glyph::ARROW_CLOCKWISE,
            action: EvologMenuAction::Restore(state.commit_id.clone()),
            tooltip: Some(restore_help),
            enabled: !state.is_immutable,
        },
        EvologMenuItem {
            selector: "evolog-context-copy-commit",
            label: "Copy Commit ID",
            icon: glyph::COPY,
            action: EvologMenuAction::Copy(state.commit_id.clone()),
            tooltip: None,
            enabled: true,
        },
        EvologMenuItem {
            selector: "evolog-context-copy-restore",
            label: "Copy ‘jj restore’ command",
            icon: glyph::TERMINAL,
            action: EvologMenuAction::Copy(format!(
                "jj restore --from {} --into {}",
                state.commit_id, state.into_rev
            )),
            tooltip: None,
            enabled: true,
        },
    ];
    let mut col = div()
        .flex()
        .flex_col()
        .min_w(px(210.))
        .py(px(4.))
        .bg(rgb(theme.detail_bg))
        .border_1()
        .border_color(rgb(theme.border))
        .rounded_sm();
    for (ix, item) in items.iter().enumerate() {
        col = col.child(menu_row(ix, item, theme, view));
    }
    col.into_any_element()
}

fn menu_row(
    index: usize,
    item: &EvologMenuItem,
    theme: &Theme,
    view: &Entity<EvologView>,
) -> AnyElement {
    let view = view.clone();
    let selector = item.selector;
    let row = div()
        .id(("evolog-context-menu-row", index))
        .debug_selector(move || selector.to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(5.))
        .text_size(ui_font_size(12.))
        .child(icon_label(
            item.icon,
            SharedString::from(item.label),
            12.,
            theme.fg_dim,
        ));
    let row = match &item.tooltip {
        Some(help) => row.tooltip(text_tooltip(help.clone())),
        None => row,
    };
    if item.enabled {
        let action = item.action.clone();
        row.text_color(rgb(theme.fg))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme.selected_bg)))
            .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                match action.clone() {
                    EvologMenuAction::Copy(value) => {
                        cx.write_to_clipboard(ClipboardItem::new_string(value));
                        view.update(cx, |this, cx| this.close_context_menu(cx));
                    }
                    EvologMenuAction::Restore(commit_id) => {
                        view.update(cx, |this, cx| {
                            this.restore_version(commit_id.clone(), cx);
                            this.close_context_menu(cx);
                        });
                    }
                }
            })
            .into_any_element()
    } else {
        row.text_color(rgb(theme.fg_faint))
            .opacity(0.45)
            .into_any_element()
    }
}
