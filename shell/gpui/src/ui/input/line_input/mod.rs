mod owner;

use gpui::{ClipboardItem, Context, KeyDownEvent};

use super::{CaretBlink, LineEdit, LineEditKeyResult};

pub type LineInputSelector<T> = for<'a> fn(&'a mut T) -> Option<&'a mut LineInput>;

#[derive(Debug, Default)]
pub struct LineInput {
    edit: LineEdit,
    caret: CaretBlink,
}

impl LineInput {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            edit: LineEdit::new(text),
            caret: CaretBlink::default(),
        }
    }

    pub fn edit(&self) -> &LineEdit {
        &self.edit
    }

    pub fn text(&self) -> &str {
        self.edit.text()
    }

    pub fn is_empty(&self) -> bool {
        self.edit.is_empty()
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.edit.set_text(text);
    }

    pub fn clear(&mut self) {
        self.set_text("");
    }

    pub fn handle_key<T>(&mut self, ev: &KeyDownEvent, cx: &mut Context<T>) -> LineEditKeyResult {
        let clipboard_text = cx.read_from_clipboard().and_then(|item| item.text());
        let result = self.edit.handle_key(ev, clipboard_text.as_deref());
        if let Some(text) = result.copy_to_clipboard.as_ref() {
            cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
        }
        result
    }

    pub fn caret_visible(&self) -> bool {
        self.caret.visible()
    }

    pub fn show_caret<T>(
        &mut self,
        cx: &mut Context<T>,
        tick: impl FnMut(&mut T, u64, &mut Context<T>) -> bool + 'static,
    ) where
        T: 'static,
    {
        self.caret.show(cx, tick);
    }

    pub fn hide_caret<T: 'static>(&mut self, cx: &mut Context<T>) {
        self.caret.hide(cx);
    }

    pub fn toggle_caret<T: 'static>(&mut self, generation: u64, cx: &mut Context<T>) -> bool {
        self.caret.toggle_if_current(generation, cx)
    }
}
