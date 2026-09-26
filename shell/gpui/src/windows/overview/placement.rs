use gpui::{Bounds, Pixels, point, px, size};
use jayjay_core::OverviewLane;
use jayjay_core::overview::OverviewGroup;

#[derive(Clone, Copy)]
pub(super) struct Geometry {
    pub spine_x: f32,
    pub trunk_label_width: f32,
    pub gutter_width: f32,
    pub column_width: f32,
    pub column_gap: f32,
    pub card_height: f32,
    pub card_gap: f32,
    pub row_height: f32,
    pub node_inset: f32,
    pub top_padding: f32,
    pub band_spacing: f32,
}

impl Geometry {
    pub(super) fn scaled(scale: f32) -> Self {
        Self {
            spine_x: 20.,
            trunk_label_width: 170. * scale,
            gutter_width: 200. * scale,
            column_width: 260. * scale,
            column_gap: 14.,
            card_height: 86. * scale,
            card_gap: 10.,
            row_height: 24. * scale,
            node_inset: 12.,
            top_padding: 14.,
            band_spacing: 40.,
        }
    }
}

pub(super) struct Band {
    pub group: usize,
    pub y: f32,
    pub last_x: f32,
}

pub(super) struct PlacedLane {
    pub lane: usize,
    pub x: f32,
    pub band_y: f32,
    pub count: usize,
}

impl PlacedLane {
    pub(super) fn node_y(&self, g: &Geometry, offset: usize) -> f32 {
        self.band_y - (self.count - offset) as f32 * g.row_height
    }

    pub(super) fn card_top(&self, g: &Geometry) -> f32 {
        self.node_y(g, 0) - g.row_height / 2. - g.card_gap - g.card_height
    }
}

pub(super) struct Placement {
    pub geometry: Geometry,
    pub bands: Vec<Band>,
    pub lanes: Vec<PlacedLane>,
    pub width: f32,
    pub height: f32,
}

impl Placement {
    pub(super) fn new(
        lanes: &[OverviewLane],
        groups: &[OverviewGroup],
        geometry: Geometry,
    ) -> Self {
        let g = geometry;
        let mut bands = Vec::new();
        let mut placed = Vec::new();
        let mut widest = 0;
        let mut previous_band_y: Option<f32> = None;
        for (group_ix, group) in groups.iter().enumerate() {
            let counts: Vec<usize> = group
                .lanes
                .iter()
                .map(|&lane| lanes[lane as usize].changes.len())
                .collect();
            let tallest = counts.iter().copied().max().unwrap_or(0);
            let top = previous_band_y.map_or(g.top_padding, |y| y + g.band_spacing);
            let band_y = top + g.card_height + g.card_gap + g.row_height * (tallest as f32 + 0.5);
            previous_band_y = Some(band_y);
            let mut last_x = g.spine_x;
            for (column, (&lane, &count)) in group.lanes.iter().zip(&counts).enumerate() {
                let x = g.gutter_width + column as f32 * (g.column_width + g.column_gap);
                placed.push(PlacedLane {
                    lane: lane as usize,
                    x,
                    band_y,
                    count,
                });
                last_x = x + g.node_inset;
            }
            widest = widest.max(group.lanes.len());
            bands.push(Band {
                group: group_ix,
                y: band_y,
                last_x,
            });
        }
        Self {
            width: g.gutter_width + widest as f32 * (g.column_width + g.column_gap) + 40.,
            height: bands.last().map_or(0., |band| band.y) + 40.,
            geometry,
            bands,
            lanes: placed,
        }
    }

    pub(super) fn lane(&self, lane: usize) -> Option<&PlacedLane> {
        self.lanes.iter().find(|placed| placed.lane == lane)
    }

    pub(super) fn selection_bounds(
        &self,
        placed: &PlacedLane,
        offset: Option<usize>,
    ) -> Bounds<Pixels> {
        let g = &self.geometry;
        let (top, height) = match offset {
            Some(offset) => (placed.node_y(g, offset) - g.row_height / 2., g.row_height),
            None => (placed.card_top(g), g.card_height),
        };
        Bounds::new(
            point(px(placed.x - g.column_gap), px(top - g.card_gap)),
            size(
                px(g.column_width + 2. * g.column_gap),
                px(height + 2. * g.card_gap),
            ),
        )
    }
}
