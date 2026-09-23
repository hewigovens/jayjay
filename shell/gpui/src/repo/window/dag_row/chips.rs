use gpui::{
    AnyElement, AppContext, InteractiveElement, IntoElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled, div, px,
};
use jayjay_core::{BookmarkInfo, ChangeInfo};

use super::row::ChipRightClick;
use super::text::{compact_id, id_cell};
use crate::app::theme::{FONT_ID, FONT_TAG, Theme};
use crate::repo::window::dag_drag::{DagDrag, DagDragGhost};
use crate::repo::window::ref_chips;
use crate::ui::primitives::{capsule, text_tooltip};

const CHIP_GAP: f32 = 5.;
const CHAR_WIDTH_FACTOR: f32 = 0.6;

enum DagChip {
    WorkingCopy,
    Conflict,
    Divergent,
    Bookmark(usize),
    GitTag(usize),
    Workspace(usize),
}

impl DagChip {
    fn label(&self, change: &ChangeInfo) -> String {
        match self {
            DagChip::WorkingCopy => "@".to_owned(),
            DagChip::Conflict => "conflict".to_owned(),
            DagChip::Divergent => "divergent".to_owned(),
            DagChip::Bookmark(ix) => change.bookmarks[*ix].clone(),
            DagChip::GitTag(ix) => change.tags[*ix].clone(),
            DagChip::Workspace(ix) => format!("{}@", change.workspaces[*ix]),
        }
    }

    fn has_icon(&self) -> bool {
        matches!(self, DagChip::Bookmark(_) | DagChip::GitTag(_))
    }
}

fn dag_chips(change: &ChangeInfo) -> Vec<DagChip> {
    let mut chips = Vec::new();
    if change.is_working_copy {
        chips.push(DagChip::WorkingCopy);
    }
    if change.has_conflict {
        chips.push(DagChip::Conflict);
    }
    if change.is_divergent {
        chips.push(DagChip::Divergent);
    }
    chips.extend((0..change.bookmarks.len()).map(DagChip::Bookmark));
    chips.extend((0..change.tags.len()).map(DagChip::GitTag));
    chips.extend((0..change.workspaces.len()).map(DagChip::Workspace));
    chips
}

fn chip_width(label: &str, has_icon: bool, t: &Theme) -> f32 {
    let text = label.chars().count() as f32 * t.scaled_font_size(FONT_TAG) * CHAR_WIDTH_FACTOR;
    text + 12.
        + if has_icon {
            t.scaled_font_size(FONT_TAG) + 3.
        } else {
            0.
        }
}

fn visible_chip_count(
    widths: &[f32],
    budget: f32,
    gap: f32,
    overflow_width: impl Fn(usize) -> f32,
) -> usize {
    if widths.is_empty() {
        return 0;
    }
    (1..=widths.len())
        .rev()
        .find(|&k| {
            let shown = widths[..k].iter().sum::<f32>() + gap * (k - 1) as f32;
            let hidden = widths.len() - k;
            let overflow = if hidden > 0 {
                gap + overflow_width(hidden)
            } else {
                0.
            };
            shown + overflow <= budget
        })
        .unwrap_or(1)
}

pub(super) fn tags_row(
    change: &ChangeInfo,
    row_ix: usize,
    t: &Theme,
    bookmarks: &[BookmarkInfo],
    refs_budget: f32,
    on_bookmark_right_click: ChipRightClick,
    on_workspace_right_click: ChipRightClick,
) -> impl IntoElement {
    let short_id = compact_id(&change.change_id);
    let change_id_width =
        short_id.chars().count() as f32 * t.scaled_font_size(FONT_ID) * CHAR_WIDTH_FACTOR;
    let chips_budget = (refs_budget - change_id_width - CHIP_GAP).max(0.);
    let chips = dag_chips(change);
    let widths: Vec<f32> = chips
        .iter()
        .map(|chip| chip_width(&chip.label(change), chip.has_icon(), t))
        .collect();
    let overflow_width = |hidden: usize| chip_width(&format!("+{hidden}"), false, t);
    let visible = visible_chip_count(&widths, chips_budget, CHIP_GAP, overflow_width);
    let hidden = chips.len() - visible;
    let shown_budget = if hidden > 0 {
        (chips_budget - CHIP_GAP - overflow_width(hidden)).max(0.)
    } else {
        chips_budget
    };
    let squeeze_first = visible == 1 && widths[0] > shown_budget;

    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(CHIP_GAP))
        .child(id_cell(
            &short_id,
            change.change_id.short_len,
            if change.is_immutable {
                t.fg_dim
            } else {
                t.change_id_prefix
            },
            FONT_ID,
            t,
        ));

    for (c_ix, chip) in chips.iter().take(visible).enumerate() {
        let element = chip_element(
            chip,
            change,
            row_ix,
            t,
            bookmarks,
            &on_bookmark_right_click,
            &on_workspace_right_click,
        );
        if c_ix == 0 && squeeze_first {
            row = row.child(
                div()
                    .flex_none()
                    .max_w(px(shown_budget))
                    .overflow_hidden()
                    .child(element),
            );
        } else {
            row = row.child(element);
        }
    }
    if hidden > 0 {
        let names = chips[visible..]
            .iter()
            .map(|chip| chip.label(change))
            .collect::<Vec<_>>()
            .join(", ");
        row = row.child(
            capsule(format!("+{hidden}"), t.row_alt_bg, t.fg, FONT_TAG)
                .id(("overflow", row_ix))
                .tooltip(text_tooltip(names)),
        );
    }
    row
}

fn chip_element(
    chip: &DagChip,
    change: &ChangeInfo,
    row_ix: usize,
    t: &Theme,
    bookmarks: &[BookmarkInfo],
    on_bookmark_right_click: &ChipRightClick,
    on_workspace_right_click: &ChipRightClick,
) -> AnyElement {
    match *chip {
        DagChip::WorkingCopy => working_copy_chip(row_ix, t).into_any_element(),
        DagChip::Conflict => {
            capsule("conflict", t.tag_conflict_bg, t.tag_conflict_fg, FONT_TAG).into_any_element()
        }
        DagChip::Divergent => capsule(
            "divergent",
            t.tag_divergent_bg,
            t.tag_divergent_fg,
            FONT_TAG,
        )
        .into_any_element(),
        DagChip::Bookmark(ix) => {
            let name = change.bookmarks[ix].clone();
            bookmark_chip(
                row_ix,
                ix,
                name.clone(),
                BookmarkInfo::is_conflicted_name(bookmarks, &name),
                t,
                on_bookmark_right_click.clone(),
            )
            .into_any_element()
        }
        DagChip::GitTag(ix) => tag_chip(change.tags[ix].clone(), t).into_any_element(),
        DagChip::Workspace(ix) => workspace_chip(
            row_ix,
            ix,
            change.workspaces[ix].clone(),
            t,
            on_workspace_right_click.clone(),
        )
        .into_any_element(),
    }
}

fn bookmark_chip(
    row_ix: usize,
    b_ix: usize,
    name: String,
    conflicted: bool,
    t: &Theme,
    on_right_click: ChipRightClick,
) -> impl IntoElement {
    let drag_name = name.clone();
    let debug_name = name.clone();
    ref_chips::bookmark_chip(name.clone().into(), conflicted, true, FONT_TAG, t)
        .id(("bm", row_ix * 16 + b_ix))
        .debug_selector(move || format!("dag-bookmark-{debug_name}"))
        .cursor_move()
        .on_drag(
            DagDrag::Bookmark {
                name: drag_name.clone(),
                conflicted,
            },
            move |drag: &DagDrag, _offset, _w, cx| cx.new(|_| DagDragGhost::new(drag.clone())),
        )
        .on_mouse_down(MouseButton::Right, move |ev, w, cx| {
            cx.stop_propagation();
            on_right_click(&name, ev, w, cx);
        })
}

fn working_copy_chip(row_ix: usize, t: &Theme) -> impl IntoElement {
    div()
        .id(("wc", row_ix))
        .debug_selector(|| "dag-working-copy".to_owned())
        .cursor_move()
        .on_drag(
            DagDrag::WorkingCopy,
            move |drag: &DagDrag, _offset, _w, cx| cx.new(|_| DagDragGhost::new(drag.clone())),
        )
        .child(capsule("@", t.tag_wc_bg, t.tag_wc_fg, FONT_TAG))
}

fn workspace_chip(
    row_ix: usize,
    w_ix: usize,
    name: String,
    t: &Theme,
    on_right_click: ChipRightClick,
) -> impl IntoElement {
    let debug_name = name.clone();
    div()
        .id(("ws", row_ix * 16 + w_ix))
        .debug_selector(move || format!("dag-workspace-{debug_name}"))
        .child(capsule(
            format!("{name}@"),
            t.tag_wc_bg,
            t.tag_wc_fg,
            FONT_TAG,
        ))
        .on_mouse_down(MouseButton::Right, move |ev, w, cx| {
            cx.stop_propagation();
            on_right_click(&name, ev, w, cx);
        })
}

fn tag_chip(name: String, t: &Theme) -> impl IntoElement {
    ref_chips::tag_chip(name.into(), FONT_TAG, t)
}

#[cfg(test)]
mod tests {
    use super::visible_chip_count;

    #[test]
    fn visible_chips_fill_the_budget_then_collapse_to_one_overflow_chip() {
        let widths = [20., 20., 20., 20., 20.];
        let overflow = |hidden: usize| 10. + hidden as f32;
        assert_eq!(visible_chip_count(&widths, 120., 5., overflow), 5);
        assert_eq!(visible_chip_count(&widths, 119., 5., overflow), 4);
        assert_eq!(visible_chip_count(&widths, 80., 5., overflow), 2);
        assert_eq!(visible_chip_count(&widths, 10., 5., overflow), 1);
        assert_eq!(visible_chip_count(&[], 10., 5., overflow), 0);
    }
}
