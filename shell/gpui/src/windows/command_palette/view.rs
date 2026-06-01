use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, rgb,
};

use super::raw::command_view;
use super::render::{action_list, divider, query_box};
use super::state::{CommandOutput, CommandPalette};
use crate::app::theme::theme;

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_focus_handlers(window, cx);
        let t = theme(cx).clone();
        let visible = self.matches();
        let selected = self.selected.min(visible.len().saturating_sub(1));

        let body = match (&self.output, self.parse_command()) {
            (CommandOutput::Idle, None) => {
                action_list(&visible, selected, &t, cx).into_any_element()
            }
            (CommandOutput::Idle, Some(body)) => {
                let cmd = format!("jj {body}");
                command_view(None, cmd.trim_end(), &self.history, &t, cx).into_any_element()
            }
            (out, _) => command_view(Some(out), "", &self.history, &t, cx).into_any_element(),
        };

        div()
            .track_focus(&self.focus_handle)
            .key_context("CommandPalette")
            .on_key_down(cx.listener(Self::on_key))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(query_box(&self.query, &t))
            .child(divider(&t))
            .child(body)
    }
}
