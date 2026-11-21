use gpui::{AnyElement, App, Context, IntoElement, ParentElement, Styled, div, px};

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::{
    DiffViewMode, edit_diff_button, mode_label, show_all_files_button, view_mode_button,
};
use crate::repo::RepoWindow;
use crate::repo::window::FocusStop;

/// Layout toggle and Edit Diff apply to the whole change, so they sit in the change header rather than a file's.
#[derive(Clone, Copy)]
pub(crate) struct DiffActions {
    /// `None` while no file diff is shown.
    pub(crate) view_mode: Option<DiffViewMode>,
    pub(crate) can_edit_diff: bool,
    /// A file is shown alone although every file's diff is asked for.
    pub(crate) shows_file_alone: bool,
}

impl DiffActions {
    fn is_empty(self) -> bool {
        self.view_mode.is_none() && !self.can_edit_diff && !self.shows_file_alone
    }

    pub(super) fn render(
        self,
        compact: bool,
        focused: Option<FocusStop>,
        t: &Theme,
        cx: &mut Context<RepoWindow>,
    ) -> Option<AnyElement> {
        if self.is_empty() {
            return None;
        }
        let mut actions = div()
            .flex()
            .flex_none()
            .flex_row()
            .items_center()
            .gap(px(2.));
        if self.shows_file_alone {
            actions = actions.child(show_all_files_button(compact, t, cx));
        }
        if let Some(mode) = self.view_mode {
            actions = actions.child(view_mode_button(
                mode,
                compact,
                focused == Some(FocusStop::DiffLayout),
                t,
                cx,
            ));
        }
        if self.can_edit_diff {
            actions = actions.child(edit_diff_button(
                compact,
                focused == Some(FocusStop::EditDiff),
                t,
                cx,
            ));
        }
        Some(actions.into_any_element())
    }

    /// Mirrors the header action's padding, icon, and gap, so the byline can budget for it.
    pub(super) fn width(self, compact: bool, t: &Theme, cx: &App) -> f32 {
        let action = |label: String| {
            let label = if compact {
                0.
            } else {
                4. + f32::from(fonts::ui_text_width(
                    cx,
                    &label,
                    px(t.scaled_font_size(11.)),
                ))
            };
            12. + 12. + label
        };
        let widths: Vec<f32> = self
            .shows_file_alone
            .then(jayjay_i18n::repo::show_all_files)
            .into_iter()
            .chain(self.view_mode.map(mode_label))
            .chain(self.can_edit_diff.then(jayjay_i18n::diff::edit_diff))
            .map(action)
            .collect();
        widths.iter().sum::<f32>() + 2. * widths.len().saturating_sub(1) as f32
    }
}
