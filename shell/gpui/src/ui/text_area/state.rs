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
    mode: TextAreaMode,
    pub(super) height: f32,
    pub(super) line_height: f32,
    /// Clamped in prepaint, where geometry is known.
    pub(super) scroll_y: Pixels,
    pub(super) scroll_caret_into_view: bool,
    pub(super) caret: CaretBlink,
    pub(super) focus_subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
enum TextAreaMode {
    Editable,
    SelectableCode { emphasized_line: Option<usize> },
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
            mode: TextAreaMode::Editable,
            height,
            line_height: 18.,
            scroll_y: px(0.),
            scroll_caret_into_view: false,
            caret: CaretBlink::default(),
            focus_subscriptions: Vec::new(),
        }
    }

    pub(crate) fn selectable_code_block(
        content: impl Into<SharedString>,
        line_count: usize,
        emphasized_line: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Self {
        let line_height = 22.;
        let mut view = Self::new(content, "", false, line_height * line_count as f32, cx);
        view.mode = TextAreaMode::SelectableCode { emphasized_line };
        view.line_height = line_height;
        view
    }

    pub(super) fn is_editable(&self) -> bool {
        matches!(self.mode, TextAreaMode::Editable)
    }

    pub(super) fn is_selectable_code(&self) -> bool {
        matches!(self.mode, TextAreaMode::SelectableCode { .. })
    }

    pub(super) fn emphasized_line(&self) -> Option<usize> {
        match self.mode {
            TextAreaMode::Editable => None,
            TextAreaMode::SelectableCode { emphasized_line } => emphasized_line,
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
