use gpui::{
    AnyElement, ClipboardItem, Context, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::{BookmarkInfo, ChangeInfo, DiffStats};

use crate::app::theme::{FONT_META, Theme};
use crate::repo::RepoWindow;
use crate::repo::window::dag_row::format_when;
use crate::ui::icons::glyph;
use crate::ui::primitives::{capsule, icon_button, icon_chip};

const LABEL_WIDTH: f32 = 70.;

pub(super) fn metadata_block(
    change: &ChangeInfo,
    stats: Option<&DiffStats>,
    recently_copied: Option<&SharedString>,
    bookmarks: &[BookmarkInfo],
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
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
        .child(id_prefix_row(
            "Change",
            change.change_id.chars().take(24).collect::<String>(),
            change.change_id.id.clone(),
            change.change_id.short_len,
            recently_copied,
            t,
            cx,
        ))
        .child(id_prefix_row(
            "Commit",
            change.commit_id.chars().take(12).collect::<String>(),
            change.commit_id.id.clone(),
            change.commit_id.short_len,
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
        metadata = metadata.child(bookmarks_row(change, recently_copied, bookmarks, t, cx));
    }

    if let Some(stats) = stats {
        metadata = metadata.child(changes_row(stats, t));
    }

    metadata.into_any_element()
}

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
    change: &ChangeInfo,
    recently_copied: Option<&SharedString>,
    bookmarks: &[BookmarkInfo],
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .child(label_cell("Bookmarks", t));

    for name in &change.bookmarks {
        let id: SharedString = format!("bookmark:{name}").into();
        let just_copied = recently_copied == Some(&id);
        let conflicted = BookmarkInfo::is_conflicted_name(bookmarks, name);
        let chip = if conflicted {
            icon_chip(
                glyph::WARNING,
                SharedString::from(name.clone()),
                t.tag_divergent_bg,
                t.tag_divergent_fg,
                t.tag_divergent_fg,
                FONT_META,
            )
            .into_any_element()
        } else {
            capsule(
                SharedString::from(name.clone()),
                t.tag_bookmark_bg,
                t.tag_bookmark_fg,
                FONT_META,
            )
            .into_any_element()
        };
        row = row.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .child(chip)
                .child(copy_button(name.clone(), id, just_copied, t, cx)),
        );
    }
    row.into_any_element()
}

fn author_row(
    change: &ChangeInfo,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
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
        .child(crate::ui::avatar::element(
            &change.author.email,
            &change.author.name,
            18.,
        ))
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

fn meta_row(
    label: &'static str,
    value: String,
    mono: bool,
    copyable: bool,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
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

/// A metadata row showing `value` with its shortest unique prefix bold.
fn id_prefix_row(
    label: &'static str,
    value: String,
    copy_value: String,
    short_len: u32,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let (prefix, rest) = crate::repo::window::dag_row::split_prefix(&value, short_len);
    let value_el = div()
        .flex()
        .flex_row()
        .flex_none()
        .font_family(crate::app::fonts::mono())
        .text_size(px(FONT_META))
        .child(
            div()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(t.fg))
                .child(SharedString::from(prefix)),
        )
        .child(div().text_color(rgb(t.fg)).child(SharedString::from(rest)));

    let id: SharedString = label.into();
    let just_copied = recently_copied == Some(&id);
    div()
        .flex()
        .flex_row()
        .items_baseline()
        .gap(px(6.))
        .child(label_cell(label, t))
        .child(value_el)
        .child(copy_button(copy_value, id, just_copied, t, cx))
        .child(div().flex_1())
        .into_any_element()
}

fn copy_button(
    value: String,
    id: SharedString,
    just_copied: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let (glyph_str, color) = if just_copied {
        (glyph::CHECK, t.success_fg)
    } else {
        (glyph::COPY, t.fg_faint)
    };
    let id_for_click = id.clone();
    icon_button(
        SharedString::from(format!("copy-{id}")),
        glyph_str,
        12.,
        20.,
        20.,
        color,
        t,
    )
    .on_click(cx.listener(move |view, _, _, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
        view.mark_copied(id_for_click.clone(), cx);
    }))
    .into_any_element()
}
