use gpui::{
    AnyElement, App, AppContext, Context, Entity, Focusable, InteractiveElement, IntoElement,
    ParentElement, Pixels, Point, SharedString, Size, StatefulInteractiveElement, Styled, Window,
    div, px,
};

use super::{overlay_actions, overlay_card, overlay_header, overlay_layer};
use crate::app::theme::Theme;
use crate::ui::commit_message_editor::CommitMessageEditor;
use crate::ui::icons::glyph;
use crate::ui::pane_drag::pane_max;
use crate::ui::primitives::button;
use crate::ui::text_area::TextArea;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PromptSize {
    pub width: f32,
    pub body_height: f32,
}

impl PromptSize {
    pub(crate) const DEFAULT: Self = Self {
        width: 680.,
        body_height: 300.,
    };
    const MIN: Self = Self {
        width: 520.,
        body_height: 190.,
    };
    const MAX: Self = Self {
        width: 960.,
        body_height: 600.,
    };
    const WINDOW_MARGIN: f32 = 64.;
    /// The card's header, summary field, actions and padding around the body.
    const CHROME_HEIGHT: f32 = 170.;

    fn fitted(self, viewport: Size<Pixels>) -> Self {
        let room_width = f32::from(viewport.width) - Self::WINDOW_MARGIN;
        let room_height = f32::from(viewport.height) - Self::WINDOW_MARGIN - Self::CHROME_HEIGHT;
        // `max` then `min` rather than `clamp`, which passes a NaN from a hand-edited config straight through.
        Self {
            width: self.width.max(Self::MIN.width).min(pane_max(
                Self::MIN.width,
                Self::MAX.width,
                room_width,
            )),
            body_height: self.body_height.max(Self::MIN.body_height).min(pane_max(
                Self::MIN.body_height,
                Self::MAX.body_height,
                room_height,
            )),
        }
    }
}

/// Window-owned prompt with shared input controls and submit/cancel actions.
pub(crate) struct TextPrompt {
    pub title: SharedString,
    pub subtitle: SharedString,
    pub primary_label: SharedString,
    input: PromptInput,
    focus_pending: bool,
    /// What the user asked for; `shown` is that size fitted to the window, so a small window does not overwrite it.
    size: Option<PromptSize>,
    shown: Option<PromptSize>,
    resize: Option<(Point<Pixels>, PromptSize)>,
}

enum PromptInput {
    Text(Entity<TextArea>),
    CommitMessage(CommitMessageEditor),
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
    leading_actions: Vec<AnyElement>,
    corner: Option<AnyElement>,
}

impl PromptSlots {
    pub fn new(
        before_input: impl IntoIterator<Item = AnyElement>,
        after_input: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            before_input: before_input.into_iter().collect(),
            after_input: after_input.into_iter().collect(),
            leading_actions: Vec::new(),
            corner: None,
        }
    }

    pub fn with_leading_actions(mut self, actions: Vec<AnyElement>) -> Self {
        self.leading_actions = actions;
        self
    }

    pub fn with_corner(mut self, corner: Option<AnyElement>) -> Self {
        self.corner = corner;
        self
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
            input: PromptInput::Text(input),
            focus_pending: true,
            size: None,
            shown: None,
            resize: None,
        }
    }

    pub fn take_focus(&mut self, window: &mut Window, cx: &mut App) {
        if !self.focus_pending {
            return;
        }
        window.focus(&self.input().read(cx).focus_handle(cx), cx);
        self.focus_pending = false;
    }

    pub fn text(&self, cx: &App) -> String {
        match &self.input {
            PromptInput::Text(input) => input.read(cx).text(),
            PromptInput::CommitMessage(editor) => editor.text(cx),
        }
    }

    pub fn commit_message<V: 'static>(
        title: impl Into<SharedString>,
        subtitle: impl Into<SharedString>,
        initial: &str,
        primary_label: impl Into<SharedString>,
        size: PromptSize,
        cx: &mut Context<V>,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            primary_label: primary_label.into(),
            input: PromptInput::CommitMessage(CommitMessageEditor::new(
                initial,
                size.body_height,
                cx,
            )),
            focus_pending: true,
            size: Some(size),
            shown: Some(size),
            resize: None,
        }
    }

    pub fn is_resizable(&self) -> bool {
        self.size.is_some()
    }

    pub fn is_resizing(&self) -> bool {
        self.resize.is_some()
    }

    pub fn start_resize(&mut self, position: Point<Pixels>) {
        self.resize = self.shown.map(|shown| (position, shown));
    }

    pub fn resize_to(&mut self, position: Point<Pixels>, viewport: Size<Pixels>, cx: &mut App) {
        let Some((start, from)) = self.resize else {
            return;
        };
        // The card stays centred, so it grows twice as far as the grip moves.
        let dragged = PromptSize {
            width: from.width + 2. * f32::from(position.x - start.x),
            body_height: from.body_height + 2. * f32::from(position.y - start.y),
        }
        .fitted(viewport);
        self.size = Some(dragged);
        self.show(dragged, cx);
    }

    pub fn end_resize(&mut self) -> Option<PromptSize> {
        self.resize.take().and(self.size)
    }

    pub fn fit(&mut self, viewport: Size<Pixels>, cx: &mut App) {
        if let Some(size) = self.size {
            self.show(size.fitted(viewport), cx);
        }
    }

    fn show(&mut self, size: PromptSize, cx: &mut App) {
        if self.shown == Some(size) {
            return;
        }
        self.shown = Some(size);
        if let PromptInput::CommitMessage(editor) = &self.input {
            editor.body.update(cx, |body, cx| {
                body.set_height(size.body_height);
                cx.notify();
            });
        }
    }

    pub fn input(&self) -> Entity<TextArea> {
        match &self.input {
            PromptInput::Text(input) => input.clone(),
            PromptInput::CommitMessage(editor) => editor.summary.clone(),
        }
    }

    pub fn body_input(&self) -> Option<Entity<TextArea>> {
        match &self.input {
            PromptInput::Text(_) => None,
            PromptInput::CommitMessage(editor) => Some(editor.body.clone()),
        }
    }

    pub fn is_focused(&self, window: &Window, cx: &App) -> bool {
        self.input().read(cx).focus_handle(cx).is_focused(window)
            || self
                .body_input()
                .is_some_and(|input| input.read(cx).focus_handle(cx).is_focused(window))
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
        let width = self.shown.map_or(style.width, |size| size.width);
        let mut panel = overlay_card(t, width).child(overlay_header(
            glyph::PENCIL,
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
        let input = match &self.input {
            PromptInput::Text(input) => input.clone().into_any_element(),
            PromptInput::CommitMessage(editor) => editor.element(),
        };
        panel = panel.child(match style.input_id {
            Some(id) => div()
                .id(id)
                .debug_selector(|| id.to_owned())
                .child(input)
                .into_any_element(),
            None => input,
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
        let actions = overlay_actions(
            button(style.cancel_id, jayjay_i18n::common::cancel(), t, false)
                .debug_selector(|| style.cancel_id.to_owned())
                .on_click(cx.listener(move |view, _, _, cx| on_cancel(view, cx))),
            primary,
        );
        panel = panel.child(if slots.leading_actions.is_empty() {
            actions
        } else {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .children(slots.leading_actions)
                .child(div().flex_1())
                .child(actions)
        });
        overlay_layer()
            .child(panel.children(slots.corner))
            .into_any_element()
    }
}
