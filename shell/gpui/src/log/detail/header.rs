use gpui::{
    AnyElement, ClipboardItem, Context, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::{ChangeInfo, DiffStats};

use super::description::description_block;
use crate::app::theme::{FONT_META, Theme};
use crate::log::LogView;
use crate::log::commit_row::format_when;
use crate::repo::revset::CompareState;
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::capsule;

const LABEL_WIDTH: f32 = 70.;

pub(super) fn detail_header(
    change: &ChangeInfo,
    stats: Option<&DiffStats>,
    compare: Option<&CompareState>,
    file_count: Option<usize>,
    recently_copied: Option<&SharedString>,
    description_height: f32,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    if let Some(compare) = compare {
        return div()
            .flex()
            .flex_col()
            .bg(rgb(t.detail_bg))
            .child(compare_banner(compare, file_count, t, cx))
            .into_any_element();
    }

    let parents = if change.parents.is_empty() {
        String::from("—")
    } else {
        change
            .parents
            .iter()
            .map(|p| p.chars().take(12).collect::<String>())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let mut metadata = div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(meta_row(
            "Change",
            change.change_id.chars().take(24).collect::<String>(),
            true,
            true,
            recently_copied,
            t,
            cx,
        ))
        .child(meta_row(
            "Commit",
            change.commit_id.chars().take(12).collect::<String>(),
            true,
            true,
            recently_copied,
            t,
            cx,
        ))
        .child(author_row(change, recently_copied, t, cx))
        .child(meta_row(
            "Date",
            format_when(change.author.timestamp_millis),
            false,
            false,
            recently_copied,
            t,
            cx,
        ))
        .child(meta_row(
            "Parents",
            parents,
            true,
            true,
            recently_copied,
            t,
            cx,
        ));

    if !change.bookmarks.is_empty() {
        metadata = metadata.child(bookmarks_row(&change.bookmarks, recently_copied, t, cx));
    }

    if let Some(s) = stats {
        metadata = metadata.child(changes_row(s, t));
    }

    let description = description_block(change, description_height, t, cx);

    let header = div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(16.))
        .py(px(12.))
        .bg(rgb(t.detail_bg));

    header.child(metadata).child(description).into_any_element()
}

fn compare_banner(
    compare: &CompareState,
    file_count: Option<usize>,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let can_reverse = compare.source_change_id.is_some() && compare.target_change_id.is_some();
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(14.))
        .py(px(8.))
        .bg(rgb(t.compare_bg))
        .child(compare_direction_button(can_reverse, t, cx))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(t.fg))
                .child(SharedString::from(compare.display.title.clone())),
        )
        .child(compare_label(&compare.display.from, t))
        .child(icon(glyph::ARROW_RIGHT, 10., t.fg_dim))
        .child(compare_label(&compare.display.to, t))
        .child(div().flex_1());

    if let Some(file_count) = file_count {
        row = row.child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(files_changed_label(file_count))),
        );
    }

    row.child(
        div()
            .id(SharedString::from("compare-close"))
            .flex()
            .items_center()
            .justify_center()
            .size(px(18.))
            .rounded_sm()
            .cursor_pointer()
            .on_click(cx.listener(|view, _, _window, cx| {
                view.vm.update(cx, |vm, cx| vm.clear_compare(cx));
            }))
            .child(icon(glyph::X_CIRCLE, 15., t.fg_dim)),
    )
    .into_any_element()
}

fn compare_direction_button(can_reverse: bool, t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    let button = div()
        .id(SharedString::from("compare-reverse"))
        .flex()
        .items_center()
        .justify_center()
        .size(px(20.))
        .rounded_sm()
        .child(icon(glyph::ARROWS_LEFT_RIGHT, 17., t.compare_accent));

    if can_reverse {
        button
            .cursor_pointer()
            .hover(|s| s.bg(rgb(t.row_alt_bg)))
            .on_click(cx.listener(|view, _, _window, cx| {
                view.vm.update(cx, |vm, cx| vm.reverse_compare(cx));
            }))
            .into_any_element()
    } else {
        button.into_any_element()
    }
}

fn compare_label(label: &str, t: &Theme) -> AnyElement {
    div()
        .max_w(px(180.))
        .overflow_hidden()
        .font_family(crate::app::fonts::mono())
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_size(px(12.))
        .text_color(rgb(t.fg))
        .child(SharedString::from(label.to_owned()))
        .into_any_element()
}

fn files_changed_label(file_count: usize) -> String {
    if file_count == 1 {
        "1 file changed".to_string()
    } else {
        format!("{file_count} files changed")
    }
}

/// Right-aligned label cell — mirrors SwiftUI's `.frame(width: 70, alignment: .trailing)`.
fn label_cell(label: &str, t: &Theme) -> AnyElement {
    div()
        .flex_none()
        .flex()
        .justify_end()
        .w(px(LABEL_WIDTH))
        .child(
            div()
                .text_size(px(FONT_META))
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(label.to_owned())),
        )
        .into_any_element()
}

fn changes_row(stats: &DiffStats, t: &Theme) -> AnyElement {
    let inserted = format!("+{}", stats.insertions);
    let deleted = format!("-{}", stats.deletions);
    let value = div()
        .flex()
        .flex_row()
        .gap(px(8.))
        .items_baseline()
        .font_family(crate::app::fonts::mono())
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_size(px(11.))
        .child(
            div()
                .text_color(rgb(t.diff_gutter_added_fg))
                .child(SharedString::from(inserted)),
        )
        .child(
            div()
                .text_color(rgb(t.diff_gutter_removed_fg))
                .child(SharedString::from(deleted)),
        );

    div()
        .flex()
        .flex_row()
        .items_baseline()
        .gap(px(6.))
        .child(label_cell("Changes", t))
        .child(value)
        .into_any_element()
}

fn bookmarks_row(
    bookmarks: &[String],
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .child(label_cell("Bookmarks", t));

    for name in bookmarks {
        let id: SharedString = format!("bookmark:{name}").into();
        let just_copied = recently_copied == Some(&id);
        row = row.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .child(capsule(
                    SharedString::from(name.clone()),
                    t.tag_bookmark_bg,
                    t.tag_bookmark_fg,
                    FONT_META,
                ))
                .child(copy_button(name.clone(), id, just_copied, t, cx)),
        );
    }
    row.into_any_element()
}

fn author_row(
    change: &ChangeInfo,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let value = format!("{} <{}>", change.author.name, change.author.email);
    let id: SharedString = "Author".into();
    let just_copied = recently_copied == Some(&id);

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .child(label_cell("Author", t))
        .child(avatar_view(&change.author.email, &change.author.name, 18.))
        .child(
            div()
                .flex_none()
                .text_size(px(FONT_META))
                .text_color(rgb(t.fg))
                .child(SharedString::from(value.clone())),
        )
        .child(copy_button(value, id, just_copied, t, cx))
        .child(div().flex_1())
        .into_any_element()
}

fn avatar_view(email: &str, name: &str, size: f32) -> AnyElement {
    if let Some(path) = crate::ui::avatar::cache_path(email)
        && path.exists()
    {
        return gpui::img(path)
            .w(px(size))
            .h(px(size))
            .rounded_full()
            .into_any_element();
    }
    let bg = crate::ui::avatar::initial_color(email);
    let initial = crate::ui::avatar::initial(name);
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(size))
        .h(px(size))
        .rounded_full()
        .bg(rgb(bg))
        .text_size(px(size * 0.55))
        .text_color(rgb(0xffffff))
        .child(SharedString::from(initial.to_string()))
        .into_any_element()
}

fn meta_row(
    label: &'static str,
    value: String,
    mono: bool,
    copyable: bool,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let mut value_el = div()
        .flex_none()
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg))
        .child(SharedString::from(value.clone()));
    if mono {
        value_el = value_el.font_family(crate::app::fonts::mono());
    }

    let mut row = div()
        .flex()
        .flex_row()
        .items_baseline()
        .gap(px(6.))
        .child(label_cell(label, t))
        .child(value_el);

    if copyable {
        let id: SharedString = label.into();
        let just_copied = recently_copied == Some(&id);
        row = row.child(copy_button(value, id, just_copied, t, cx));
    }
    row.child(div().flex_1()).into_any_element()
}

fn copy_button(
    value: String,
    id: SharedString,
    just_copied: bool,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let (glyph_str, color) = if just_copied {
        (glyph::CHECK, t.success_fg)
    } else {
        (glyph::COPY, t.fg_faint)
    };
    let id_for_click = id.clone();
    div()
        .id(SharedString::from(format!("copy-{id}")))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(20.))
        .h(px(20.))
        .rounded_sm()
        .cursor_pointer()
        .text_color(rgb(color))
        .on_click(cx.listener(move |view, _, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
            view.mark_copied(id_for_click.clone(), cx);
        }))
        .child(icon(glyph_str, 12., color))
        .into_any_element()
}
