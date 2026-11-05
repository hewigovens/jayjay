use super::pending_edit::PendingEdit;
use super::view::ShortcutSettings;
use crate::app::key_caps::KeyCaps;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::primitives::key_cap;
use gpui::prelude::FluentBuilder as _;
use gpui::{Context, Div, FontWeight, InteractiveElement, ParentElement, Styled, div, px, rgb};
use jayjay_core::shortcuts::{EffectiveBindings, EffectiveEntry, ShortcutContext};
use jayjay_i18n::{common, shortcuts};

const CHORD_WIDTH: f32 = 170.;
const RESET_WIDTH: f32 = 20.;
const EDIT_SIZE: f32 = 20.;
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
            .flex_1()
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
                caps = caps.child(key_cap(symbol, t));
            }
        }
        let edit = (!command.is_locked() && !recording && pending.is_none()).then(|| {
            let primary = entry
                .chord
                .map(|chord| KeyCaps::from(chord).to_string())
                .unwrap_or_else(shortcuts::unbound);
            Self::icon_button(
                format!("shortcut-edit-{}", command.id()),
                if unbound { glyph::PLUS } else { glyph::PENCIL },
                format!("{}: {primary}", command.title()),
                t,
                cx,
                move |this, window, cx| this.record(command, window, cx),
            )
        });
        let (centered, trailing) = if unbound { (edit, None) } else { (None, edit) };
        let caps = caps.children(centered.map(|edit| {
            div()
                .w(px(EMPTY_WIDTH))
                .h(px(22.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .border_1()
                .border_color(rgb(t.border))
                .child(edit)
        }));
        let chord = div()
            .when(recording, |cell| {
                cell.track_focus(&self.recorder_focus)
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| this.cancel(cx)))
            })
            .flex_none()
            .w(px(CHORD_WIDTH))
            .h(px(28.))
            .px(px(2.))
            .flex()
            .items_center()
            .rounded_md()
            .border_1()
            .border_color(border.map_or(gpui::transparent_black(), |color| rgb(color).into()))
            .text_size(ui_font_size(12.))
            .when(command.is_locked(), |cell| cell.opacity(0.6))
            .when(trailing.is_some(), |cell| {
                cell.child(div().flex_none().w(px(EDIT_SIZE)))
            })
            .child(caps)
            .children(trailing);
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
        let mut actions = div().flex_none().w(px(CHORD_WIDTH)).flex().gap(px(6.));
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
