use gpui::{
    App, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity, GlobalElementId,
    IntoElement, LayoutId, PaintQuad, Pixels, Style, Window, point, px, relative,
};

use super::super::{LineLayout, TextArea, TextLayout};
use super::layout::build_lines;
use super::paint::{cursor_quad, selection_quads};
use crate::app::theme::theme;

pub(in crate::ui::text_area) struct TextAreaElement {
    pub(in crate::ui::text_area) input: Entity<TextArea>,
    pub(in crate::ui::text_area) height: f32,
}

pub(in crate::ui::text_area) struct PrepaintState {
    lines: Vec<LineLayout>,
    line_height: Pixels,
    cursor: Option<PaintQuad>,
    selections: Vec<PaintQuad>,
    scroll_y: Pixels,
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
        let (lines, line_height) = build_lines(input, bounds, window);
        let selected_range = input.selection.range().clone();
        let cursor_offset = input.cursor_offset();
        let caret_visible = input.caret_visible();
        let pending_into_view = input.scroll_caret_into_view;
        let current_scroll = input.scroll_y;

        let content_height = px(f32::from(line_height) * lines.len() as f32);
        let max_scroll = (content_height - bounds.size.height).max(px(0.));
        let mut scroll_y = current_scroll.min(max_scroll).max(px(0.));
        if pending_into_view
            && let Some(line) = lines
                .iter()
                .find(|line| line.range.start <= cursor_offset && cursor_offset <= line.range.end)
                .or_else(|| lines.last())
        {
            let bottom = line.top + line_height;
            if line.top < scroll_y {
                scroll_y = line.top;
            } else if bottom > scroll_y + bounds.size.height {
                scroll_y = bottom - bounds.size.height;
            }
        }
        if scroll_y != current_scroll || pending_into_view {
            self.input.update(cx, |input, _| {
                input.scroll_y = scroll_y;
                input.scroll_caret_into_view = false;
            });
        }

        let scrolled = Bounds::new(point(bounds.left(), bounds.top() - scroll_y), bounds.size);
        let t = theme(cx);
        let selections = selection_quads(&lines, &selected_range, scrolled, line_height, t);
        let cursor = if selected_range.is_empty() && caret_visible {
            cursor_quad(&lines, cursor_offset, scrolled, line_height, t.fg)
        } else {
            None
        };

        PrepaintState {
            lines,
            line_height,
            cursor,
            selections,
            scroll_y,
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
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for selection in prepaint.selections.drain(..) {
                window.paint_quad(selection);
            }
            for line in &prepaint.lines {
                line.shaped
                    .paint(
                        point(bounds.left(), bounds.top() + line.top - prepaint.scroll_y),
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
        });

        let lines = std::mem::take(&mut prepaint.lines);
        let line_height = prepaint.line_height;
        self.input.update(cx, |input, _| {
            input.last_layout = Some(TextLayout { lines, line_height });
            input.last_bounds = Some(bounds);
        });
    }
}
