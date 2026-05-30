mod editing;
mod element;
mod input;
mod render;

use std::ops::Range;

use gpui::{
    App, Bounds, Context, FocusHandle, Focusable, Pixels, ShapedLine, SharedString, actions, div,
    px, rgb,
};

use crate::app::theme::Theme;

actions!(
    text_area,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Newline,
        Paste,
        Cut,
        Copy
    ]
);

pub struct TextArea {
    pub(super) focus_handle: FocusHandle,
    pub(super) content: SharedString,
    pub(super) placeholder: SharedString,
    pub(super) selected_range: Range<usize>,
    pub(super) selection_reversed: bool,
    pub(super) marked_range: Option<Range<usize>>,
    pub(super) last_layout: Option<TextLayout>,
    pub(super) last_bounds: Option<Bounds<Pixels>>,
    pub(super) is_selecting: bool,
    pub(super) multiline: bool,
    pub(super) height: f32,
}

pub(super) struct TextLayout {
    pub(super) lines: Vec<LineLayout>,
    pub(super) line_height: Pixels,
}

pub(super) struct LineLayout {
    pub(super) range: Range<usize>,
    pub(super) shaped: ShapedLine,
    pub(super) top: Pixels,
}

impl TextArea {
    pub fn new(
        content: impl Into<SharedString>,
        placeholder: impl Into<SharedString>,
        multiline: bool,
        height: f32,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = content.into();
        let end = content.len();
        Self {
            focus_handle: cx.focus_handle(),
            content,
            placeholder: placeholder.into(),
            selected_range: end..end,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            multiline,
            height,
        }
    }

    pub fn text(&self) -> String {
        self.content.to_string()
    }

    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        let end = self.content.len();
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        self.last_layout = None;
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.set_text("", cx);
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

pub fn button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
    primary: bool,
) -> gpui::Stateful<gpui::Div> {
    use gpui::{InteractiveElement, ParentElement, Styled};

    let (bg, fg) = if primary {
        (theme.toggle_active_bg, theme.toggle_active_fg)
    } else {
        (theme.toggle_inactive_bg, theme.toggle_inactive_fg)
    };
    div()
        .id(id.into())
        .flex()
        .items_center()
        .justify_center()
        .px(px(10.))
        .h(px(28.))
        .rounded_sm()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(12.))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme.row_alt_bg)))
        .child(label.into())
}

pub(super) fn line_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (ix, ch) in text.char_indices() {
        if ch == '\n' {
            ranges.push(start..ix);
            start = ix + ch.len_utf8();
        }
    }
    ranges.push(start..text.len());
    ranges
}
