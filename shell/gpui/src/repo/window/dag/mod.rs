//! DAG row renderer for the repo sidebar: draws the row shapes the Rust renderer computes.

mod paint;
mod style;

use gpui::{AnyElement, ContentMask, IntoElement, Pixels, Styled, canvas, point, px};
use jayjay_core::GraphEntry;
use jayjay_core::dag::{DagEdgeKind, DagLayout, DagLinkCell, DagVerticalCell};

use crate::app::theme::Theme;

use paint::{LinePattern, paint_node, stroke_line_pattern, stroke_rounded_elbow_pattern};
use style::DagNodeStyle;

const PREFERRED_LANE_PITCH: f32 = 13.5;
const ABSOLUTE_GRAPH_MAX_WIDTH: f32 = 192.0;
const MAX_SIDEBAR_FRACTION: f32 = 0.45;
const LEADING_PAD: f32 = 8.0;
const TRAILING_PAD: f32 = 6.0;
const HORIZONTAL_PADDING: f32 = LEADING_PAD + TRAILING_PAD;
const PREFERRED_NODE_RADIUS: f32 = 4.5;
const MINIMUM_NODE_RADIUS: f32 = 1.5;
/// Aligns with the first text line in the DAG row.
const NODE_TOP_OFFSET: f32 = 15.0;
const LINK_CENTER_FRACTION: f32 = 0.45;
const INDIRECT_EDGE_DASH_PATTERN: &[f32] = &[3.0, 3.0];
const MISSING_EDGE_DASH_PATTERN: &[f32] = &[2.0, 2.0];

/// Maps logical columns to pixel positions for the sidebar's current width. One value is built per render and shared by every visible row, so column pitch never drifts row to row.
#[derive(Clone, Copy)]
pub(super) struct DagGeometry {
    pub lane_pitch: f32,
    pub node_radius: f32,
    pub graph_width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkComponent {
    Vertical(DagEdgeKind),
    Horizontal(DagEdgeKind),
    LeftFork(DagEdgeKind),
    RightFork(DagEdgeKind),
    LeftMerge(DagEdgeKind),
    RightMerge(DagEdgeKind),
}

#[derive(Clone, Copy)]
struct LinkBand {
    top: Pixels,
    center: Pixels,
    bottom: Pixels,
    half_pitch: Pixels,
}

impl LinkComponent {
    fn edge_kind(self) -> DagEdgeKind {
        match self {
            Self::Vertical(kind)
            | Self::Horizontal(kind)
            | Self::LeftFork(kind)
            | Self::RightFork(kind)
            | Self::LeftMerge(kind)
            | Self::RightMerge(kind) => kind,
        }
    }

    fn rounded_elbow(self, x: Pixels, band: LinkBand) -> Option<[gpui::Point<Pixels>; 3]> {
        match self {
            Self::LeftFork(_) => Some([
                point(x - band.half_pitch, band.center),
                point(x, band.center),
                point(x, band.bottom),
            ]),
            Self::RightFork(_) => Some([
                point(x + band.half_pitch, band.center),
                point(x, band.center),
                point(x, band.bottom),
            ]),
            Self::LeftMerge(_) => Some([
                point(x, band.top),
                point(x, band.center),
                point(x - band.half_pitch, band.center),
            ]),
            Self::RightMerge(_) => Some([
                point(x, band.top),
                point(x, band.center),
                point(x + band.half_pitch, band.center),
            ]),
            Self::Vertical(_) | Self::Horizontal(_) => None,
        }
    }
}

impl DagGeometry {
    pub(super) fn new(logical_column_count: u32, available_sidebar_width: f32) -> Self {
        let columns = logical_column_count.max(1) as f32;
        let width_budget =
            ABSOLUTE_GRAPH_MAX_WIDTH.min(available_sidebar_width * MAX_SIDEBAR_FRACTION);
        let preferred_width = HORIZONTAL_PADDING + columns * PREFERRED_LANE_PITCH;
        let width_floor = HORIZONTAL_PADDING + PREFERRED_LANE_PITCH;
        let graph_width = preferred_width.min(width_budget).max(width_floor);
        let lane_pitch = (graph_width - HORIZONTAL_PADDING) / columns;
        // Full radius at the preferred pitch; shrink proportionally only once the sidebar compresses lanes below it.
        let node_radius = PREFERRED_NODE_RADIUS.min(
            MINIMUM_NODE_RADIUS.max(PREFERRED_NODE_RADIUS * lane_pitch / PREFERRED_LANE_PITCH),
        );
        Self {
            lane_pitch,
            node_radius,
            graph_width,
        }
    }
}

pub(super) fn dag_column(
    entry: &GraphEntry,
    layout: &DagLayout,
    geometry: &DagGeometry,
    theme: &Theme,
) -> AnyElement {
    let style = DagNodeStyle::resolve(&entry.change, theme, geometry.node_radius);
    let line_color = theme.dag_line;
    let edge_color = theme.dag_edge;

    let Some(row) = layout.row(&entry.change.commit_id) else {
        return canvas(|_, _, _| (), |_, _, _, _| {})
            .flex_none()
            .w(px(geometry.graph_width))
            .h_full()
            .into_any_element();
    };
    let node_column = row.node_column;
    let incoming = row.incoming;
    let node_line = row.node_line.clone();
    let link_line = row.link_line.clone();
    let pad_line = row.pad_line.clone();
    let termination_columns = row.termination_columns.clone();

    let graph_width = geometry.graph_width;
    let lane_pitch = geometry.lane_pitch;
    let node_radius = geometry.node_radius;
    let x_position =
        move |column: u32| -> f32 { LEADING_PAD + column as f32 * lane_pitch + lane_pitch / 2.0 };

    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let h = bounds.size.height;
            let oy = bounds.origin.y;
            let ox = bounds.origin.x;
            let column_center_x = |column: u32| -> Pixels { ox + px(x_position(column)) };

            let my_x = column_center_x(node_column);
            let node_y = oy + px(NODE_TOP_OFFSET);
            let radius_px = px(node_radius);
            let row_bottom = oy + h;
            let start_y = node_y + radius_px;
            let link_center_y = if link_line.is_some() {
                node_y + (row_bottom - node_y) * LINK_CENTER_FRACTION
            } else {
                node_y
            };
            let link_bottom_y = if link_line.is_some() {
                row_bottom.min(link_center_y + px(paint::CORNER_RADIUS))
            } else {
                node_y
            };

            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                // The node line is the renderer state above this row's transition band.
                for (column, cell) in node_line.iter().enumerate() {
                    let column = column as u32;
                    if column == node_column {
                        continue;
                    }
                    let Some(pattern) = line_pattern_for(cell) else {
                        continue;
                    };
                    stroke_line_pattern(
                        window,
                        column_center_x(column),
                        oy,
                        column_center_x(column),
                        node_y,
                        line_color,
                        pattern,
                    );
                }

                if let Some(kind) = incoming {
                    stroke_line_pattern(
                        window,
                        my_x,
                        oy,
                        my_x,
                        node_y - radius_px,
                        line_color,
                        line_pattern_for_kind(kind),
                    );
                }

                if let Some(link_line) = &link_line {
                    for (column, cell) in link_line.iter().enumerate() {
                        let column = column as u32;
                        let x = column_center_x(column);
                        let top = if cell.is_child && column == node_column {
                            start_y
                        } else {
                            node_y
                        };
                        for component in link_components(cell) {
                            paint_link_component(
                                window,
                                component,
                                x,
                                LinkBand {
                                    top,
                                    center: link_center_y,
                                    bottom: link_bottom_y,
                                    half_pitch: px(lane_pitch / 2.0),
                                },
                                edge_color,
                            );
                        }
                    }
                }

                // The pad line is the renderer state below the transition band.
                for (column, cell) in pad_line.iter().enumerate() {
                    let Some(pattern) = line_pattern_for(cell) else {
                        continue;
                    };
                    let column = column as u32;
                    let pad_start = if link_line.is_some() {
                        link_bottom_y
                    } else if column == node_column {
                        start_y
                    } else {
                        node_y
                    };
                    stroke_line_pattern(
                        window,
                        column_center_x(column),
                        pad_start,
                        column_center_x(column),
                        row_bottom,
                        line_color,
                        pattern,
                    );
                }

                // Missing-parent terminators.
                for &column in &termination_columns {
                    let x = column_center_x(column);
                    let start = if link_line.is_some() {
                        link_bottom_y
                    } else if column == node_column {
                        start_y
                    } else {
                        node_y
                    };
                    let end = start + (row_bottom - start) * 0.55;
                    stroke_line_pattern(
                        window,
                        x,
                        start,
                        x,
                        end,
                        edge_color,
                        LinePattern::Dashed(MISSING_EDGE_DASH_PATTERN),
                    );
                }

                // Node on top.
                paint_node(window, my_x, node_y, style);
            });
        },
    )
    .flex_none()
    .w(px(graph_width))
    .overflow_hidden()
    .h_full()
    .into_any_element()
}

fn line_pattern_for(cell: &DagVerticalCell) -> Option<LinePattern> {
    match cell {
        DagVerticalCell::Empty => None,
        DagVerticalCell::Direct => Some(LinePattern::Solid),
        DagVerticalCell::Indirect => Some(LinePattern::Dashed(INDIRECT_EDGE_DASH_PATTERN)),
    }
}

fn line_pattern_for_kind(kind: DagEdgeKind) -> LinePattern {
    match kind {
        DagEdgeKind::Direct => LinePattern::Solid,
        DagEdgeKind::Indirect => LinePattern::Dashed(INDIRECT_EDGE_DASH_PATTERN),
    }
}

fn link_components(cell: &DagLinkCell) -> impl Iterator<Item = LinkComponent> + '_ {
    [
        cell.vertical.map(LinkComponent::Vertical),
        cell.horizontal.map(LinkComponent::Horizontal),
        cell.left_fork.map(LinkComponent::LeftFork),
        cell.right_fork.map(LinkComponent::RightFork),
        cell.left_merge.map(LinkComponent::LeftMerge),
        cell.right_merge.map(LinkComponent::RightMerge),
    ]
    .into_iter()
    .flatten()
}

fn paint_link_component(
    window: &mut gpui::Window,
    component: LinkComponent,
    x: Pixels,
    band: LinkBand,
    color: u32,
) {
    let pattern = line_pattern_for_kind(component.edge_kind());
    let radius = px(paint::CORNER_RADIUS)
        .min(band.half_pitch)
        .min(band.center - band.top)
        .min(band.bottom - band.center);
    if let Some([start, corner, end]) = component.rounded_elbow(x, band) {
        stroke_rounded_elbow_pattern(window, start, corner, end, radius, color, pattern);
        return;
    }

    match component {
        LinkComponent::Vertical(_) => {
            stroke_line_pattern(window, x, band.top, x, band.bottom, color, pattern)
        }
        LinkComponent::Horizontal(_) => stroke_line_pattern(
            window,
            x - band.half_pitch,
            band.center,
            x + band.half_pitch,
            band.center,
            color,
            pattern,
        ),
        LinkComponent::LeftFork(_)
        | LinkComponent::RightFork(_)
        | LinkComponent::LeftMerge(_)
        | LinkComponent::RightMerge(_) => unreachable!("elbows returned above"),
    }
}

#[cfg(test)]
mod tests {
    use jayjay_core::dag::{DagEdgeKind, DagLinkCell};

    use super::{LinkBand, LinkComponent, link_components};

    #[test]
    fn link_components_preserve_every_typed_renderer_segment() {
        let cell = DagLinkCell {
            vertical: Some(DagEdgeKind::Direct),
            horizontal: Some(DagEdgeKind::Indirect),
            left_fork: Some(DagEdgeKind::Direct),
            right_fork: Some(DagEdgeKind::Indirect),
            left_merge: Some(DagEdgeKind::Direct),
            right_merge: Some(DagEdgeKind::Indirect),
            is_child: true,
        };

        assert_eq!(
            link_components(&cell).collect::<Vec<_>>(),
            vec![
                LinkComponent::Vertical(DagEdgeKind::Direct),
                LinkComponent::Horizontal(DagEdgeKind::Indirect),
                LinkComponent::LeftFork(DagEdgeKind::Direct),
                LinkComponent::RightFork(DagEdgeKind::Indirect),
                LinkComponent::LeftMerge(DagEdgeKind::Direct),
                LinkComponent::RightMerge(DagEdgeKind::Indirect),
            ]
        );
    }

    #[test]
    fn forks_and_merges_retain_rounded_elbows() {
        let bends = [
            LinkComponent::LeftFork(DagEdgeKind::Direct),
            LinkComponent::RightFork(DagEdgeKind::Direct),
            LinkComponent::LeftMerge(DagEdgeKind::Direct),
            LinkComponent::RightMerge(DagEdgeKind::Direct),
        ];
        let straights = [
            LinkComponent::Vertical(DagEdgeKind::Direct),
            LinkComponent::Horizontal(DagEdgeKind::Direct),
        ];

        let band = LinkBand {
            top: gpui::px(0.0),
            center: gpui::px(10.0),
            bottom: gpui::px(20.0),
            half_pitch: gpui::px(10.0),
        };

        assert!(
            bends
                .into_iter()
                .all(|component| component.rounded_elbow(gpui::px(10.0), band).is_some())
        );
        assert!(
            straights
                .into_iter()
                .all(|component| component.rounded_elbow(gpui::px(10.0), band).is_none())
        );
    }
}
