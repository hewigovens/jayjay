use std::ops::Range;

use gpui::{
    App, Bounds, Element, ElementId, ElementInputHandler, Entity, GlobalElementId, IntoElement,
    LayoutId, PaintQuad, Pixels, SharedString, Style, TextRun, Window, fill, hsla, point, px,
    relative, rgba, size,
};

use super::{LineLayout, TextArea, TextLayout};

pub(super) struct TextAreaElement {
    pub(super) input: Entity<TextArea>,
    pub(super) height: f32,
}

pub(super) struct PrepaintState {
    lines: Vec<LineLayout>,
    line_height: Pixels,
    cursor: Option<PaintQuad>,
    selections: Vec<PaintQuad>,
}

impl IntoElement for TextAreaElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextAreaElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = px(self.height).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let ranges = if content.is_empty() {
            std::iter::once(0..0).collect()
        } else {
            input.line_ranges()
        };
        let mut lines = Vec::new();
        for (ix, range) in ranges.into_iter().enumerate() {
            let display_text: SharedString = if content.is_empty() {
                input.placeholder.clone()
            } else {
                SharedString::from(content[range.clone()].to_string())
            };
            let color = if content.is_empty() {
                hsla(0., 0., 0.55, 0.62)
            } else {
                style.color
            };
            let run = TextRun {
                len: display_text.len(),
                font: style.font(),
                color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let shaped = window
                .text_system()
                .shape_line(display_text, font_size, &[run], None);
            lines.push(LineLayout {
                range,
                shaped,
                top: px(ix as f32 * f32::from(line_height)),
            });
        }

        let selections = selection_quads(&lines, &input.selected_range, bounds, line_height);
        let cursor = if input.selected_range.is_empty() && input.caret_visible() {
            cursor_quad(&lines, input.cursor_offset(), bounds, line_height)
        } else {
            None
        };

        PrepaintState {
            lines,
            line_height,
            cursor,
            selections,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        for selection in prepaint.selections.drain(..) {
            window.paint_quad(selection);
        }
        for line in &prepaint.lines {
            line.shaped
                .paint(
                    point(bounds.left(), bounds.top() + line.top),
                    prepaint.line_height,
                    gpui::TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .unwrap();
        }
        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        let lines = std::mem::take(&mut prepaint.lines);
        let line_height = prepaint.line_height;
        self.input.update(cx, |input, _| {
            input.last_layout = Some(TextLayout { lines, line_height });
            input.last_bounds = Some(bounds);
        });
    }
}

fn selection_quads(
    lines: &[LineLayout],
    selected: &Range<usize>,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
) -> Vec<PaintQuad> {
    if selected.is_empty() {
        return Vec::new();
    }
    let mut quads = Vec::new();
    for line in lines {
        let start = selected.start.max(line.range.start);
        let end = selected.end.min(line.range.end);
        if start > end || (start == end && !line.range.is_empty()) {
            continue;
        }
        let local_start = start.saturating_sub(line.range.start).min(line.range.len());
        let local_end = end.saturating_sub(line.range.start).min(line.range.len());
        quads.push(fill(
            Bounds::from_corners(
                point(
                    bounds.left() + line.shaped.x_for_index(local_start),
                    bounds.top() + line.top,
                ),
                point(
                    bounds.left() + line.shaped.x_for_index(local_end),
                    bounds.top() + line.top + line_height,
                ),
            ),
            rgba(0x333b82f6),
        ));
    }
    quads
}

fn cursor_quad(
    lines: &[LineLayout],
    cursor: usize,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
) -> Option<PaintQuad> {
    let line = lines
        .iter()
        .find(|line| line.range.start <= cursor && cursor <= line.range.end)
        .or_else(|| lines.last())?;
    let local = cursor
        .saturating_sub(line.range.start)
        .min(line.range.len());
    Some(fill(
        Bounds::new(
            point(
                bounds.left() + line.shaped.x_for_index(local),
                bounds.top() + line.top,
            ),
            size(px(1.5), line_height),
        ),
        gpui::blue(),
    ))
}
