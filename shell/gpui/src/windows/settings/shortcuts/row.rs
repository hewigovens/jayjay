use super::pending_edit::PendingEdit;
use super::view::ShortcutSettings;
use crate::app::key_caps::KeyCaps;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::key_cap;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, div, px, rgb,
};
use jayjay_core::shortcuts::{EffectiveBindings, EffectiveEntry, ShortcutContext};
use jayjay_i18n::{common, shortcuts};
use std::rc::Rc;

const CHORD_WIDTH: f32 = 112.;
const RESET_WIDTH: f32 = 20.;
const EMPTY_WIDTH: f32 = 78.;

impl ShortcutSettings {
    pub(super) fn group_title(context: ShortcutContext, t: &Theme) -> Div {
        div()
            .mt(px(8.))
            .px(px(8.))
            .py(px(6.))
            .rounded_sm()
            .bg(rgb(t.row_alt_bg))
            .text_size(ui_font_size(11.))
            .font_weight(FontWeight::BOLD)
            .text_color(rgb(t.fg_dim))
            .child(context.title().to_uppercase())
    }

    pub(super) fn shows(&self, entry: &EffectiveEntry, query: &str, modified: bool) -> bool {
        entry.command.title().to_lowercase().contains(query) && (modified || !self.modified_only)
    }

    pub(super) fn row(
        &self,
        entry: &EffectiveEntry,
        modified: bool,
        bindings: &EffectiveBindings,
        t: &Theme,
        cx: &Context<Self>,
    ) -> Div {
        let command = entry.command;
        let recording = self.recording == Some(command);
        let pending = self.pending.filter(|edit| edit.command() == command);
        let platform = bindings.platform();
        let (chords, border) = if recording {
            let border = if self.error.is_some() {
                t.error_fg
            } else {
                t.selected_accent
            };
            let held = KeyCaps::from(&self.held).caps().to_vec();
            let chords = if held.is_empty() {
                Vec::new()
            } else {
                vec![held]
            };
            (chords, Some(border))
        } else if let Some(edit) = pending {
            let chords = edit.chord(bindings).map(|chord| chord.symbols(platform));
            (chords.into_iter().collect(), Some(t.warning_fg))
        } else {
            let chords = entry.chords().map(|chord| chord.symbols(platform));
            (chords.collect::<Vec<_>>(), None)
        };
        let unbound = chords.is_empty();
        let mut caps = div()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(3.));
        if unbound && recording {
            caps = caps
                .text_color(rgb(t.fg_faint))
                .child(shortcuts::record_prompt());
        }
        for (ix, chord) in chords.into_iter().enumerate() {
            if ix > 0 {
                caps = caps.child(div().px(px(2.)).text_color(rgb(t.fg_faint)).child("/"));
            }
            for symbol in chord {
                caps = caps.child(
                    key_cap(symbol, t)
                        .when(command.is_locked(), |cap| cap.text_color(rgb(t.fg_dim))),
                );
            }
        }
        let editable = !command.is_locked() && !recording && pending.is_none();
        if unbound && editable {
            caps = caps.child(
                div()
                    .w(px(EMPTY_WIDTH))
                    .h(px(22.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(t.border))
                    .child(icon(glyph::PLUS, 12., t.fg_dim)),
            );
        }
        if recording && let Some(color) = border {
            caps = caps
                .min_w(px(EMPTY_WIDTH))
                .h(px(22.))
                .px(px(3.))
                .rounded_md()
                .border_1()
                .border_color(rgb(color));
        }
        let border = border
            .filter(|_| !recording)
            .map_or(gpui::transparent_black(), |color| rgb(color).into());
        let inner = div()
            .when(recording, |cell| {
                cell.track_focus(&self.recorder_focus)
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| this.cancel(cx)))
            })
            .h_full()
            .px(px(3.))
            .flex()
            .items_center()
            .rounded_md()
            .when(!editable, |cell| cell.border_1().border_color(border))
            .child(caps);
        let chord = if editable {
            let primary = entry
                .chord
                .map(|chord| KeyCaps::from(chord).to_string())
                .unwrap_or_else(shortcuts::unbound);
            Self::control(
                format!("shortcut-edit-{}", command.id()).into(),
                format!("{}: {primary}", command.title()),
                t,
                cx,
                Rc::new(move |this, window, cx| this.record(command, window, cx)),
            )
            .h_full()
            .rounded_md()
            .border_color(gpui::transparent_black())
            .child(inner)
            .into_any_element()
        } else {
            inner.into_any_element()
        };
        let chord = div()
            .flex_none()
            .min_w(px(CHORD_WIDTH))
            .flex()
            .justify_end()
            .h(px(28.))
            .text_size(ui_font_size(12.))
            .child(chord);
        let reset = div().flex_none().w(px(RESET_WIDTH)).when(modified, |slot| {
            slot.child(Self::icon_button(
                format!("shortcut-reset-{}", command.id()),
                glyph::ARROW_UTURN_BACK,
                shortcuts::reset(),
                t,
                cx,
                move |this, _, cx| this.apply(PendingEdit::Reset(command), false, cx),
            ))
        });
        let mut row = div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .px(px(8.))
            .py(px(5.))
            .border_b_1()
            .border_color(rgb(t.border))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .whitespace_normal()
                            .when(command.is_locked(), |title| title.text_color(rgb(t.fg_dim)))
                            .child(command.title()),
                    )
                    .child(reset)
                    .child(chord),
            );
        if (recording || pending.is_some())
            && let Some(error) = self.error.clone()
        {
            row = row.child(self.note(error, pending.is_some(), t, cx));
        }
        row
    }

    fn note(&self, message: String, pending: bool, t: &Theme, cx: &Context<Self>) -> Div {
        let color = if pending { t.warning_fg } else { t.error_fg };
        let mut actions = div().flex_none().min_w(px(CHORD_WIDTH)).flex().gap(px(6.));
        if pending && self.replace_allowed {
            actions = actions.child(
                Self::button(
                    "shortcuts-replace",
                    shortcuts::replace(),
                    t,
                    cx,
                    |this, _, cx| {
                        if let Some(edit) = this.pending {
                            this.apply(edit, true, cx);
                        }
                    },
                )
                .flex_1()
                .flex()
                .justify_center(),
            );
        }
        if pending {
            actions = actions.child(
                Self::button(
                    "shortcuts-cancel",
                    common::cancel(),
                    t,
                    cx,
                    |this, _, cx| this.cancel(cx),
                )
                .flex_1()
                .flex()
                .justify_center(),
            );
        }
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .text_size(ui_font_size(11.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .whitespace_normal()
                    .text_color(rgb(color))
                    .child(message),
            )
            .when(pending, |note| note.child(actions))
    }
}
