use gpui::{
    Context, CursorStyle, InteractiveElement, IntoElement, MouseButton, ParentElement, Render,
    Styled, Window, div, px, rgb, rgba,
};

use super::TextArea;
use super::element::TextAreaElement;
use crate::app::theme::{FONT_BODY, theme, ui_font_size, with_alpha};

impl Render for TextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_focus_handlers(window, cx);
        let t = theme(cx).clone();
        let fill = with_alpha(t.fg, if t.is_dark { 0x12 } else { 0x0a });
        let border = with_alpha(t.fg, if t.is_dark { 0x24 } else { 0x1a });
        let uses_code_font = self.uses_code_font();
        let line_height = if uses_code_font {
            self.line_height.max(t.code_line_height())
        } else {
            self.line_height
        };
        let mut root = div()
            .track_focus(&self.focus_handle)
            .key_context("TextArea")
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_to_line_start))
            .on_action(cx.listener(Self::delete_to_line_end))
            .on_action(cx.listener(Self::delete_previous_word))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::word_left))
            .on_action(cx.listener(Self::word_right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::select_word_left))
            .on_action(cx.listener(Self::select_word_right))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::document_start))
            .on_action(cx.listener(Self::document_end))
            .on_action(cx.listener(Self::select_document_start))
            .on_action(cx.listener(Self::select_document_end))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_scroll_wheel(cx.listener(Self::on_scroll_wheel))
            .w_full()
            .text_color(rgb(t.fg))
            .line_height(px(line_height));
        if uses_code_font {
            let font_size = if self.is_selectable_code() {
                t.compact_code_font_size()
            } else {
                t.font_size
            };
            root = root
                .font_family(crate::app::fonts::mono())
                .text_size(px(font_size));
        } else {
            root = root.text_size(ui_font_size(FONT_BODY));
        }
        let content_height = if self.is_full_bleed_pane() {
            root = root.h_full().bg(rgba(fill)).p(px(10.));
            None
        } else if self.is_editable() {
            root = root
                .h(px(self.height))
                .rounded_md()
                .border_1()
                .border_color(rgba(border))
                .bg(rgba(fill))
                .p(px(6.));
            Some(self.height - 12.)
        } else {
            let line_count = self.content.matches('\n').count() + 1;
            let height = self.height + (line_height - self.line_height) * line_count as f32;
            root = root.h(px(height));
            Some(height)
        };
        root.child(TextAreaElement {
            input: cx.entity(),
            height: content_height,
        })
    }
}
