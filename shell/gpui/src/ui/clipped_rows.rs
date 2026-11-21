use std::ops::Range;

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Style, Window, point, px, relative, size,
};

type RenderRows = Box<dyn Fn(Range<usize>, &mut Window, &mut App) -> Vec<AnyElement>>;

/// Equal-height rows laid out at their full height that build only the rows inside the enclosing clip, for row lists that scroll with an outer container.
pub(crate) struct ClippedRows {
    count: usize,
    row_height: Pixels,
    render: RenderRows,
}

pub(crate) fn clipped_rows(
    count: usize,
    row_height: Pixels,
    render: impl Fn(Range<usize>, &mut Window, &mut App) -> Vec<AnyElement> + 'static,
) -> ClippedRows {
    ClippedRows {
        count,
        row_height,
        render: Box::new(render),
    }
}

fn rows_in_clip(
    count: usize,
    row_height: f32,
    top: f32,
    clip_top: f32,
    clip_bottom: f32,
) -> Range<usize> {
    if row_height <= 0. || clip_bottom <= clip_top {
        return 0..0;
    }
    let first = ((clip_top - top) / row_height).floor().max(0.) as usize;
    let last = ((clip_bottom - top) / row_height).ceil().max(0.) as usize;
    first.min(count)..last.min(count)
}

impl IntoElement for ClippedRows {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ClippedRows {
    type RequestLayoutState = ();
    type PrepaintState = Vec<AnyElement>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (self.row_height * self.count).into();
        style.flex_shrink = 0.;
        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let clip = window.content_mask().bounds;
        let range = rows_in_clip(
            self.count,
            f32::from(self.row_height),
            f32::from(bounds.top()),
            f32::from(clip.top()),
            f32::from(clip.bottom()),
        );
        let space = size(
            AvailableSpace::Definite(bounds.size.width),
            AvailableSpace::Definite(self.row_height),
        );
        let mut rows = (self.render)(range.clone(), window, cx);
        for (row, ix) in rows.iter_mut().zip(range) {
            row.layout_as_root(space, window, cx);
            let top = px(f32::from(self.row_height) * ix as f32);
            row.prepaint_at(bounds.origin + point(px(0.), top), window, cx);
        }
        rows
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        rows: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        for row in rows {
            row.paint(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rows_in_clip;

    #[test]
    fn only_rows_inside_the_clip_are_built() {
        assert_eq!(rows_in_clip(100, 20., 0., 0., 100.), 0..5);
        assert_eq!(rows_in_clip(100, 20., -410., 0., 100.), 20..26);
        assert_eq!(rows_in_clip(3, 20., 500., 0., 100.), 0..0);
        assert_eq!(rows_in_clip(3, 20., -500., 0., 100.), 3..3);
    }
}
