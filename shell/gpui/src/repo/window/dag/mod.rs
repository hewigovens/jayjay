//! DAG lane and node renderer for the repo sidebar.

mod paint;
mod style;

use gpui::{AnyElement, IntoElement, Pixels, Styled, canvas, px};
use jayjay_core::dag::DagLayout;
use jayjay_core::{EdgeType, GraphEntry};

use crate::app::theme::Theme;

use paint::{paint_node, stroke_curve, stroke_line};
use style::DagNodeStyle;

const LANE_WIDTH: f32 = 18.0;
const LEADING_PAD: f32 = 8.0;
const TRAILING_PAD: f32 = 6.0;
/// Aligns with the first text line in the DAG row.
const NODE_TOP_OFFSET: f32 = 15.0;

fn lane_column_width(max_lanes: usize) -> f32 {
    let lanes = max_lanes.max(1);
    lanes as f32 * LANE_WIDTH + LEADING_PAD + TRAILING_PAD
}

/// Lane geometry for a single DAG row — used to decide what lines to draw.
pub(super) struct DagRowLanes {
    pub row_lane: usize,
    pub active_lanes: Vec<usize>,
    pub prev_active_lanes: Vec<usize>,
    pub next_active_lanes: Vec<usize>,
    pub max_lanes: usize,
}

pub(super) fn dag_column(
    entry: &GraphEntry,
    lanes: DagRowLanes,
    layout: &DagLayout,
    theme: &Theme,
) -> AnyElement {
    let DagRowLanes {
        row_lane,
        active_lanes,
        prev_active_lanes,
        next_active_lanes,
        max_lanes,
    } = lanes;
    let total_w = lane_column_width(max_lanes);
    let style = DagNodeStyle::resolve(&entry.change, theme);
    let line_color = theme.dag_line;
    let edge_color = theme.dag_edge;

    // Resolve targets up front — `layout` can't move into the canvas closure.
    let edge_targets: Vec<usize> = entry
        .edges
        .iter()
        .filter(|e| !matches!(e.edge_type, EdgeType::Missing))
        .map(|e| layout.lane(&e.target))
        .collect();

    let mut active_other: Vec<usize> = active_lanes
        .into_iter()
        .filter(|&l| l != row_lane)
        .collect();
    active_other.sort();
    active_other.dedup();

    let has_above = prev_active_lanes.contains(&row_lane);
    let has_same_lane_parent = edge_targets.contains(&row_lane);
    let has_below = has_same_lane_parent || next_active_lanes.contains(&row_lane);

    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let h = bounds.size.height;
            let oy = bounds.origin.y;
            let ox = bounds.origin.x;
            let lane_center_x = |lane: usize| -> Pixels {
                ox + px(LEADING_PAD + lane as f32 * LANE_WIDTH + LANE_WIDTH / 2.0)
            };

            let my_x = lane_center_x(row_lane);
            let node_y = oy + px(NODE_TOP_OFFSET);
            let radius_px = px(style.radius);
            let row_bottom = oy + h;

            // 1. Non-current active lane continuations: full top → bottom.
            for &lane in &active_other {
                stroke_line(
                    window,
                    lane_center_x(lane),
                    oy,
                    lane_center_x(lane),
                    row_bottom,
                    line_color,
                );
            }

            // 2. Current lane top stub — only when something is above to connect to.
            if has_above {
                stroke_line(window, my_x, oy, my_x, node_y - radius_px, line_color);
            }

            // 3. Edges to parents — straight for same lane, quadratic curve otherwise.
            let start_y = node_y + radius_px;
            for &target_lane in &edge_targets {
                let target_x = lane_center_x(target_lane);
                if target_lane == row_lane {
                    stroke_line(window, my_x, start_y, my_x, row_bottom, edge_color);
                } else {
                    stroke_curve(window, my_x, start_y, target_x, row_bottom, edge_color);
                }
            }

            // 3b. Bottom stub for non-tail nodes on a forking lane.
            if !has_same_lane_parent && has_below {
                stroke_line(window, my_x, start_y, my_x, row_bottom, line_color);
            }

            // 4. Node on top.
            paint_node(window, my_x, node_y, style);
        },
    )
    .flex_none()
    .w(px(total_w))
    .h_full()
    .into_any_element()
}
