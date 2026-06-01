mod editing;
mod element;
mod input;
mod render;

use std::ops::Range;

use gpui::{
    App, Bounds, Context, FocusHandle, Focusable, KeyBinding, Pixels, ShapedLine, SharedString,
    Subscription, actions,
};

use crate::ui::input::CaretBlink;

actions!(
    text_area,
    [
        Backspace,
        Delete,
        DeleteToLineStart,
        DeletePreviousWord,
        Left,
        Right,
        WordLeft,
        WordRight,
        SelectLeft,
        SelectRight,
        SelectWordLeft,
        SelectWordRight,
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
    pub(super) caret: CaretBlink,
    pub(super) focus_subscriptions: Vec<Subscription>,
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
            caret: CaretBlink::default(),
            focus_subscriptions: Vec::new(),
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
        self.show_caret(cx);
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.set_text("", cx);
    }
}

pub fn key_bindings(mod_key: &str) -> Vec<KeyBinding> {
    let mut bindings = vec![
        KeyBinding::new("backspace", Backspace, Some("TextArea")),
        KeyBinding::new("delete", Delete, Some("TextArea")),
        KeyBinding::new("left", Left, Some("TextArea")),
        KeyBinding::new("right", Right, Some("TextArea")),
        KeyBinding::new("alt-left", WordLeft, Some("TextArea")),
        KeyBinding::new("alt-right", WordRight, Some("TextArea")),
        KeyBinding::new("shift-left", SelectLeft, Some("TextArea")),
        KeyBinding::new("shift-right", SelectRight, Some("TextArea")),
        KeyBinding::new("alt-shift-left", SelectWordLeft, Some("TextArea")),
        KeyBinding::new("alt-shift-right", SelectWordRight, Some("TextArea")),
        KeyBinding::new("alt-backspace", DeletePreviousWord, Some("TextArea")),
        KeyBinding::new("alt-delete", DeletePreviousWord, Some("TextArea")),
        KeyBinding::new("home", Home, Some("TextArea")),
        KeyBinding::new("end", End, Some("TextArea")),
        KeyBinding::new("enter", Newline, Some("TextArea")),
        KeyBinding::new(format!("{mod_key}-a").as_str(), SelectAll, Some("TextArea")),
        KeyBinding::new(format!("{mod_key}-v").as_str(), Paste, Some("TextArea")),
        KeyBinding::new(format!("{mod_key}-x").as_str(), Cut, Some("TextArea")),
        KeyBinding::new(format!("{mod_key}-c").as_str(), Copy, Some("TextArea")),
    ];
    if mod_key == "cmd" {
        bindings.extend([
            KeyBinding::new("ctrl-a", Home, Some("TextArea")),
            KeyBinding::new("ctrl-e", End, Some("TextArea")),
            KeyBinding::new("cmd-backspace", DeleteToLineStart, Some("TextArea")),
            KeyBinding::new("cmd-delete", DeleteToLineStart, Some("TextArea")),
        ]);
    }
    bindings
}

impl Focusable for TextArea {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
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
