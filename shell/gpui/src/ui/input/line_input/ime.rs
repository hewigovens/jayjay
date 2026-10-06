use std::ops::Range;

use gpui::{
    App, Bounds, Context, Entity, FocusHandle, InputHandler, IntoElement, Pixels, Point, Styled,
    UTF16Selection, Window, canvas,
};
use jayjay_core::utf16::{byte_offset, utf16_offset};

use super::{LineInput, LineInputSelector};
use crate::ui::input::sanitize_single_line;

pub type LineInputEdited<T> = fn(&mut T, &mut Context<T>);

struct LineInputHandler<T: 'static> {
    owner: Entity<T>,
    select: LineInputSelector<T>,
    on_edit: LineInputEdited<T>,
    bounds: Bounds<Pixels>,
}

impl LineInput {
    /// An invisible layer that registers this input with the platform while `focus` is focused, so IME text reaches it.
    pub(crate) fn ime_layer<T: 'static>(
        owner: Entity<T>,
        focus: Option<FocusHandle>,
        select: LineInputSelector<T>,
        on_edit: LineInputEdited<T>,
    ) -> impl IntoElement {
        canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                if let Some(focus) = focus {
                    let handler = LineInputHandler {
                        owner,
                        select,
                        on_edit,
                        bounds,
                    };
                    window.handle_input(&focus, handler, cx);
                }
            },
        )
        .absolute()
        .size_full()
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        let text = self.text();
        utf16_offset(text, range.start)..utf16_offset(text, range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        let text = self.text();
        byte_offset(text, range_utf16.start)..byte_offset(text, range_utf16.end)
    }

    fn replacement_range(&self, range_utf16: Option<&Range<usize>>) -> Range<usize> {
        range_utf16
            .map(|range| self.range_from_utf16(range))
            .or(self.marked.clone())
            .unwrap_or_else(|| self.edit.selection_range())
    }
}

impl<T: 'static> LineInputHandler<T> {
    fn read<R>(&self, cx: &mut App, read: impl FnOnce(&LineInput) -> R) -> Option<R> {
        let select = self.select;
        self.owner
            .update(cx, |owner, _| select(owner).map(|input| read(input)))
    }

    fn edit(&self, cx: &mut App, edit: impl FnOnce(&mut LineInput)) {
        let (select, on_edit) = (self.select, self.on_edit);
        self.owner.update(cx, |owner, cx| {
            let Some(input) = select(owner) else {
                return;
            };
            edit(input);
            on_edit(owner, cx);
            LineInput::show_for_owner(owner, cx, select);
            cx.notify();
        });
    }
}

impl<T: 'static> InputHandler for LineInputHandler<T> {
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        cx: &mut App,
    ) -> Option<UTF16Selection> {
        self.read(cx, |input| UTF16Selection {
            range: input.range_to_utf16(&input.edit.selection_range()),
            reversed: input.edit.selection_reversed(),
        })
    }

    fn marked_text_range(&mut self, _: &mut Window, cx: &mut App) -> Option<Range<usize>> {
        self.read(cx, |input| {
            input
                .marked
                .as_ref()
                .map(|range| input.range_to_utf16(range))
        })
        .flatten()
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _: &mut Window,
        cx: &mut App,
    ) -> Option<String> {
        self.read(cx, |input| {
            let range = input.range_from_utf16(&range_utf16);
            adjusted_range.replace(input.range_to_utf16(&range));
            input.text()[range].to_owned()
        })
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut App,
    ) {
        let Some(text) = field_text(text) else {
            return;
        };
        self.edit(cx, |input| {
            let range = input.replacement_range(range_utf16.as_ref());
            input.replace_range(range, &text);
        });
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut App,
    ) {
        let Some(new_text) = field_text(new_text) else {
            return;
        };
        self.edit(cx, |input| {
            let range = input.replacement_range(range_utf16.as_ref());
            let start = range.start;
            input.replace_range(range, &new_text);
            input.marked = (!new_text.is_empty()).then(|| start..start + new_text.len());
            // `new_selected_range_utf16` is relative to the marked text, not the whole field.
            if let Some(selected) = new_selected_range_utf16 {
                let selected =
                    byte_offset(&new_text, selected.start)..byte_offset(&new_text, selected.end);
                input
                    .edit
                    .select_range(start + selected.start..start + selected.end);
            }
        });
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut App) {
        let select = self.select;
        self.owner.update(cx, |owner, _| {
            if let Some(input) = select(owner) {
                input.marked = None;
            }
        });
    }

    // The field does not keep its glyph layout, so the IME candidate window anchors to the field itself.
    fn bounds_for_range(
        &mut self,
        _: Range<usize>,
        _: &mut Window,
        _: &mut App,
    ) -> Option<Bounds<Pixels>> {
        Some(self.bounds)
    }

    fn character_index_for_point(
        &mut self,
        _: Point<Pixels>,
        _: &mut Window,
        _: &mut App,
    ) -> Option<usize> {
        None
    }

    fn prefers_ime_for_printable_keys(&mut self, _: &mut Window, _: &mut App) -> bool {
        true
    }
}

/// Unhandled keys such as Tab also arrive as text: control-only input is no edit, while an empty string still clears a composition.
fn field_text(text: &str) -> Option<String> {
    let kept: String = sanitize_single_line(text)
        .chars()
        .filter(|ch| !ch.is_control())
        .collect();
    (kept.is_empty() == text.is_empty()).then_some(kept)
}

#[cfg(test)]
mod tests {
    use gpui::{Render, TestAppContext, VisualTestContext, div};

    use super::*;

    struct Owner {
        input: LineInput,
        edits: usize,
    }

    impl Render for Owner {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    fn handler(owner: Entity<Owner>) -> LineInputHandler<Owner> {
        LineInputHandler {
            owner,
            select: |owner| Some(&mut owner.input),
            on_edit: |owner, _| owner.edits += 1,
            bounds: Bounds::default(),
        }
    }

    #[gpui::test]
    fn composition_replaces_the_marked_run_and_control_text_is_ignored(cx: &mut TestAppContext) {
        let (owner, cx) = cx.add_window_view(|_, _| Owner {
            input: LineInput::new("é "),
            edits: 0,
        });
        let cx: &mut VisualTestContext = cx;
        let mut handler = handler(owner.clone());

        cx.update(|window, cx| {
            handler.replace_and_mark_text_in_range(None, "ni", Some(1..1), window, cx);
            assert_eq!(handler.marked_text_range(window, cx), Some(2..4));
            let selected = handler.selected_text_range(false, window, cx).unwrap();
            assert_eq!(selected.range, 3..3);

            handler.replace_text_in_range(None, "你", window, cx);
            assert_eq!(handler.marked_text_range(window, cx), None);
        });

        cx.update(|window, cx| {
            owner.update(cx, |owner, _| owner.input.edit.select_range(0..usize::MAX));
            handler.replace_text_in_range(None, "\t", window, cx);
        });

        owner.read_with(cx, |owner, _| {
            assert_eq!(owner.input.text(), "é 你");
            assert_eq!(owner.input.edit.selection_range(), 0.."é 你".len());
            assert_eq!(owner.edits, 2);
        });
    }
}
