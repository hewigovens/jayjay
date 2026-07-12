use std::ops::Range;

use gpui::{
    App, Bounds, Context, FocusHandle, Focusable, Pixels, ShapedLine, SharedString, Subscription,
    px,
};

use crate::ui::input::{CaretBlink, TextSelection};

pub struct TextArea {
    pub(super) focus_handle: FocusHandle,
    pub(super) content: SharedString,
    pub(super) placeholder: SharedString,
    pub(super) selection: TextSelection,
    pub(super) marked_range: Option<Range<usize>>,
    pub(super) last_layout: Option<TextLayout>,
    pub(super) last_bounds: Option<Bounds<Pixels>>,
    pub(super) is_selecting: bool,
    pub(super) multiline: bool,
    pub(super) height: f32,
    /// Clamped in prepaint, where geometry is known.
    pub(super) scroll_y: Pixels,
    pub(super) scroll_caret_into_view: bool,
    pub(super) caret: CaretBlink,
    pub(super) focus_subscriptions: Vec<Subscription>,
}

pub(in crate::ui::text_area) struct TextLayout {
    pub(super) lines: Vec<LineLayout>,
    pub(super) line_height: Pixels,
}

pub(in crate::ui::text_area) struct LineLayout {
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
            selection: TextSelection::at(end),
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            multiline,
            height,
            scroll_y: px(0.),
            scroll_caret_into_view: false,
            caret: CaretBlink::default(),
            focus_subscriptions: Vec::new(),
        }
    }

    pub fn text(&self) -> String {
        self.content.to_string()
    }

    pub fn scroll_offset_y(&self) -> Pixels {
        self.scroll_y
    }

    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        let end = self.content.len();
        self.selection = TextSelection::at(end);
        self.marked_range = None;
        self.last_layout = None;
        self.show_caret(cx);
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
