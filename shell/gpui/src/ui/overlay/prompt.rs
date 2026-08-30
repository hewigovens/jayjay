use gpui::{
    AnyElement, App, AppContext, Context, Entity, Focusable, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::{overlay_actions, overlay_card, overlay_header, overlay_layer};
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::primitives::button;
use crate::ui::text_area::TextArea;

/// Window-agnostic single-field prompt. Each window owns one and supplies submit/cancel.
pub(crate) struct TextPrompt {
    pub title: SharedString,
    pub subtitle: SharedString,
    pub primary_label: SharedString,
    pub input: Entity<TextArea>,
    focus_pending: bool,
}

pub(crate) struct PromptStyle {
    pub width: f32,
    pub input_id: Option<&'static str>,
    pub cancel_id: &'static str,
    pub primary_id: &'static str,
    pub key_context: Option<&'static str>,
    pub primary_enabled: bool,
}

impl PromptStyle {
    pub fn new(width: f32, cancel_id: &'static str, primary_id: &'static str) -> Self {
        Self {
            width,
            input_id: None,
            cancel_id,
            primary_id,
            key_context: None,
            primary_enabled: true,
        }
    }
}

pub(crate) struct PromptSlots {
    before_input: Vec<AnyElement>,
    after_input: Vec<AnyElement>,
}

impl PromptSlots {
    pub fn new(
        before_input: impl IntoIterator<Item = AnyElement>,
        after_input: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            before_input: before_input.into_iter().collect(),
            after_input: after_input.into_iter().collect(),
        }
    }
}

impl TextPrompt {
    pub fn single_line<V: 'static>(
        title: impl Into<SharedString>,
        subtitle: impl Into<SharedString>,
        initial: impl Into<SharedString>,
        placeholder: impl Into<SharedString>,
        primary_label: impl Into<SharedString>,
        cx: &mut Context<V>,
    ) -> Self {
        Self::build(
            title,
            subtitle,
            initial,
            placeholder,
            primary_label,
            false,
            32.,
            cx,
        )
    }

    pub fn multiline<V: 'static>(
        title: impl Into<SharedString>,
        subtitle: impl Into<SharedString>,
        initial: impl Into<SharedString>,
        placeholder: impl Into<SharedString>,
        primary_label: impl Into<SharedString>,
        height: f32,
        cx: &mut Context<V>,
    ) -> Self {
        Self::build(
            title,
            subtitle,
            initial,
            placeholder,
            primary_label,
            true,
            height,
            cx,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build<V: 'static>(
        title: impl Into<SharedString>,
        subtitle: impl Into<SharedString>,
        initial: impl Into<SharedString>,
        placeholder: impl Into<SharedString>,
        primary_label: impl Into<SharedString>,
        multiline: bool,
        height: f32,
        cx: &mut Context<V>,
    ) -> Self {
        let input = cx.new(|cx| TextArea::new(initial, placeholder, multiline, height, cx));
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            primary_label: primary_label.into(),
            input,
            focus_pending: true,
        }
    }

    pub fn take_focus(&mut self, window: &mut Window, cx: &mut App) {
        if !self.focus_pending {
            return;
        }
        window.focus(&self.input.read(cx).focus_handle(cx), cx);
        self.focus_pending = false;
    }

    pub fn text(&self, cx: &App) -> String {
        self.input.read(cx).text()
    }

    pub(crate) fn overlay<V: 'static>(
        &self,
        style: &PromptStyle,
        t: &Theme,
        cx: &mut Context<V>,
        slots: PromptSlots,
        on_cancel: impl Fn(&mut V, &mut Context<V>) + 'static,
        on_submit: impl Fn(&mut V, &mut Context<V>) + 'static,
    ) -> AnyElement {
        let mut panel = overlay_card(t, style.width).child(overlay_header(
            glyph::PENCIL_CIRCLE,
            t.fg_dim,
            self.title.clone(),
            self.subtitle.clone(),
            t,
        ));
        if let Some(key_context) = style.key_context {
            panel = panel.key_context(key_context);
        }
        for child in slots.before_input {
            panel = panel.child(child);
        }
        panel = panel.child(match style.input_id {
            Some(id) => div()
                .id(id)
                .debug_selector(|| id.to_owned())
                .child(self.input.clone())
                .into_any_element(),
            None => self.input.clone().into_any_element(),
        });
        for child in slots.after_input {
            panel = panel.child(child);
        }
        let primary = button(style.primary_id, self.primary_label.clone(), t, true)
            .debug_selector(|| style.primary_id.to_owned());
        let primary = if style.primary_enabled {
            primary.on_click(cx.listener(move |view, _, _, cx| on_submit(view, cx)))
        } else {
            primary.opacity(0.45)
        };
        panel = panel.child(overlay_actions(
            button(style.cancel_id, "Cancel", t, false)
                .debug_selector(|| style.cancel_id.to_owned())
                .on_click(cx.listener(move |view, _, _, cx| on_cancel(view, cx))),
            primary,
        ));
        overlay_layer().child(panel).into_any_element()
    }
}
