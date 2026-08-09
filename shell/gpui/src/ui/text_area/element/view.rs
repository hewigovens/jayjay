use std::{ops::Range, sync::Arc};

use gpui::{
    App, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity, GlobalElementId,
    IntoElement, LayoutId, PaintQuad, Pixels, Style, Window, point, px, relative,
};

use super::super::{LineLayout, TextArea, TextLayout, TextLayoutKey};
use super::layout::build_lines;
use super::paint::{cursor_quad, line_background_quads, selection_quads};
use crate::app::theme::theme;

pub(in crate::ui::text_area) struct TextAreaElement {
    pub(in crate::ui::text_area) input: Entity<TextArea>,
    pub(in crate::ui::text_area) height: Option<f32>,
}

pub(in crate::ui::text_area) struct PrepaintState {
    key: TextLayoutKey,
    lines: Arc<[LineLayout]>,
    visible_lines: Range<usize>,
    line_height: Pixels,
    cursor: Option<PaintQuad>,
    line_backgrounds: Vec<PaintQuad>,
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
        style.size.height = self
            .height
            .map_or_else(|| relative(1.).into(), |height| px(height).into());
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
        let key = layout_key(bounds, window, theme(cx));
        let (lines, line_height) = input
            .last_layout
            .as_ref()
            .filter(|layout| layout.key == key)
            .map_or_else(
                || {
                    let (lines, line_height) = build_lines(input, bounds, window, theme(cx));
                    (Arc::from(lines), line_height)
                },
                |layout| (layout.lines.clone(), layout.line_height),
            );
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

        let visible_lines =
            visible_line_range(lines.len(), scroll_y, bounds.size.height, line_height);
        let visible = &lines[visible_lines.clone()];
        let scrolled = Bounds::new(point(bounds.left(), bounds.top() - scroll_y), bounds.size);
        let t = theme(cx);
        let line_backgrounds = line_background_quads(visible, scrolled, line_height, t);
        let selections = selection_quads(visible, &selected_range, scrolled, line_height, t);
        let cursor = if selected_range.is_empty() && caret_visible {
            cursor_quad(visible, cursor_offset, scrolled, line_height, t.fg)
        } else {
            None
        };

        PrepaintState {
            key,
            lines,
            visible_lines,
            line_height,
            cursor,
            line_backgrounds,
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
            for background in prepaint.line_backgrounds.drain(..) {
                window.paint_quad(background);
            }
            for selection in prepaint.selections.drain(..) {
                window.paint_quad(selection);
            }
            for line in &prepaint.lines[prepaint.visible_lines.clone()] {
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

        let layout = TextLayout {
            key: prepaint.key.clone(),
            lines: prepaint.lines.clone(),
            line_height: prepaint.line_height,
        };
        self.input.update(cx, |input, _| {
            input.last_layout = Some(layout);
            input.last_bounds = Some(bounds);
        });
    }
}

fn layout_key(
    bounds: Bounds<Pixels>,
    window: &Window,
    theme: &crate::app::theme::Theme,
) -> TextLayoutKey {
    let style = window.text_style();
    TextLayoutKey {
        width: bounds.size.width,
        font: style.font(),
        font_size: style.font_size.to_pixels(window.rem_size()),
        line_height: window.line_height(),
        text_color: style.color,
        theme_colors: [
            theme.diff_added_bg,
            theme.diff_removed_bg,
            theme.diff_added_word_bg,
            theme.diff_removed_word_bg,
            theme.tok_keyword,
            theme.tok_string,
            theme.tok_comment,
            theme.tok_number,
            theme.tok_type,
        ],
    }
}

fn visible_line_range(
    line_count: usize,
    scroll_y: Pixels,
    viewport_height: Pixels,
    line_height: Pixels,
) -> Range<usize> {
    let line_height = f32::from(line_height).max(1.);
    let start = (f32::from(scroll_y) / line_height).floor() as usize;
    let end =
        ((f32::from(scroll_y + viewport_height) / line_height).ceil() as usize + 1).min(line_count);
    start.min(line_count)..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_lines_only_cover_the_viewport() {
        assert_eq!(
            visible_line_range(1_000, px(180.), px(360.), px(18.)),
            10..31
        );
        assert_eq!(visible_line_range(12, px(180.), px(360.), px(18.)), 10..12);
    }

    #[gpui::test]
    fn scrolling_reuses_the_shaped_line_layout(cx: &mut gpui::TestAppContext) {
        use gpui::{ScrollDelta, ScrollWheelEvent, TouchPhase, VisualTestContext, size};

        cx.update(|cx| cx.set_global(crate::app::theme::Theme::light()));
        let content = (0..1_000)
            .map(|line| format!("let value_{line} = {line};"))
            .collect::<Vec<_>>()
            .join("\n");
        let (input, cx) = cx
            .add_window_view(|_, cx| TextArea::new(content, "", true, 360., cx).starting_at_top());
        let cx: &mut VisualTestContext = cx;
        cx.simulate_resize(size(px(640.), px(360.)));
        cx.run_until_parked();
        let before = input.read_with(cx, |input, _| {
            input.last_layout.as_ref().unwrap().lines.clone()
        });

        cx.simulate_event(ScrollWheelEvent {
            position: point(px(100.), px(100.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(-180.))),
            modifiers: Default::default(),
            touch_phase: TouchPhase::Moved,
        });
        cx.run_until_parked();

        input.read_with(cx, |input, _| {
            assert!(Arc::ptr_eq(
                &before,
                &input.last_layout.as_ref().unwrap().lines
            ));
            assert_eq!(input.scroll_offset_y(), px(180.));
        });
    }
}
