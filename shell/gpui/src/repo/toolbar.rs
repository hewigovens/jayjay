use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

use crate::app::theme::{Theme, theme};
use crate::log::LogView;
use crate::ui::icons::{self, glyph};
use crate::windows::settings::SettingsView;

const TOOLBAR_HEIGHT: f32 = 44.;
const TRAFFIC_LIGHT_INSET: f32 = 78.;

pub fn toolbar(
    repo_path: SharedString,
    bookmark_count: usize,
    has_wc_changes: bool,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let t = theme(cx).clone();

    let repo_name = repo_path
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(repo_path.as_ref())
        .to_owned();

    div()
        .id(SharedString::from("toolbar"))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(TOOLBAR_HEIGHT))
        .pl(px(TRAFFIC_LIGHT_INSET))
        .pr(px(12.))
        .gap(px(6.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|_, ev: &MouseDownEvent, window, _cx| {
                if ev.click_count == 2 {
                    window.zoom_window();
                }
            }),
        )
        .child(bookmarks_button(bookmark_count, &t, cx))
        .child(coming_soon_icon_button(
            glyph::FILTER,
            "tb-filter",
            "Filter",
            &t,
            cx,
        ))
        .child(divider(&t))
        .child(refresh_button(has_wc_changes, &t, cx))
        .child(coming_soon_icon_button(
            glyph::ARROW_DOWN,
            "tb-pull",
            "Pull",
            &t,
            cx,
        ))
        .child(coming_soon_icon_button(
            glyph::ARROW_UP,
            "tb-push",
            "Push",
            &t,
            cx,
        ))
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(13.))
                .text_color(rgb(t.fg))
                .child(SharedString::from(repo_name)),
        )
        .child(div().flex_1())
        .child(toolbar_icon_button(
            glyph::GEAR,
            "tb-settings",
            &t,
            Some(|_ev: &ClickEvent, _w: &mut Window, cx: &mut gpui::App| {
                SettingsView::open(cx);
            }),
        ))
        .into_any_element()
}

fn bookmarks_button(count: usize, t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    let label = if count == 0 {
        SharedString::from("Bookmarks")
    } else {
        SharedString::from(format!("Bookmarks ({count})"))
    };
    let hover_bg = t.row_alt_bg;
    let active_bg = t.selected_bg;
    div()
        .id(SharedString::from("tb-bookmarks"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(10.))
        .py(px(4.))
        .rounded_sm()
        .bg(rgb(t.toolbar_button_bg))
        .text_size(px(11.))
        .text_color(rgb(t.fg_dim))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(hover_bg)))
        .active(|s| s.bg(rgb(active_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, _w, cx| {
                view.open_bookmark_picker(ev.position, cx);
            }),
        )
        .child(icons::icon(glyph::GIT_BRANCH, 14., t.fg_dim))
        .child(label)
        .into_any_element()
}

fn coming_soon_icon_button(
    glyph_str: &'static str,
    id: &'static str,
    label: &'static str,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let hover_bg = t.row_alt_bg;
    let active_bg = t.selected_bg;
    div()
        .id(SharedString::from(id))
        .flex()
        .items_center()
        .justify_center()
        .w(px(28.))
        .h(px(24.))
        .rounded_sm()
        .bg(rgb(t.toolbar_icon_bg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(hover_bg)))
        .active(|s| s.bg(rgb(active_bg)))
        .on_click(cx.listener(move |view, _ev: &ClickEvent, _w, cx| {
            view.show_coming_soon(label, cx);
        }))
        .child(icons::icon(glyph_str, 14., t.fg_dim))
        .into_any_element()
}

fn refresh_button(badge: bool, t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    let hover_bg = t.row_alt_bg;
    let active_bg = t.selected_bg;
    let mut content = div()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w(px(28.))
        .h(px(24.))
        .child(icons::icon(glyph::ARROW_CLOCKWISE, 14., t.fg_dim));
    if badge {
        content = content.child(
            div()
                .absolute()
                .top(px(4.))
                .right(px(6.))
                .w(px(6.))
                .h(px(6.))
                .rounded_full()
                .bg(rgb(0xf59e0b)),
        );
    }
    div()
        .id(SharedString::from("tb-refresh"))
        .flex()
        .items_center()
        .justify_center()
        .w(px(28.))
        .h(px(24.))
        .rounded_sm()
        .bg(rgb(t.toolbar_icon_bg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(hover_bg)))
        .active(|s| s.bg(rgb(active_bg)))
        .on_click(cx.listener(|view, _ev: &ClickEvent, _w, cx| {
            let vm = view.vm.clone();
            vm.update(cx, |vm, cx| vm.refresh(false, cx));
        }))
        .child(content)
        .into_any_element()
}

fn divider(t: &Theme) -> AnyElement {
    div()
        .w(px(1.))
        .h(px(20.))
        .bg(rgb(t.border))
        .into_any_element()
}

type Click = fn(&ClickEvent, &mut Window, &mut gpui::App);

fn toolbar_icon_button(
    glyph_str: &'static str,
    id: &'static str,
    t: &Theme,
    on_click: Option<Click>,
) -> AnyElement {
    let hover_bg = t.row_alt_bg;
    let active_bg = t.selected_bg;
    let mut el = div()
        .id(SharedString::from(id))
        .flex()
        .items_center()
        .justify_center()
        .w(px(28.))
        .h(px(24.))
        .rounded_sm()
        .bg(rgb(t.toolbar_icon_bg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(hover_bg)))
        .active(|s| s.bg(rgb(active_bg)))
        .child(icons::icon(glyph_str, 14., t.fg_dim));
    if let Some(handler) = on_click {
        el = el.on_click(handler);
    }
    el.into_any_element()
}
