use std::{ops::Range, sync::Arc};

use gpui::{
    App, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable, Font, Hsla, Pixels,
    ShapedLine, SharedString, Subscription, px,
};
use jayjay_core::diff::DiffSpanStyle;

use crate::app::theme::Theme;
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
    read_only: bool,
    presentation: TextAreaPresentation,
    pub(super) line_numbers: bool,
    pub(super) height: f32,
    pub(super) line_height: f32,
    /// Clamped in prepaint, where geometry is known.
    pub(super) scroll_y: Pixels,
    pub(super) scroll_caret_into_view: bool,
    pub(super) caret: CaretBlink,
    pub(super) focus_subscriptions: Vec<Subscription>,
    syntax: Option<SyntaxHighlightState>,
    pub(super) tints: Vec<Tint>,
}

/// The color is read from the theme at layout time so it follows appearance changes.
#[derive(Clone)]
pub(crate) struct Tint {
    pub(crate) range: Range<usize>,
    pub(crate) color: fn(&Theme) -> u32,
}

pub(crate) struct TextAreaUpdated;

/// A user-driven scroll: wheel input, or the caret being pulled into view.
pub(crate) struct TextAreaScrolled;

impl EventEmitter<TextAreaUpdated> for TextArea {}
impl EventEmitter<TextAreaScrolled> for TextArea {}

#[derive(Clone, Copy)]
enum TextAreaMode {
    Editable,
    SelectableCode { emphasized_line: Option<usize> },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextAreaPresentation {
    Field,
    FullBleedPane,
    Label,
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
    pub(super) theme_colors: [u32; 11],
}

pub(in crate::ui::text_area) struct LineLayout {
    pub(super) logical_line: usize,
    pub(super) range: Range<usize>,
    pub(super) shaped: ShapedLine,
    pub(super) number: Option<ShapedLine>,
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
            read_only: false,
            presentation: TextAreaPresentation::Field,
            line_numbers: false,
            height,
            line_height: 18.,
            scroll_y: px(0.),
            scroll_caret_into_view: false,
            caret: CaretBlink::default(),
            focus_subscriptions: Vec::new(),
            syntax: None,
            tints: Vec::new(),
        }
    }

    pub(crate) fn starting_at_top(mut self) -> Self {
        self.selection = TextSelection::at(0);
        self.scroll_y = px(0.);
        self.scroll_caret_into_view = false;
        self
    }

    pub(crate) fn label(content: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let mut view = Self::new(content, "", true, 0., cx).starting_at_top();
        view.presentation = TextAreaPresentation::Label;
        view.set_read_only(true, cx);
        view
    }

    pub(crate) fn with_tints(mut self, tints: Vec<Tint>) -> Self {
        self.tints = tints;
        self
    }

    pub(crate) fn is_label(&self) -> bool {
        self.presentation == TextAreaPresentation::Label
    }

    pub(crate) fn full_bleed_pane(mut self) -> Self {
        self.presentation = TextAreaPresentation::FullBleedPane;
        self
    }

    pub fn with_line_numbers(mut self) -> Self {
        self.line_numbers = true;
        self
    }

    pub fn line_number_rows(&self) -> Vec<Option<usize>> {
        self.last_layout.as_ref().map_or_else(Vec::new, |layout| {
            layout
                .lines
                .iter()
                .map(|line| line.number.as_ref().map(|_| line.logical_line + 1))
                .collect()
        })
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
        matches!(self.mode, TextAreaMode::Editable) && !self.read_only
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Keeps the field's look but rejects edits.
    pub fn set_read_only(&mut self, read_only: bool, cx: &mut Context<Self>) {
        if self.read_only == read_only {
            return;
        }
        self.read_only = read_only;
        if read_only {
            self.caret.hide(cx);
        } else {
            self.show_caret(cx);
        }
        cx.notify();
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

    pub(in crate::ui::text_area) fn logical_line_count(&self) -> usize {
        self.content.matches('\n').count() + 1
    }

    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.replace_text(text, true, cx);
    }

    pub(crate) fn set_text_keeping_scroll(
        &mut self,
        text: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.replace_text(text, false, cx);
    }

    fn replace_text(
        &mut self,
        text: impl Into<SharedString>,
        reset_scroll: bool,
        cx: &mut Context<Self>,
    ) {
        self.content = text.into();
        let end = self.content.len();
        self.selection = TextSelection::at(end);
        self.marked_range = None;
        self.last_layout = None;
        self.schedule_syntax_highlight(false, cx);
        cx.emit(TextAreaUpdated);
        self.show_caret(cx);
        // The caret waits at the end for the next keystroke to scroll to it.
        self.scroll_caret_into_view = false;
        if reset_scroll {
            self.scroll_y = px(0.);
        }
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
