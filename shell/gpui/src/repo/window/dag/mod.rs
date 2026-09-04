//! DAG lane and node renderer for the repo sidebar.

mod paint;
mod style;

use gpui::{AnyElement, ContentMask, IntoElement, Pixels, Styled, canvas, px};
use jayjay_core::dag::DagLayout;
use jayjay_core::{EdgeType, GraphEntry};

use crate::app::theme::Theme;

use paint::{LinePattern, paint_node, stroke_curve_pattern, stroke_line_pattern};
use style::DagNodeStyle;

const LANE_WIDTH: f32 = 18.0;
const LEADING_PAD: f32 = 8.0;
const TRAILING_PAD: f32 = 6.0;
/// Aligns with the first text line in the DAG row.
const NODE_TOP_OFFSET: f32 = 15.0;
const OVERFLOW_DASH_PATTERN: &[f32] = &[10.0, 4.0, 10.0, 12.0];
const INDIRECT_EDGE_DASH_PATTERN: &[f32] = &[3.0, 3.0];

fn lane_column_width(display_lane_count: usize) -> f32 {
    let lanes = display_lane_count.max(1);
    lanes as f32 * LANE_WIDTH + LEADING_PAD + TRAILING_PAD
}

/// Lane geometry for a single DAG row — used to decide what lines to draw.
pub(super) struct DagRowLanes {
    pub row_lane: usize,
    pub pass_through_lanes: Vec<usize>,
    pub prev_active_lanes: Vec<usize>,
    pub next_active_lanes: Vec<usize>,
    pub has_overflow: bool,
}

pub(super) fn dag_column(
    entry: &GraphEntry,
    lanes: DagRowLanes,
    layout: &DagLayout,
    theme: &Theme,
) -> AnyElement {
    let DagRowLanes {
        row_lane,
        pass_through_lanes,
        prev_active_lanes,
        next_active_lanes,
        has_overflow,
    } = lanes;
    let total_w = lane_column_width(layout.display_lane_count());
    let overflow_display_lane = layout.display_lane_count().max(1) - 1;
    let style = DagNodeStyle::resolve(&entry.change, theme);
    let line_color = theme.dag_line;
    let edge_color = theme.dag_edge;
    let row_display_lane = layout.display_lane(row_lane);
    let node_top_offset = NODE_TOP_OFFSET + (theme.scaled_font_size(10.) - 10.) / 2.;

    // Resolve targets up front — `layout` can't move into the canvas closure.
    let edge_targets: Vec<(usize, usize, EdgeType)> = entry
        .edges
        .iter()
        .filter(|e| !matches!(e.edge_type, EdgeType::Missing))
        .map(|e| {
            let target_lane = layout.lane(&e.target);
            (target_lane, layout.display_lane(target_lane), e.edge_type)
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
    let has_same_lane_parent = edge_targets
        .iter()
        .any(|&(target_lane, _, _)| target_lane == row_lane);
    let has_below = has_same_lane_parent || next_active_lanes.contains(&row_lane);

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
                // 1. Lanes that pass through this row: full top → bottom.
                for &display_lane in &pass_through_display_lanes {
                    stroke_line_pattern(
                        window,
                        display_lane_center_x(display_lane),
                        oy,
                        display_lane_center_x(display_lane),
                        row_bottom,
                        line_color,
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
                        line_color,
                        line_pattern(row_display_lane),
                    );
                }

                // 3. Edges to parents — straight for same lane, quadratic curve otherwise.
                let start_y = node_y + radius_px;
                for &(_, target_display_lane, edge_type) in &edge_targets {
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
                            edge_color,
                            edge_pattern,
                        );
                    } else {
                        stroke_curve_pattern(
                            window,
                            my_x,
                            start_y,
                            target_x,
                            row_bottom,
                            edge_color,
                            edge_pattern,
                        );
                    }
                }

                // 3b. Bottom stub for non-tail nodes on a forking lane.
                if !has_same_lane_parent && has_below {
                    stroke_line_pattern(
                        window,
                        my_x,
                        start_y,
                        my_x,
                        row_bottom,
                        line_color,
                        line_pattern(row_display_lane),
                    );
                }

                // 4. Node on top.
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
