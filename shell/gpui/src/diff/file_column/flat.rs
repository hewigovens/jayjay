use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    UniformListScrollHandle, Window, div, px, rgb, uniform_list,
};
use jayjay_core::DiffHunk;

use super::row::{
    FileRowHandlers, FileRowState, file_name_opacity, file_row_height, file_text_content,
    file_text_inset, file_text_limits, finish_file_row, review_checkbox, row_bg, row_separator,
};
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::ui::primitives::no_scrollbar_gutter;

pub(crate) fn middle_elide(s: &str, max_chars: usize) -> String {
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

pub(super) struct FlatBodyState {
    pub(super) hunks: Arc<Vec<DiffHunk>>,
    pub(super) visible_indices: Arc<Vec<usize>>,
    pub(super) selected_ix: Option<usize>,
    pub(super) multi_selected: Arc<HashSet<usize>>,
    pub(super) theme: Theme,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) change_id: Option<String>,
    pub(super) show_review: bool,
    pub(super) note_counts: Arc<HashMap<String, usize>>,
    pub(super) column_width: f32,
}

pub(super) fn flat_body(state: FlatBodyState, cx: &mut Context<RepoWindow>) -> AnyElement {
    let FlatBodyState {
        hunks,
        visible_indices,
        selected_ix,
        multi_selected,
        theme,
        scroll,
        change_id,
        show_review,
        note_counts,
        column_width,
    } = state;
    let count = visible_indices.len();
    let fixed_chrome = if show_review { 80.0 } else { 56.0 };
    let text_px = (column_width - fixed_chrome).max(80.0);
    let (basename_chars, path_chars) = file_text_limits(text_px, &theme);
    let list = uniform_list(
        "files-flat",
        count,
        cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
            let theme = theme.clone();
            let change_id = change_id.clone();
            let visible_indices = visible_indices.clone();
            let note_counts = note_counts.clone();
            let multi_selected = multi_selected.clone();
            range
                .map(|ix| {
                    let hunk_ix = visible_indices[ix];
                    let is_selected =
                        selected_ix == Some(hunk_ix) || multi_selected.contains(&hunk_ix);
                    let hunk = &hunks[hunk_ix];
                    let path = hunk.path.clone();
                    let identity = hunk.review_identity.clone();
                    let show_review = show_review && !identity.is_empty();
                    let path_for_review = path.clone();
                    let identity_for_review = identity.clone();
                    let change_for_review = change_id.clone();
                    let reviewed = match (show_review, change_id.as_ref()) {
                        (true, Some(cid)) => this.is_reviewed(cid, &path, &identity),
                        _ => false,
                    };
                    let note_count = note_counts.get(&path).copied().unwrap_or(0);
                    flat_file_row(
                        FileRowState {
                            hunk,
                            is_selected,
                            reviewed,
                            show_review,
                            note_count,
                            ix: hunk_ix,
                            theme: &theme,
                        },
                        basename_chars,
                        path_chars,
                        FileRowHandlers {
                            on_click: cx.listener(move |view, event: &ClickEvent, _window, cx| {
                                view.handle_file_row_click(hunk_ix, event.modifiers(), cx);
                            }),
                            on_right_click: cx.listener(
                                move |view, ev: &MouseDownEvent, _w, cx| {
                                    view.open_file_context_menu(&path, ev.position, cx);
                                },
                            ),
                            on_review_click: cx.listener(
                                move |view, _event: &ClickEvent, _w, cx| {
                                    if let Some(cid) = change_for_review.clone() {
                                        view.toggle_reviewed(
                                            cid,
                                            path_for_review.clone(),
                                            identity_for_review.clone(),
                                            cx,
                                        );
                                    }
                                },
                            ),
                        },
                    )
                })
                .collect()
        }),
    )
    .track_scroll(&scroll);
    no_scrollbar_gutter(list).h_full().into_any_element()
}

fn flat_file_row<F, FR, FRev>(
    state: FileRowState<'_>,
    basename_chars: usize,
    path_chars: usize,
    handlers: FileRowHandlers<F, FR, FRev>,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    FR: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    FRev: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let FileRowState {
        hunk,
        is_selected,
        reviewed,
        show_review,
        note_count,
        ix,
        theme,
    } = state;
    let FileRowHandlers {
        on_click,
        on_right_click,
        on_review_click,
    } = handlers;
    let bg_row = row_bg(is_selected, theme);

    let basename = middle_elide(
        hunk.path.rsplit('/').next().unwrap_or(&hunk.path),
        basename_chars,
    );
    let path_display = middle_elide(&hunk.path, path_chars);

    let name_opacity = file_name_opacity(show_review, reviewed);
    let content = file_text_content(
        SharedString::from(basename),
        SharedString::from(path_display),
        name_opacity,
        theme,
    );

    let mut row = div()
        .id(("file", ix))
        .debug_selector(move || format!("file-row-{ix}"))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(file_row_height(theme)))
        .gap(px(8.))
        .mx(px(4.))
        .px(px(6.))
        .rounded_md()
        .bg(rgb(bg_row))
        .relative()
        .cursor_pointer()
        .on_click(on_click)
        .on_mouse_down(MouseButton::Right, on_right_click)
        .child(row_separator(6. + file_text_inset(show_review), theme));
    if show_review {
        row = row.child(review_checkbox(
            ("review-flat", ix),
            reviewed,
            theme,
            on_review_click,
        ));
    }
    finish_file_row(row, hunk, content, note_count, theme)
}
