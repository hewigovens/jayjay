use gpui::{
    Anchor, AnyElement, Context, Entity, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Point, Styled, anchored, deferred, div, px, rgb,
};
use jayjay_core::ChangeInfo;

use super::FileHistoryView;
use super::comparison::FileHistoryEndpoint;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::primitives::icon_label;

pub(super) struct FileHistoryMenu {
    anchor: Point<Pixels>,
    entry: ChangeInfo,
}

impl FileHistoryView {
    pub(super) fn open_menu(
        &mut self,
        anchor: Point<Pixels>,
        entry: ChangeInfo,
        cx: &mut Context<Self>,
    ) {
        self.menu = Some(FileHistoryMenu { anchor, entry });
        cx.notify();
    }

    pub(super) fn close_menu(&mut self, cx: &mut Context<Self>) {
        if self.menu.take().is_some() {
            cx.notify();
        }
    }
}

pub(super) fn render_menu(
    menu: &FileHistoryMenu,
    theme: &Theme,
    view: &Entity<FileHistoryView>,
) -> AnyElement {
    let backdrop_view = view.clone();
    let backdrop = div()
        .id("file-history-menu-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(MouseButton::Left, {
            let view = backdrop_view.clone();
            move |_: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                view.update(cx, |this, cx| this.close_menu(cx));
            }
        })
        .on_mouse_down(MouseButton::Right, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            backdrop_view.update(cx, |this, cx| this.close_menu(cx));
        });

    let panel = anchored()
        .anchor(Anchor::TopLeft)
        .position(menu.anchor)
        .snap_to_window_with_margin(px(6.))
        .child(menu_panel(menu, theme, view));

    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(backdrop)
            .child(panel),
    )
    .with_priority(2)
    .into_any_element()
}

fn menu_panel(menu: &FileHistoryMenu, theme: &Theme, view: &Entity<FileHistoryView>) -> AnyElement {
    let entry = menu.entry.clone();
    div()
        .flex()
        .flex_col()
        .min_w(px(180.))
        .py(px(4.))
        .bg(rgb(theme.detail_bg))
        .border_1()
        .border_color(rgb(theme.border))
        .rounded_sm()
        .child(menu_row(
            "file-history-menu-pin",
            "Pin as Base",
            glyph::PIN,
            entry.clone(),
            theme,
            view,
            MenuAction::Pin,
        ))
        .child(menu_row(
            "file-history-menu-show",
            "Show in Graph",
            glyph::LIST_TREE,
            entry,
            theme,
            view,
            MenuAction::Show,
        ))
        .into_any_element()
}

#[derive(Clone, Copy)]
enum MenuAction {
    Pin,
    Show,
}

fn menu_row(
    selector: &'static str,
    label: &'static str,
    icon: &'static str,
    entry: ChangeInfo,
    theme: &Theme,
    view: &Entity<FileHistoryView>,
    action: MenuAction,
) -> AnyElement {
    let view = view.clone();
    div()
        .id(selector)
        .debug_selector(move || selector.to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(5.))
        .text_size(ui_font_size(12.))
        .text_color(rgb(theme.fg))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.selected_bg)))
        .child(icon_label(icon, label, 12., theme.fg_dim))
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
            cx.stop_propagation();
            view.update(cx, |this, cx| {
                this.close_menu(cx);
                match action {
                    MenuAction::Pin => {
                        this.pin_endpoint(FileHistoryEndpoint::from_change(&entry), cx);
                    }
                    MenuAction::Show => this.reveal_entry(&entry, window, cx),
                }
            });
        })
        .into_any_element()
}
