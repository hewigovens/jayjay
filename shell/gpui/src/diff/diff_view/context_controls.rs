use gpui::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, ParentElement, Role, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::diff::{ContextExpansion, ContextRegion};

use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::repo::window::RepoWindow;
use crate::ui::primitives::button;

pub(super) fn context_error_banner(
    message: SharedString,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(18.))
        .py(px(6.))
        .bg(rgba(with_alpha(
            t.tag_conflict_fg,
            if t.is_dark { 0x16 } else { 0x0c },
        )))
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg_dim))
        .debug_selector(|| "context-error-banner".to_owned())
        .child(div().flex_1().min_w_0().truncate().child(message))
        .child(
            button("context-error-dismiss", "Dismiss", t, false).on_click(cx.listener(
                |view, _, _, cx| {
                    view.dismiss_context_expansion_error(cx);
                },
            )),
        )
        .into_any_element()
}

const SHOW_MORE_LINES: u32 = 10;
const SHOW_MORE_ID: &str = "show-10";
const SHOW_MORE_LABEL: &str = "Show 10";
const SHOW_ALL_ID: &str = "show-all";
const SHOW_ALL_LABEL: &str = "Show all";

pub(super) fn context_controls(
    scope: &'static str,
    region: ContextRegion,
    theme: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Div {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .ml(px(10.));
    if region.initial_line_count > SHOW_MORE_LINES {
        row = row.child(context_button(
            scope,
            region.id,
            SHOW_MORE_ID,
            SHOW_MORE_LABEL,
            ContextExpansion::ShowMore {
                line_count: SHOW_MORE_LINES,
            },
            theme,
            cx,
        ));
    }
    row.child(context_button(
        scope,
        region.id,
        SHOW_ALL_ID,
        SHOW_ALL_LABEL,
        ContextExpansion::ShowAll,
        theme,
        cx,
    ))
}

fn context_button(
    scope: &'static str,
    region_id: u32,
    action_id: &'static str,
    label: &'static str,
    expansion: ContextExpansion,
    theme: &Theme,
    cx: &mut Context<RepoWindow>,
) -> impl gpui::IntoElement {
    let id: SharedString = format!("diff-context-{scope}-{region_id}-{action_id}").into();
    let debug_id = id.clone();
    div()
        .id(id)
        .debug_selector(move || debug_id.to_string())
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(format!("{label} unmodified lines"))
        .flex()
        .items_center()
        .h(px(theme.scaled_control_height(16., 11.)))
        .px(px(5.))
        .rounded_sm()
        .text_size(ui_font_size(11.))
        .text_color(rgb(theme.diff_text_dim))
        .cursor_pointer()
        .hover({
            let hover = with_alpha(theme.selected_accent, 0x18);
            move |style| style.bg(rgba(hover)).underline()
        })
        .focus({
            let focus = with_alpha(theme.selected_accent, 0x26);
            move |style| style.bg(rgba(focus))
        })
        .on_click(cx.listener(move |view, _, _, cx| {
            cx.stop_propagation();
            view.expand_context(region_id, expansion, cx);
        }))
        .on_key_down(cx.listener(move |view, ev: &gpui::KeyDownEvent, _, cx| {
            if matches!(ev.keystroke.key.as_str(), "enter" | "space")
                && !ev.keystroke.modifiers.modified()
            {
                cx.stop_propagation();
                view.expand_context(region_id, expansion, cx);
            }
        }))
        .child(label)
}
