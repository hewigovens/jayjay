use std::{ops::Range, sync::Arc};

use gpui::{
    App, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable, Font, Hsla, Pixels,
    ShapedLine, SharedString, Subscription, px,
};
use jayjay_core::diff::DiffSpanStyle;

use crate::ui::input::{CaretBlink, TextSelection};

mod highlighting;

use highlighting::SyntaxHighlightState;

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
    presentation: TextAreaPresentation,
    pub(super) height: f32,
    pub(super) line_height: f32,
    /// Clamped in prepaint, where geometry is known.
    pub(super) scroll_y: Pixels,
    pub(super) scroll_caret_into_view: bool,
    pub(super) caret: CaretBlink,
    pub(super) focus_subscriptions: Vec<Subscription>,
    syntax: Option<SyntaxHighlightState>,
}

pub(crate) struct TextAreaUpdated;

impl EventEmitter<TextAreaUpdated> for TextArea {}

#[derive(Clone, Copy)]
enum TextAreaMode {
    Editable,
    SelectableCode { emphasized_line: Option<usize> },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextAreaPresentation {
    Field,
    FullBleedPane,
}

pub(in crate::ui::text_area) struct TextLayout {
    pub(super) key: TextLayoutKey,
    pub(super) lines: Arc<[LineLayout]>,
    pub(super) line_height: Pixels,
}

#[derive(Clone, PartialEq)]
pub(in crate::ui::text_area) struct TextLayoutKey {
    pub(super) width: Pixels,
    pub(super) font: Font,
    pub(super) font_size: Pixels,
    pub(super) line_height: Pixels,
    pub(super) text_color: Hsla,
    pub(super) theme_colors: [u32; 9],
}

pub(in crate::ui::text_area) struct LineLayout {
    pub(super) range: Range<usize>,
    pub(super) shaped: ShapedLine,
    pub(super) top: Pixels,
    pub(super) style: DiffSpanStyle,
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
            presentation: TextAreaPresentation::Field,
            height,
            line_height: 18.,
            scroll_y: px(0.),
            scroll_caret_into_view: false,
            caret: CaretBlink::default(),
            focus_subscriptions: Vec::new(),
            syntax: None,
        }
    }

    pub(crate) fn starting_at_top(mut self) -> Self {
        self.selection = TextSelection::at(0);
        self.scroll_y = px(0.);
        self.scroll_caret_into_view = false;
        self
    }

    pub(crate) fn full_bleed_pane(mut self) -> Self {
        self.presentation = TextAreaPresentation::FullBleedPane;
        self
    }

    pub(crate) fn subscribe_updates<T: 'static>(text_area: &Entity<Self>, cx: &mut Context<T>) {
        cx.subscribe(text_area, |_, _, _: &TextAreaUpdated, cx| cx.notify())
            .detach();
    }

    pub(crate) fn selectable_code_block(
        content: impl Into<SharedString>,
        line_count: usize,
        emphasized_line: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Self {
        let line_height = 22.;
        let mut view = Self::new(content, "", true, line_height * line_count as f32, cx);
        view.mode = TextAreaMode::SelectableCode { emphasized_line };
        view.line_height = line_height;
        view
    }

    pub(super) fn is_editable(&self) -> bool {
        matches!(self.mode, TextAreaMode::Editable)
    }

    pub(super) fn is_full_bleed_pane(&self) -> bool {
        self.presentation == TextAreaPresentation::FullBleedPane
    }

    pub(super) fn is_selectable_code(&self) -> bool {
        matches!(self.mode, TextAreaMode::SelectableCode { .. })
    }

    pub(super) fn uses_code_font(&self) -> bool {
        self.syntax.is_some() || self.is_selectable_code()
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
        self.schedule_syntax_highlight(false, cx);
        cx.emit(TextAreaUpdated);
        self.show_caret(cx);
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.set_text("", cx);
    }

    pub(super) fn content_changed(&mut self, cx: &mut Context<Self>) {
        self.last_layout = None;
        self.schedule_syntax_highlight(true, cx);
        cx.emit(TextAreaUpdated);
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
