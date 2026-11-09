use jayjay_i18n::menu;

use super::SelectionState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectionCommand {
    Merge,
    Squash,
    Parallelize,
    Fix,
    RebaseOntoTrunk,
    Abandon,
}

impl SelectionCommand {
    pub const ALL: [Self; 6] = [
        Self::Merge,
        Self::Squash,
        Self::Parallelize,
        Self::Fix,
        Self::RebaseOntoTrunk,
        Self::Abandon,
    ];

    pub(super) fn is_available(self, state: &SelectionState) -> bool {
        match self {
            Self::Merge => state.can_merge,
            Self::Squash => state.can_squash,
            Self::Parallelize => state.can_parallelize,
            Self::Fix => state.can_fix,
            Self::RebaseOntoTrunk | Self::Abandon => state.can_abandon,
        }
    }

    pub fn title(self, count: usize) -> String {
        match self {
            Self::Merge => menu::merge_selected(count),
            Self::Squash => menu::squash_selected(count),
            Self::Parallelize => menu::parallelize_selected(count),
            Self::Fix => menu::fix_selected(count),
            Self::RebaseOntoTrunk => menu::rebase_selected_onto_trunk(count),
            Self::Abandon => menu::abandon_selected(count),
        }
    }

    pub fn accelerator(self) -> Option<char> {
        (self == Self::Abandon).then_some('a')
    }

    pub fn help(self, available: bool) -> String {
        match self {
            Self::Merge if available => menu::choose_the_first_parent_for_the(),
            Self::Merge => menu::merge_requires_independent_heads(),
            Self::Squash if available => menu::combine_the_selected_range_into_its(),
            Self::Squash => menu::squash_requires_a_consecutive_linear_range(),
            Self::Parallelize if available => {
                menu::make_the_selected_changes_independent_siblings()
            }
            Self::Parallelize => menu::parallelize_requires_two_or_more_mutable(),
            Self::Fix if available => menu::rewrite_these_changes_and_their_descendants(),
            Self::Fix => menu::immutable_changes_cannot_be_rewritten(),
            Self::RebaseOntoTrunk if available => {
                menu::rebase_the_selected_changes_while_preserving()
            }
            Self::RebaseOntoTrunk => menu::immutable_changes_cannot_be_rewritten(),
            Self::Abandon if available => menu::abandon_the_selected_changes(),
            Self::Abandon => menu::immutable_changes_cannot_be_abandoned(),
        }
    }
}
