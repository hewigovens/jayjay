use gpui::{
    AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, SharedString, StatefulInteractiveElement, Styled, div, point, px, rgb,
};
use jayjay_core::{
    RevsetCompletion, RevsetFilterState, RevsetSuggestion, RevsetSuggestionKind,
    default_revset_preset, revset_presets,
};

use super::super::RepoWindow;
use super::super::picker::{self, PickerQuery};
use super::completions::completion_row;
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::input::line_input_content;

const REVSET_DOCS_URL: &str = "https://jj-vcs.github.io/jj/latest/revsets/";

pub(crate) struct RevsetPopupState {
    pub(super) query: PickerQuery,
    pub(super) error: Option<String>,
}

impl RevsetPopupState {
    pub(super) fn new(query: String, error: Option<String>) -> Self {
        let mut picker_query = PickerQuery::new();
        picker_query.input.set_text(query);
        Self {
            query: picker_query,
            error,
        }
    }
}

pub(crate) fn render_revset_popup(
    state: &RevsetPopupState,
    filter: &RevsetFilterState,
    completions: Vec<RevsetCompletion>,
    suggestions: Vec<RevsetSuggestion>,
    bar: gpui::Bounds<Pixels>,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let close_view = view.clone();
    picker::overlay(
        "revset-popup-backdrop",
        point(bar.origin.x, bar.bottom() + px(4.)),
        panel(
            state,
            filter,
            completions,
            suggestions,
            f32::from(bar.size.width),
            t,
            view,
        ),
        move |_: &MouseDownEvent, _, cx| {
            close_view.update(cx, |view, cx| view.close_revset_popup(cx));
        },
    )
}

fn panel(
    state: &RevsetPopupState,
    filter: &RevsetFilterState,
    completions: Vec<RevsetCompletion>,
    suggestions: Vec<RevsetSuggestion>,
    width: f32,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let mut rows = Vec::new();
    if !completions.is_empty() {
        rows.push(picker::section_header(
            "revset-popup-completions",
            "Completions",
            t,
        ));
    }
    let first_suggestion = completions.len();
    for (index, completion) in completions.into_iter().enumerate() {
        let view = view.clone();
        let selected = state.query.selected == Some(index);
        let picked = completion.clone();
        rows.push(completion_row(index, &completion, selected, t, move |cx| {
            view.update(cx, |view, cx| {
                view.accept_revset_popup_completion(&picked, cx)
            });
        }));
    }
    for (kind, title, id) in [
        (
            RevsetSuggestionKind::Current,
            "Current",
            "revset-popup-current",
        ),
        (
            RevsetSuggestionKind::Bookmark,
            "Bookmarks",
            "revset-popup-bookmarks",
        ),
        (
            RevsetSuggestionKind::Recent,
            "Recent",
            "revset-popup-recent",
        ),
    ] {
        let section: Vec<_> = suggestions
            .iter()
            .enumerate()
            .filter(|(_, suggestion)| suggestion.kind == kind)
            .collect();
        if section.is_empty() {
            continue;
        }
        rows.push(picker::section_header(id, title, t));
        for (index, suggestion) in section {
            rows.push(suggestion_row(
                suggestion.clone(),
                state.query.selected == Some(first_suggestion + index),
                t,
                view,
            ));
        }
    }
    if rows.is_empty() {
        rows.push(picker::empty("Return applies it as a revset", t));
    }

    div()
        .debug_selector(|| "revset-popup".to_owned())
        .flex()
        .flex_col()
        .w(px(width))
        .max_h(px(480.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_lg()
        .overflow_hidden()
        .occlude()
        .child(field(state, t))
        .children(state.error.clone().map(|error| {
            div()
                .debug_selector(|| "revset-popup-error".to_owned())
                .px(px(12.))
                .py(px(8.))
                .bg(rgb(t.tag_removed_bg))
                .font_family(fonts::mono())
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.tag_removed_fg))
                .child(SharedString::from(error))
        }))
        .child(chips(&filter.revset, t, view))
        .child(
            div()
                .id("revset-popup-rows")
                .flex()
                .flex_col()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(&state.query.scroll)
                .pb(px(4.))
                .children(rows),
        )
        .child(footer(t))
        .into_any_element()
}

fn field(state: &RevsetPopupState, t: &Theme) -> AnyElement {
    div()
        .debug_selector(|| "revset-popup-field".to_owned())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(8.))
        .h(px(40.))
        .px(px(12.))
        .border_b_1()
        .border_color(rgb(t.border))
        .font_family(fonts::mono())
        .text_size(ui_font_size(13.))
        .child(icons::icon(
            if state.error.is_some() {
                glyph::WARNING
            } else {
                glyph::FILTER
            },
            13.,
            if state.error.is_some() {
                t.tag_removed_fg
            } else {
                t.fg_dim
            },
        ))
        .child(line_input_content(
            &state.query.input,
            "Revset, bookmark or preset",
            t,
            Some("revset-popup-caret"),
        ))
        .into_any_element()
}

fn chips(current: &str, t: &Theme, view: &Entity<RepoWindow>) -> AnyElement {
    let presets = std::iter::once(default_revset_preset()).chain(revset_presets().iter().cloned());
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(px(6.))
        .p(px(12.))
        .children(presets.map(|preset| {
            let active = preset.revset == current;
            let (background, foreground) = if active {
                (t.toggle_active_bg, t.toggle_active_fg)
            } else {
                (t.toggle_inactive_bg, t.toggle_inactive_fg)
            };
            let selector = format!("revset-chip-{}", preset.id);
            let click_view = view.clone();
            div()
                .id(SharedString::from(selector.clone()))
                .debug_selector(move || selector.clone())
                .flex()
                .flex_none()
                .items_center()
                .h(px(t.scaled_control_height(24., 12.)))
                .px(px(10.))
                .rounded_full()
                .bg(rgb(background))
                .text_size(ui_font_size(12.))
                .text_color(rgb(foreground))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(t.row_alt_bg)))
                .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    click_view.update(cx, |view, cx| view.apply_revset(&preset.revset, cx));
                })
                .child(SharedString::from(preset.label))
        }))
        .into_any_element()
}

fn suggestion_row(
    suggestion: RevsetSuggestion,
    selected: bool,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let id = format!(
        "revset-popup-row-{:?}-{}",
        suggestion.kind, suggestion.title
    );
    let icon = match suggestion.kind {
        RevsetSuggestionKind::Current => None,
        RevsetSuggestionKind::Bookmark => Some(glyph::BOOKMARK),
        RevsetSuggestionKind::Recent => Some(glyph::ARROW_UTURN_BACK),
    };
    let wraps = suggestion.kind == RevsetSuggestionKind::Current;
    let click_view = view.clone();
    let title = SharedString::from(suggestion.title.clone());
    let mut row = picker::row(id, selected, 28., t)
        .h_auto()
        .min_h(px(28.))
        .py(px(6.))
        .gap(px(8.))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            click_view.update(cx, |view, cx| {
                view.activate_revset_suggestion(suggestion.clone(), cx);
            });
        });
    if let Some(icon) = icon {
        row = row.child(icons::icon(icon, 11., t.fg_dim));
    }
    let text = div()
        .min_w_0()
        .font_family(fonts::mono())
        .text_size(ui_font_size(if wraps { 11. } else { 12. }))
        .text_color(rgb(t.fg))
        .child(title);
    row.child(if wraps { text } else { text.truncate() })
        .into_any_element()
}

fn footer(t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .gap(px(14.))
        .h(px(30.))
        .px(px(14.))
        .border_t_1()
        .border_color(rgb(t.border))
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_faint))
        .child("Return to apply")
        .child("Esc to cancel")
        .child(div().flex_1())
        .child(
            div()
                .id("revset-docs")
                .cursor_pointer()
                .hover(|s| s.text_color(rgb(t.fg)))
                .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    crate::app::links::open_url(cx, REVSET_DOCS_URL);
                })
                .child("Revset language"),
        )
        .into_any_element()
}
