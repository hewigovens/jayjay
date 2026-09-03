//! DAG row renderer for the repo sidebar: draws the row shapes the Rust renderer computes.

mod paint;
mod style;

use gpui::{AnyElement, ContentMask, IntoElement, Pixels, Styled, canvas, point, px};
use jayjay_core::GraphEntry;
use jayjay_core::dag::{
    DagContinuationDirection, DagEdgeKind, DagLayout, DagLinkCell, DagVerticalCell,
};

use crate::app::theme::Theme;

use paint::{LinePattern, paint_node, stroke_line_pattern, stroke_rounded_elbow_pattern};
use style::DagNodeStyle;

const PREFERRED_LANE_PITCH: f32 = 13.5;
const MINIMUM_LEGIBLE_LANE_PITCH: f32 = 10.0;
const ABSOLUTE_GRAPH_MAX_WIDTH: f32 = 192.0;
const MAX_SIDEBAR_FRACTION: f32 = 0.45;
const LEADING_PAD: f32 = 8.0;
const TRAILING_PAD: f32 = 6.0;
const HORIZONTAL_PADDING: f32 = LEADING_PAD + TRAILING_PAD;
const PREFERRED_NODE_RADIUS: f32 = 4.5;
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

#[derive(Clone, Copy)]
struct ContinuationMarkerGeometry {
    shaft_start: gpui::Point<Pixels>,
    tip: gpui::Point<Pixels>,
    arrowhead_left: gpui::Point<Pixels>,
    arrowhead_right: gpui::Point<Pixels>,
}

impl ContinuationMarkerGeometry {
    fn new(
        direction: DagContinuationDirection,
        x: Pixels,
        row_height: Pixels,
        node_y: Pixels,
        node_radius: Pixels,
    ) -> Self {
        const BOUNDARY_INSET: f32 = 2.0;
        const ARROWHEAD_HALF_WIDTH: f32 = 2.5;
        const ARROWHEAD_DEPTH: f32 = 4.0;

        let points_toward_top = direction == DagContinuationDirection::Incoming;
        let tip_y = if points_toward_top {
            px(BOUNDARY_INSET)
        } else {
            row_height - px(BOUNDARY_INSET)
        };
        let arrowhead_base_y = tip_y
            + if points_toward_top {
                px(ARROWHEAD_DEPTH)
            } else {
                -px(ARROWHEAD_DEPTH)
            };
        Self {
            shaft_start: point(
                x,
                if points_toward_top {
                    node_y - node_radius
                } else {
                    node_y + node_radius
                },
            ),
            tip: point(x, tip_y),
            arrowhead_left: point(x - px(ARROWHEAD_HALF_WIDTH), arrowhead_base_y),
            arrowhead_right: point(x + px(ARROWHEAD_HALF_WIDTH), arrowhead_base_y),
        }
    }
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
        let compressed_pitch = (width_budget - HORIZONTAL_PADDING) / columns;
        let lane_pitch = compressed_pitch.clamp(MINIMUM_LEGIBLE_LANE_PITCH, PREFERRED_LANE_PITCH);
        let graph_width = HORIZONTAL_PADDING + columns * lane_pitch;
        let node_radius = PREFERRED_NODE_RADIUS;
        Self {
            lane_pitch,
            node_radius,
            graph_width,
        }
    }
}

fn link_top(column: u32, node_column: u32, node_y: Pixels, node_radius: Pixels) -> Pixels {
    if column == node_column {
        node_y + node_radius
    } else {
        node_y
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
    let continuations = row.continuations.clone();

    let graph_width = geometry.graph_width;
    let lane_pitch = geometry.lane_pitch;
    let node_radius = style.radius;
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
                        let top = link_top(column, node_column, node_y, radius_px);
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

                const MARKER_SPACING: f32 = 4.0;
                for (index, continuation) in continuations.iter().enumerate() {
                    let offset =
                        (index as f32 - (continuations.len() - 1) as f32 / 2.0) * MARKER_SPACING;
                    let marker = ContinuationMarkerGeometry::new(
                        continuation.direction,
                        my_x + px(offset),
                        h,
                        node_y - oy,
                        radius_px,
                    );
                    let shaft_start = point(marker.shaft_start.x, marker.shaft_start.y + oy);
                    let tip = point(marker.tip.x, marker.tip.y + oy);
                    let color = continuation_color(&continuation.key);
                    stroke_line_pattern(
                        window,
                        shaft_start.x,
                        shaft_start.y,
                        tip.x,
                        tip.y,
                        color,
                        line_pattern_for_kind(continuation.edge_kind),
                    );
                    for arrowhead in [marker.arrowhead_left, marker.arrowhead_right] {
                        let arrowhead = point(arrowhead.x, arrowhead.y + oy);
                        stroke_line_pattern(
                            window,
                            arrowhead.x,
                            arrowhead.y,
                            tip.x,
                            tip.y,
                            color,
                            LinePattern::Solid,
                        );
                    }
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

fn continuation_color(key: &str) -> u32 {
    const COLORS: [u32; 6] = [0x3B82F6, 0xF59E0B, 0x8B5CF6, 0x22C55E, 0xEC4899, 0x06B6D4];
    const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;
    let hash = key.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    });
    COLORS[hash as usize % COLORS.len()]
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
    use jayjay_core::dag::{DagContinuationDirection, DagEdgeKind, DagLinkCell};

    use super::{
        ContinuationMarkerGeometry, DagGeometry, LinkBand, LinkComponent,
        MINIMUM_LEGIBLE_LANE_PITCH, PREFERRED_LANE_PITCH, link_components, link_top,
    };

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

    #[test]
    fn node_column_link_starts_outside_node() {
        let node_column = 1;
        let other_column = 2;
        let node_y = gpui::px(12.0);
        let node_radius = gpui::px(5.0);

        assert_eq!(
            link_top(node_column, node_column, node_y, node_radius),
            node_y + node_radius
        );
        assert_eq!(
            link_top(other_column, node_column, node_y, node_radius),
            node_y
        );
    }

    #[test]
    fn narrow_sidebar_never_compresses_lanes_or_nodes_below_legible_sizes() {
        let geometry = DagGeometry::new(10, 200.0);

        assert_eq!(geometry.lane_pitch, MINIMUM_LEGIBLE_LANE_PITCH);
        assert_eq!(geometry.node_radius, super::PREFERRED_NODE_RADIUS);
        assert_eq!(
            geometry.graph_width,
            super::HORIZONTAL_PADDING + 10.0 * MINIMUM_LEGIBLE_LANE_PITCH
        );
    }

    #[test]
    fn ordinary_projected_graph_uses_preferred_pitch() {
        let geometry = DagGeometry::new(8, 1_000.0);

        assert_eq!(geometry.lane_pitch, PREFERRED_LANE_PITCH);
    }

    #[test]
    fn continuation_arrows_point_toward_their_clipped_row_boundary() {
        let outgoing = ContinuationMarkerGeometry::new(
            DagContinuationDirection::Outgoing,
            gpui::px(20.0),
            gpui::px(44.0),
            gpui::px(15.0),
            gpui::px(4.5),
        );
        let incoming = ContinuationMarkerGeometry::new(
            DagContinuationDirection::Incoming,
            gpui::px(20.0),
            gpui::px(44.0),
            gpui::px(15.0),
            gpui::px(4.5),
        );

        assert!(outgoing.tip.y > outgoing.shaft_start.y);
        assert_eq!(outgoing.tip.y, gpui::px(42.0));
        assert!(incoming.tip.y < incoming.shaft_start.y);
        assert_eq!(incoming.tip.y, gpui::px(2.0));
    }
}
