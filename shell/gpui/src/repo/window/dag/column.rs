use gpui::{AnyElement, ContentMask, IntoElement, Pixels, Styled, canvas, px};
use jayjay_core::dag::{DagLayout, LaneSpan};
use jayjay_core::{EdgeType, GraphEntry};

use crate::app::theme::Theme;

use super::super::dag_row::node_center_offset;
use super::paint::{LineInk, LinePattern, paint_node, stroke_curve_pattern, stroke_line_pattern};
use super::style::DagNodeStyle;

const LANE_WIDTH: f32 = 18.0;
const ACTIVE_LINE_WIDTH: f32 = 2.5;
const LEADING_PAD: f32 = 8.0;
const TRAILING_PAD: f32 = 6.0;
const OVERFLOW_DASH_PATTERN: &[f32] = &[10.0, 4.0, 10.0, 12.0];
const INDIRECT_EDGE_DASH_PATTERN: &[f32] = &[3.0, 3.0];
const MISSING_EDGE_DASH_PATTERN: &[f32] = &[2.0, 2.0];

pub(crate) fn lane_column_width(display_lane_count: usize) -> f32 {
    let lanes = display_lane_count.max(1);
    lanes as f32 * LANE_WIDTH + LEADING_PAD + TRAILING_PAD
}

pub(crate) struct DagRowLanes {
    pub row: usize,
    pub row_lane: usize,
    /// The selection's lane, drawn emphasized while every other line fades.
    pub active_lane: Option<LaneSpan>,
    pub pass_through_lanes: Vec<usize>,
    pub prev_active_lanes: Vec<usize>,
    pub has_overflow: bool,
    pub has_missing_ancestry: bool,
}

pub(crate) fn dag_column(
    entry: &GraphEntry,
    lanes: DagRowLanes,
    layout: &DagLayout,
    theme: &Theme,
) -> AnyElement {
    let DagRowLanes {
        row,
        row_lane,
        active_lane,
        pass_through_lanes,
        prev_active_lanes,
        has_overflow,
        has_missing_ancestry,
    } = lanes;
    let display_lane_count = layout.display_lane_count();
    let total_w = lane_column_width(display_lane_count);
    let overflow_display_lane = display_lane_count.max(1) - 1;
    let style = DagNodeStyle::resolve(&entry.change, theme);
    let row_display_lane = layout.display_lane(row_lane);
    let faded_alpha = if theme.is_dark { 0.4 } else { 0.3 };
    let ink = move |color: u32, is_active: bool| match (active_lane, is_active) {
        (None, _) => LineInk::from(color),
        (Some(_), true) => LineInk {
            color,
            alpha: 1.0,
            width: ACTIVE_LINE_WIDTH,
        },
        (Some(_), false) => LineInk {
            color,
            alpha: faded_alpha,
            ..color.into()
        },
    };
    let span = active_lane.map(|span| (span, layout.display_lane(span.lane)));
    let in_lane = |lane: usize| span.is_some_and(|(span, _)| span.lane == lane);
    let line_color = theme.dag_line;
    let edge_color = theme.dag_edge;
    let node_top_offset = node_center_offset(theme);

    // Resolve targets up front — `layout` can't move into the canvas closure.
    let continues_lane = span.is_some_and(|(span, _)| span.first_row <= row && row < span.last_row);
    let edge_targets: Vec<(usize, EdgeType, LineInk)> = entry
        .edges
        .iter()
        .filter(|e| !matches!(e.edge_type, EdgeType::Missing))
        .map(|e| {
            let target_lane = layout.lane(&e.target);
            let is_active = continues_lane && in_lane(row_lane) && in_lane(target_lane);
            (
                layout.display_lane(target_lane),
                e.edge_type,
                ink(edge_color, is_active),
            )
        })
        .collect();

    let mut pass_through_display_lanes: Vec<usize> = pass_through_lanes
        .into_iter()
        .map(|lane| layout.display_lane(lane))
        .filter(|&lane| lane != row_display_lane)
        .collect();
    pass_through_display_lanes.sort();
    pass_through_display_lanes.dedup();

    let has_above = prev_active_lanes.contains(&row_lane);
    let top_stub_ink = ink(
        line_color,
        in_lane(row_lane)
            && span.is_some_and(|(span, _)| span.first_row < row && row <= span.last_row),
    );
    let passes_active = span.is_some_and(|(span, _)| span.first_row < row && row < span.last_row);
    let pass_through_inks: Vec<(usize, LineInk)> = pass_through_display_lanes
        .iter()
        .map(|&lane| {
            let is_active = passes_active && span.is_some_and(|(_, display)| display == lane);
            (lane, ink(line_color, is_active))
        })
        .collect();
    let side_ink = ink(edge_color, false);

    let overflow_pattern = LinePattern::Dashed(OVERFLOW_DASH_PATTERN);
    let line_pattern = move |display_lane| {
        if has_overflow && display_lane == overflow_display_lane {
            overflow_pattern
        } else {
            LinePattern::Solid
        }
    };

    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let h = bounds.size.height;
            let oy = bounds.origin.y;
            let ox = bounds.origin.x;
            let display_lane_center_x = |display_lane: usize| -> Pixels {
                ox + px(LEADING_PAD + display_lane as f32 * LANE_WIDTH + LANE_WIDTH / 2.0)
            };

            let my_x = display_lane_center_x(row_display_lane);
            let node_y = oy + px(node_top_offset);
            let radius_px = px(style.radius);
            let row_bottom = oy + h;

            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                for &(display_lane, lane_ink) in &pass_through_inks {
                    stroke_line_pattern(
                        window,
                        display_lane_center_x(display_lane),
                        oy,
                        display_lane_center_x(display_lane),
                        row_bottom,
                        lane_ink,
                        line_pattern(display_lane),
                    );
                }

                // 2. Current lane top stub — only when something is above to connect to.
                if has_above {
                    stroke_line_pattern(
                        window,
                        my_x,
                        oy,
                        my_x,
                        node_y - radius_px,
                        top_stub_ink,
                        line_pattern(row_display_lane),
                    );
                }

                let start_y = node_y + radius_px;
                for &(target_display_lane, edge_type, edge_ink) in &edge_targets {
                    let target_x = display_lane_center_x(target_display_lane);
                    let edge_pattern = if edge_type == EdgeType::Indirect {
                        LinePattern::Dashed(INDIRECT_EDGE_DASH_PATTERN)
                    } else if has_overflow
                        && (row_display_lane == overflow_display_lane
                            || target_display_lane == overflow_display_lane)
                    {
                        overflow_pattern
                    } else {
                        LinePattern::Solid
                    };
                    if target_display_lane == row_display_lane {
                        stroke_line_pattern(
                            window,
                            my_x,
                            start_y,
                            my_x,
                            row_bottom,
                            edge_ink,
                            edge_pattern,
                        );
                    } else {
                        stroke_curve_pattern(
                            window,
                            my_x,
                            start_y,
                            target_x,
                            row_bottom,
                            edge_ink,
                            edge_pattern,
                        );
                    }
                }

                if has_missing_ancestry {
                    let has_visible_parent = !edge_targets.is_empty();
                    let terminal_x = my_x
                        + px(if has_visible_parent {
                            LANE_WIDTH * 0.35
                        } else {
                            0.0
                        });
                    // End the side cap before the parent curves fan out at 40% of the remaining height.
                    let terminal_y = start_y
                        + (row_bottom - start_y) * if has_visible_parent { 0.25 } else { 0.55 };
                    stroke_line_pattern(
                        window,
                        my_x,
                        start_y,
                        terminal_x,
                        terminal_y,
                        side_ink,
                        LinePattern::Dashed(MISSING_EDGE_DASH_PATTERN),
                    );
                    stroke_line_pattern(
                        window,
                        terminal_x - px(2.0),
                        terminal_y,
                        terminal_x + px(2.0),
                        terminal_y,
                        side_ink,
                        LinePattern::Solid,
                    );
                }

                paint_node(window, my_x, node_y, style);
            });
        },
    )
    .flex_none()
    .w(px(total_w))
    .overflow_hidden()
    .h_full()
    .into_any_element()
}
