use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    UniformListScrollHandle, Window, div, px, rgb, uniform_list,
};
use jayjay_core::DiffHunk;

use super::row::{review_checkbox, row_bg, status_dot};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::ui::primitives::no_scrollbar_gutter;

// Char-based middle truncation (approx of SwiftUI's `.truncationMode(.middle)`).
fn middle_elide(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        return s.to_owned();
    }
    let keep = max_chars.saturating_sub(1);
    if keep == 0 {
        return "…".to_owned();
    }
    let head_len = keep / 2;
    let tail_len = keep - head_len;
    let head: String = chars[..head_len].iter().collect();
    let tail: String = chars[chars.len() - tail_len..].iter().collect();
    format!("{head}…{tail}")
}

#[allow(clippy::too_many_arguments)]
pub(super) fn flat_body(
    hunks: Arc<Vec<DiffHunk>>,
    selected_ix: Option<usize>,
    t: Theme,
    scroll: UniformListScrollHandle,
    change_id: Option<String>,
    show_review: bool,
    column_width: f32,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let count = hunks.len();
    // Approximate text area = column - row padding/checkbox/dot/gaps.
    let fixed_chrome = if show_review { 80.0 } else { 56.0 };
    let text_px = (column_width - fixed_chrome).max(80.0);
    let basename_chars = ((text_px / 7.2) as usize).max(8);
    let path_chars = ((text_px / 6.0) as usize).max(10);
    let list = uniform_list(
        "files-flat",
        count,
        cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
            let t = t.clone();
            let change_id = change_id.clone();
            range
                .map(|ix| {
                    let is_selected = selected_ix == Some(ix);
                    let hunk = &hunks[ix];
                    let path = hunk.path.clone();
                    let identity = hunk.review_identity.clone();
                    let path_for_review = path.clone();
                    let identity_for_review = identity.clone();
                    let change_for_review = change_id.clone();
                    let reviewed = match change_id.as_ref() {
                        Some(cid) => this.is_reviewed(cid, &path, &identity),
                        None => false,
                    };
                    flat_file_row(
                        hunk,
                        is_selected,
                        reviewed,
                        show_review,
                        ix,
                        basename_chars,
                        path_chars,
                        &t,
                        cx.listener(move |view, _event, _window, cx| {
                            view.select_file(ix, cx);
                        }),
                        cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                            let items = RepoWindow::build_file_menu(&path);
                            view.open_context_menu(ev.position, items, cx);
                        }),
                        cx.listener(move |view, _event: &ClickEvent, _w, cx| {
                            if let Some(cid) = change_for_review.clone() {
                                view.toggle_reviewed(
                                    cid,
                                    path_for_review.clone(),
                                    identity_for_review.clone(),
                                    cx,
                                );
                            }
                        }),
                    )
                })
                .collect()
        }),
    )
    .track_scroll(&scroll);
    no_scrollbar_gutter(list).h_full().into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn flat_file_row<F, FR, FRev>(
    hunk: &DiffHunk,
    is_selected: bool,
    reviewed: bool,
    show_review: bool,
    ix: usize,
    basename_chars: usize,
    path_chars: usize,
    t: &Theme,
    on_click: F,
    on_right_click: FR,
    on_review_click: FRev,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    FR: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    FRev: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let bg_row = row_bg(is_selected, ix, t);

    let basename = middle_elide(
        hunk.path.rsplit('/').next().unwrap_or(&hunk.path),
        basename_chars,
    );
    let path_display = middle_elide(&hunk.path, path_chars);

    let name_color = if reviewed { t.fg_faint } else { t.fg };
    let content = div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .flex_1()
        .min_w_0()
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(12.))
                .text_color(rgb(name_color))
                .child(SharedString::from(basename)),
        )
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(10.))
                .text_color(rgb(t.fg_faint))
                .truncate()
                .child(SharedString::from(path_display)),
        );

    let mut row = div()
        .id(("file", ix))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(50.))
        .gap(px(8.))
        .px(px(10.))
        .bg(rgb(bg_row))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .cursor_pointer()
        .on_click(on_click)
        .on_mouse_down(MouseButton::Right, on_right_click);
    if show_review {
        row = row.child(review_checkbox(
            ("review-flat", ix),
            reviewed,
            t,
            on_review_click,
        ));
    }
    row.child(status_dot(hunk, t))
        .child(content)
        .into_any_element()
}
