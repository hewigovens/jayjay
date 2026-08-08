mod owner;

use gpui::{ClipboardItem, Context, KeyDownEvent, ScrollHandle, point, px};

use super::{CaretBlink, LineEdit, LineEditKeyResult};

pub type LineInputSelector<T> = for<'a> fn(&'a mut T) -> Option<&'a mut LineInput>;

#[derive(Debug, Default)]
pub struct LineInput {
    edit: LineEdit,
    caret: CaretBlink,
    scroll: ScrollHandle,
}

impl LineInput {
    pub(crate) fn new(text: impl Into<String>) -> Self {
        Self {
            edit: LineEdit::new(text),
            caret: CaretBlink::default(),
            scroll: ScrollHandle::new(),
        }
    }

    pub(crate) fn edit(&self) -> &LineEdit {
        &self.edit
    }

    pub(crate) fn text(&self) -> &str {
        self.edit.text()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.edit.is_empty()
    }

    pub(crate) fn set_text(&mut self, text: impl Into<String>) {
        self.edit.set_text(text);
        self.reveal_cursor_edge();
    }

    pub(crate) fn clear(&mut self) {
        self.set_text("");
    }

    pub(crate) fn handle_key<T>(
        &mut self,
        ev: &KeyDownEvent,
        cx: &mut Context<T>,
    ) -> LineEditKeyResult {
        let clipboard_text = cx.read_from_clipboard().and_then(|item| item.text());
        let result = self.edit.handle_key(ev, clipboard_text.as_deref());
        if let Some(text) = result.copy_to_clipboard.as_ref() {
            cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
        }
        if result.handled {
            self.reveal_cursor_edge();
        }
        result
    }

    pub(crate) fn caret_visible(&self) -> bool {
        self.caret.visible()
    }

    pub(crate) fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub(crate) fn reveal_cursor_edge(&self) {
        let current = self.scroll.offset();
        let x = if self.edit.cursor_offset() == 0 {
            px(0.)
        } else if self.edit.cursor_offset() == self.edit.text().len() {
            -self.scroll.max_offset().x
        } else {
            current.x
        };
        self.scroll.set_offset(point(x, current.y));
    }

    fn show_caret<T>(
        &mut self,
        cx: &mut Context<T>,
        tick: impl FnMut(&mut T, u64, &mut Context<T>) -> bool + 'static,
    ) where
        T: 'static,
    {
        self.caret.show(cx, tick);
    }

    fn hide_caret<T: 'static>(&mut self, cx: &mut Context<T>) {
        self.caret.hide(cx);
    }

    fn toggle_caret<T: 'static>(&mut self, generation: u64, cx: &mut Context<T>) -> bool {
        self.caret.toggle_if_current(generation, cx)
    }
}
