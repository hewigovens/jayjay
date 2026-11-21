use std::ops::Range;

use gpui::{
    AnyElement, App, IntoElement, Styled, UniformListScrollHandle, Window, px, uniform_list,
};

use crate::diff::{bounds_capture, column_bounds_capture};
use crate::repo::window::PanelBoundsSlot;
use crate::ui::clipped_rows::clipped_rows;
use crate::ui::primitives::no_scrollbar_gutter;

/// Where a diff's rows scroll: in their own list, or as part of a stack of files that scrolls as one.
#[derive(Clone)]
pub(crate) enum RowPlacement {
    Scrolling(UniformListScrollHandle),
    /// `active` marks the selected file, the only one a text selection belongs to.
    Stacked {
        active: bool,
    },
}

impl RowPlacement {
    pub(super) fn shows_selection(&self) -> bool {
        match self {
            Self::Scrolling(_) => true,
            Self::Stacked { active } => *active,
        }
    }

    pub(super) fn scroll(&self) -> Option<&UniformListScrollHandle> {
        match self {
            Self::Scrolling(scroll) => Some(scroll),
            Self::Stacked { .. } => None,
        }
    }

    pub(super) fn bounds_capture(&self, slot: PanelBoundsSlot) -> AnyElement {
        match self {
            Self::Scrolling(_) => bounds_capture(slot).into_any_element(),
            Self::Stacked { .. } => column_bounds_capture(slot).into_any_element(),
        }
    }

    pub(super) fn rows(
        &self,
        id: &'static str,
        count: usize,
        row_height: f32,
        render: impl Fn(Range<usize>, &mut Window, &mut App) -> Vec<AnyElement> + 'static,
    ) -> AnyElement {
        match self {
            Self::Scrolling(scroll) => {
                no_scrollbar_gutter(uniform_list(id, count, render).track_scroll(scroll))
                    .h_full()
                    .into_any_element()
            }
            Self::Stacked { .. } => clipped_rows(count, px(row_height), render).into_any_element(),
        }
    }
}
