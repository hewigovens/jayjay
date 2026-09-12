use gpui::{
    AnyElement, App, AppContext, Context, Entity, IntoElement, ParentElement, Styled, div, px,
};
use jayjay_core::commit_message;

use super::text_area::TextArea;

pub(crate) struct CommitMessageEditor {
    original: String,
    pub summary: Entity<TextArea>,
    pub body: Entity<TextArea>,
}

impl CommitMessageEditor {
    pub fn new<V: 'static>(message: &str, body_height: f32, cx: &mut Context<V>) -> Self {
        Self {
            original: message.to_owned(),
            summary: cx.new(|cx| {
                TextArea::new(commit_message::summary(message), "Summary", false, 32., cx)
            }),
            body: cx.new(|cx| {
                TextArea::new(
                    commit_message::body(message),
                    "Description (optional)",
                    true,
                    body_height,
                    cx,
                )
            }),
        }
    }

    pub fn text(&self, cx: &App) -> String {
        commit_message::update(
            &self.original,
            &self.summary.read(cx).text(),
            &self.body.read(cx).text(),
        )
    }

    pub fn element(&self) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(self.summary.clone())
            .child(self.body.clone())
            .into_any_element()
    }
}
