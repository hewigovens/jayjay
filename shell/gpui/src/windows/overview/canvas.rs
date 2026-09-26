use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, ClickEvent, Context, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    canvas, div, px, rgb,
};
use jayjay_core::overview::{OverviewGroup, OverviewSnapshot};
use jayjay_core::theme::mix;
use jayjay_core::{
    OverviewBase, OverviewBaseKind, OverviewChange, OverviewLane, OverviewWorkspace,
};

use super::OverviewView;
use super::placement::{Geometry, Placement};
use crate::app::theme::{FONT_TAG, Theme, ui_font_size};
use crate::repo::window::{
    DagNodeStyle, LinePattern, NodeFill, NodeShape, chip_width, compact_id, format_relative,
    id_cell, paint_node, stroke_line_pattern, visible_chip_count,
};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::capsule;

impl OverviewView {
    pub(super) fn render_tree(
        &self,
        snapshot: &OverviewSnapshot,
        groups: &[OverviewGroup],
        placement: &Placement,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let lanes = &snapshot.overview.lanes;
        let trunk_name = snapshot.trunk_name();
        let mut tree = div()
            .relative()
            .w(px(placement.width))
            .h(px(placement.height))
            .child(lines(placement, groups, t));
        let g = &placement.geometry;
        for band in &placement.bands {
            let base = &groups[band.group].base;
            tree = tree
                .child(
                    trunk_label(base, trunk_name, t)
                        .absolute()
                        .left(px(g.spine_x + 12.))
                        .top(px(band.y - g.row_height - 2.))
                        .w(px(g.trunk_label_width))
                        .h(px(g.row_height)),
                )
                .child(
                    base_detail(base, t)
                        .absolute()
                        .left(px(g.spine_x + 12.))
                        .top(px(band.y + 4.))
                        .w(px(g.trunk_label_width)),
                );
        }
        for placed in &placement.lanes {
            let lane_ix = placed.lane;
            let lane = &lanes[lane_ix];
            let lane_id = &self.lane_ids[lane_ix];
            let selected = self.selection.lane.as_deref() == Some(lane_id);
            tree = tree.child(
                self.lane_card(lane_ix, lane, selected, g, t, cx)
                    .absolute()
                    .left(px(placed.x))
                    .top(px(placed.card_top(g)))
                    .w(px(g.column_width))
                    .h(px(g.card_height)),
            );
            for (offset, change) in lane.changes.iter().enumerate() {
                let chosen = self.selection.change.as_deref() == Some(&change.commit_id.id);
                tree = tree.child(
                    self.change_row(lane_ix, change, chosen, g, t, cx)
                        .absolute()
                        .left(px(placed.x))
                        .top(px(placed.node_y(g, offset) - g.row_height / 2.))
                        .w(px(g.column_width))
                        .h(px(g.row_height)),
                );
            }
        }
        tree.into_any_element()
    }

    fn lane_card(
        &self,
        lane_ix: usize,
        lane: &OverviewLane,
        selected: bool,
        g: &Geometry,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let needs_attention = !lane.attention.is_empty();
        let border = if selected {
            t.selected_accent
        } else if needs_attention {
            t.warning_fg
        } else {
            t.border
        };
        let count = lane.changes.len();
        let footer = lane.attention.first().cloned().unwrap_or_else(|| {
            format!(
                "{count} change{} · {}",
                if count == 1 { "" } else { "s" },
                format_relative(lane.latest_timestamp_millis)
            )
        });
        // GPUI borders sit inside the box, so the padding gives back what a thicker border takes.
        let inset = if selected || needs_attention { 1. } else { 0. };
        let head = lane.head().change_id.unique_prefix();
        div()
            .id(ElementId::named_usize("overview-lane", lane_ix))
            .debug_selector(move || format!("overview-lane-{head}"))
            .flex()
            .flex_col()
            .gap(px(4.))
            .pl(px(12. - inset))
            .pr(px(10. - inset))
            .py(px(8. - inset))
            .rounded(px(8.))
            .bg(rgb(t.detail_bg))
            .border(px(1. + inset))
            .border_color(rgb(border))
            .cursor_pointer()
            .child(
                div()
                    .h(px(t.scaled_font_size(32.)))
                    .text_size(ui_font_size(12.))
                    .line_height(px(t.scaled_font_size(16.)))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(description_color(&lane.head().description, t)))
                    .line_clamp(2)
                    .child(SharedString::from(lane.title().to_owned())),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(CHIP_GAP))
                    .h(px(t.scaled_font_size(16.)))
                    .overflow_hidden()
                    .children(lane_chips(lane, g.column_width - 24., t)),
            )
            .child(
                div()
                    .text_size(ui_font_size(10.))
                    .text_color(rgb(if needs_attention {
                        t.warning_fg
                    } else {
                        t.fg_dim
                    }))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(SharedString::from(footer)),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |view, event: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    view.open_lane_menu(lane_ix, event.position, cx);
                }),
            )
            .on_click(cx.listener(move |view, event: &ClickEvent, _, cx| {
                cx.stop_propagation();
                view.select_lane(lane_ix, cx);
                view.selection.lane_panel = true;
                if event.click_count() == 2 {
                    view.show_in_graph(lane_ix, None, cx);
                }
            }))
    }

    fn change_row(
        &self,
        lane_ix: usize,
        change: &OverviewChange,
        chosen: bool,
        g: &Geometry,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let commit_id = change.commit_id.id.clone();
        let menu_change = change.clone();
        div()
            .id(SharedString::from(format!("overview-change-{commit_id}")))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .pr(px(4.))
            .rounded(px(4.))
            .when(chosen, |row| row.bg(rgb(t.selected_bg)))
            .cursor_pointer()
            .child(
                node(change, t)
                    .w(px(g.node_inset * 2.))
                    .h_full()
                    .flex_none(),
            )
            .child(id_cell(
                &compact_id(&change.change_id),
                change.change_id.short_len,
                t.change_id_prefix,
                10.,
                t,
            ))
            .children(
                change
                    .has_conflict
                    .then(|| icons::icon(glyph::WARNING, 10., t.tag_conflict_fg).flex_none()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(ui_font_size(11.))
                    .text_color(rgb(description_color(&change.description, t)))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(SharedString::from(change.title().to_owned())),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |view, event: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    view.open_change_menu(lane_ix, &menu_change, event.position, cx);
                }),
            )
            .on_click(cx.listener(move |view, event: &ClickEvent, _, cx| {
                cx.stop_propagation();
                view.select_change(lane_ix, commit_id.clone(), cx);
                if event.click_count() == 2 {
                    view.show_in_graph(lane_ix, Some(commit_id.clone()), cx);
                }
            }))
    }
}

fn lines(placement: &Placement, groups: &[OverviewGroup], t: &Theme) -> AnyElement {
    let g = placement.geometry;
    let line = t.dag_line;
    let muted = mix(t.dag_line, t.sidebar_bg, 0.45);
    let background = t.sidebar_bg;
    let warning = t.warning_fg;
    let stems: Vec<(f32, f32, f32)> = placement
        .lanes
        .iter()
        .map(|placed| {
            (
                placed.x + g.node_inset,
                placed.card_top(&g) + g.card_height,
                placed.band_y,
            )
        })
        .collect();
    let bands: Vec<(f32, f32, bool)> = placement
        .bands
        .iter()
        .map(|band| {
            let older = groups[band.group].base.kind == OverviewBaseKind::OlderTrunk;
            (band.y, band.last_x, older)
        })
        .collect();
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let ox = bounds.origin.x;
            let oy = bounds.origin.y;
            let at = |x: f32, y: f32| (ox + px(x), oy + px(y));
            for &(x, top, bottom) in &stems {
                let (x0, y0) = at(x, top);
                let (x1, y1) = at(x, bottom);
                stroke_line_pattern(window, x0, y0, x1, y1, muted, LinePattern::Solid);
            }
            if let (Some(first), Some(last)) = (bands.first(), bands.last())
                && first.0 != last.0
            {
                let (x0, y0) = at(g.spine_x, first.0 + 6.);
                let (x1, y1) = at(g.spine_x, last.0 - 6.);
                stroke_line_pattern(
                    window,
                    x0,
                    y0,
                    x1,
                    y1,
                    muted,
                    LinePattern::Dashed(&[3., 4.]),
                );
            }
            for &(y, last_x, older) in &bands {
                let (x0, y0) = at(g.spine_x + 6., y);
                let (x1, _) = at(last_x, y);
                stroke_line_pattern(window, x0, y0, x1, y0, muted, LinePattern::Solid);
                let (cx, cy) = at(g.spine_x, y);
                for fill in [
                    NodeFill::Filled(background),
                    NodeFill::Outlined(if older { warning } else { line }, 1.8),
                ] {
                    paint_node(
                        window,
                        cx,
                        cy,
                        DagNodeStyle {
                            shape: NodeShape::Diamond,
                            radius: 6.,
                            fill,
                        },
                    );
                }
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

fn node(change: &OverviewChange, t: &Theme) -> gpui::Div {
    let radius = if change.workspaces.is_empty() {
        4.
    } else {
        4.5
    };
    let hollow = change.is_empty && change.workspaces.is_empty() && !change.has_conflict;
    let fill = if change.has_conflict {
        t.tag_conflict_fg
    } else if !change.workspaces.is_empty() {
        t.tag_wc_fg
    } else if change.is_empty {
        t.sidebar_bg
    } else {
        t.fg_dim
    };
    div().flex().items_center().justify_center().child(
        div()
            .size(px(radius * 2.))
            .rounded_full()
            .bg(rgb(fill))
            .when(hollow, |dot| {
                dot.border(px(1.5)).border_color(rgb(t.fg_dim))
            }),
    )
}

const CHIP_GAP: f32 = 4.;

pub(super) fn description_color(description: &str, t: &Theme) -> u32 {
    if description.is_empty() {
        t.fg_faint
    } else {
        t.fg
    }
}

pub(super) fn workspace_label(workspace: &OverviewWorkspace) -> String {
    if workspace.is_current {
        format!("@ {}", workspace.name)
    } else {
        format!("{}@", workspace.name)
    }
}

fn lane_chips(lane: &OverviewLane, budget: f32, t: &Theme) -> Vec<AnyElement> {
    let workspaces = lane
        .workspaces
        .iter()
        .map(|workspace| (workspace_label(workspace), t.tag_wc_bg, t.tag_wc_fg));
    let bookmarks = lane
        .changes
        .iter()
        .flat_map(|change| &change.bookmarks)
        .map(|bookmark| (bookmark.clone(), t.tag_bookmark_bg, t.tag_bookmark_fg));
    let chips: Vec<_> = workspaces.chain(bookmarks).collect();
    let widths: Vec<f32> = chips
        .iter()
        .map(|(label, ..)| chip_width(label, false, t))
        .collect();
    let shown = visible_chip_count(&widths, budget, CHIP_GAP, |hidden| {
        chip_width(&format!("+{hidden}"), false, t)
    });
    let mut elements: Vec<AnyElement> = chips
        .into_iter()
        .take(shown)
        .map(|(label, bg, fg)| {
            capsule(label, bg, fg, FONT_TAG)
                .min_w_0()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis_middle()
                .into_any_element()
        })
        .collect();
    let hidden = widths.len() - shown;
    if hidden > 0 {
        elements
            .push(capsule(format!("+{hidden}"), t.tag_bg, t.tag_fg, FONT_TAG).into_any_element());
    }
    elements
}

fn trunk_label(base: &OverviewBase, trunk_name: &str, t: &Theme) -> gpui::Div {
    let (label, tint) = match base.kind {
        OverviewBaseKind::Trunk => (
            base.bookmarks
                .first()
                .cloned()
                .unwrap_or_else(|| trunk_name.to_owned()),
            t.tag_wc_fg,
        ),
        OverviewBaseKind::OlderTrunk => (
            format!("{} behind {trunk_name}", base.behind_trunk),
            t.warning_fg,
        ),
        OverviewBaseKind::Mutable => ("fork point".to_owned(), t.fg_dim),
        OverviewBaseKind::Other => (
            base.bookmarks
                .first()
                .cloned()
                .unwrap_or_else(|| base.change_id.unique_prefix()),
            t.fg_dim,
        ),
    };
    div()
        .flex()
        .items_center()
        .px(px(4.))
        .text_size(ui_font_size(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(tint))
        .whitespace_nowrap()
        .overflow_hidden()
        .text_ellipsis()
        .child(SharedString::from(label))
}

fn base_detail(base: &OverviewBase, t: &Theme) -> gpui::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(4.))
        .child(id_cell(
            &compact_id(&base.commit_id),
            base.commit_id.short_len,
            t.commit_id_prefix,
            10.,
            t,
        ))
        .child(
            div()
                .text_size(ui_font_size(10.))
                .text_color(rgb(t.fg_faint))
                .whitespace_nowrap()
                .child(SharedString::from(format_relative(base.timestamp_millis))),
        )
}
