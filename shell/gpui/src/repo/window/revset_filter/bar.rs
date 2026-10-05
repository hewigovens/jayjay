use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, MouseButton, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, canvas, div, px, rgb,
};
use jayjay_core::{RevsetFilter, RevsetFilterKind};

use super::super::{FocusStop, RepoWindow, focus_ring, picker};
use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::input::line_input_content;
use crate::ui::primitives::text_tooltip;

const WIDTH: f32 = 495.;
const HEIGHT: f32 = 28.;
const MIN_WIDTH: f32 = 280.;

/// The graph's filter in the title bar: click the revset to edit it in place, the filter icon for presets, bookmarks and recent revsets.
pub(crate) fn revset_bar(view: &RepoWindow, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let (revset, has_previous) = {
        let vm = view.vm.read(cx);
        (vm.revset().to_owned(), vm.revset_filter.previous.is_some())
    };
    let filter = RevsetFilter::of(&revset);
    let narrowed = filter.kind != RevsetFilterKind::Default;
    let editing = view.revset_editor.is_some();
    let focused = view.focused_control;
    let bounds = view.revset_bar_bounds.clone();
    let background = if editing {
        t.detail_bg
    } else if narrowed {
        t.toggle_active_bg
    } else {
        t.toolbar_group_bg
    };

    let bordered = editing || focused == Some(FocusStop::RevsetFilter);

    let mut bar = div()
        .id("revset-bar")
        .debug_selector(|| "revset-bar".to_owned())
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .w(px(WIDTH))
        .min_w(px(MIN_WIDTH))
        .h(px(HEIGHT))
        .pl(px(4.))
        .pr(px(if narrowed && !editing { 4. } else { 10. }))
        .rounded_full()
        .bg(rgb(background))
        .when(bordered, |el| {
            el.border_1().border_color(rgb(t.selected_accent))
        })
        .child(
            // An absolute child starts inside the padding and the border, which would shift what hangs under the bar.
            canvas(move |b, _, _| bounds.set(Some(b)), |_, _, _, _| {})
                .absolute()
                .inset(px(if bordered { -1. } else { 0. })),
        );
    if has_previous && !editing {
        bar = bar.child(
            bar_icon(
                "revset-back",
                glyph::ARROW_UTURN_BACK,
                Some("Back to previous filter"),
                t.fg_dim,
                focused == Some(FocusStop::RevsetBack),
                t,
            )
            .on_click(cx.listener(|view, _: &ClickEvent, _, cx| {
                view.return_to_previous_revset(cx);
            })),
        );
    }
    let presets = bar_icon(
        "revset-presets",
        if filter.kind == RevsetFilterKind::Bookmark {
            glyph::BOOKMARK
        } else {
            glyph::FILTER
        },
        None,
        if narrowed {
            t.toggle_active_fg
        } else {
            t.fg_dim
        },
        focused == Some(FocusStop::RevsetPresets),
        t,
    )
    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
        view.toggle_revset_popup(window, cx);
    }));
    bar = bar.child(picker::opener(
        presets,
        |view| view.revset_popup.is_some(),
        RepoWindow::close_revset_popup,
        cx,
    ));
    bar = match view.revset_editor.as_ref() {
        Some(input) => bar.child(
            div()
                .id("revset-editor")
                .debug_selector(|| "revset-editor".to_owned())
                .flex()
                .items_center()
                .flex_1()
                .min_w_0()
                .code_text(11.)
                .cursor_text()
                .track_focus(&view.revset_editor_focus)
                .on_key_down(cx.listener(|view, ev, window, cx| {
                    if view.handle_revset_editor_key(ev, window, cx) {
                        cx.stop_propagation();
                    }
                }))
                .child(line_input_content(
                    input,
                    "Revset",
                    t,
                    Some("revset-editor-caret"),
                )),
        ),
        None => bar.child(summary(&filter, &revset, t, cx)),
    };
    if narrowed && !editing {
        bar = bar.child(
            bar_icon(
                "revset-reset",
                glyph::X_CIRCLE,
                Some("Reset to default"),
                t.fg_faint,
                focused == Some(FocusStop::RevsetReset),
                t,
            )
            .on_click(cx.listener(|view, _: &ClickEvent, _, cx| view.apply_revset("", cx))),
        );
    }
    bar.into_any_element()
}

fn summary(
    filter: &RevsetFilter,
    revset: &str,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let narrowed = filter.kind != RevsetFilterKind::Default;
    let mut summary = div()
        .id("revset-summary")
        .debug_selector(|| "revset-summary".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(7.))
        .flex_1()
        .min_w_0()
        .h_full()
        .cursor_text()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
            view.begin_revset_edit(window, cx);
        }));
    if !filter.label.is_empty() {
        summary = summary.child(
            div()
                .flex_none()
                .text_size(ui_font_size(12.))
                .when(narrowed, |el| el.font_weight(gpui::FontWeight::MEDIUM))
                .text_color(rgb(if narrowed {
                    t.toggle_active_fg
                } else {
                    t.fg_dim
                }))
                .child(SharedString::from(filter.label.clone())),
        );
    }
    if filter.kind != RevsetFilterKind::Bookmark {
        summary = summary.child(
            div()
                .min_w_0()
                .truncate()
                .code_text(11.)
                .text_color(rgb(if filter.kind == RevsetFilterKind::Custom {
                    t.fg
                } else {
                    t.fg_faint
                }))
                .child(SharedString::from(revset.to_owned())),
        );
    }
    summary.into_any_element()
}

fn bar_icon(
    id: &'static str,
    glyph_str: &'static str,
    tooltip: Option<&'static str>,
    color: u32,
    focused: bool,
    t: &Theme,
) -> gpui::Stateful<gpui::Div> {
    focus_ring(
        div()
            .id(id)
            .debug_selector(move || id.to_owned())
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .w(px(26.))
            .h_full()
            .rounded_full()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(t.row_alt_bg)))
            .when_some(tooltip, |el, tooltip| el.tooltip(text_tooltip(tooltip)))
            .child(icons::icon(glyph_str, 12., color)),
        focused,
        t,
    )
}
