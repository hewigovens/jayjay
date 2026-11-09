use jayjay_i18n::menu;

use crate::shortcuts::ShortcutCommand;
use crate::types::ChangeInfo;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChangeCommand {
    NewChange,
    NewChangeBefore,
    NewChangeAfter,
    Edit,
    Describe,
    SquashIntoParent,
    MoveChangesToWorkingCopy,
    RebaseOntoTrunk,
    Fix,
    CreateBookmark,
    CreateTag,
    ShowEvolution,
    Duplicate,
    Absorb,
    Revert,
    Abandon,
}

impl ChangeCommand {
    pub const ALL: [Self; 16] = [
        Self::NewChange,
        Self::NewChangeBefore,
        Self::NewChangeAfter,
        Self::Edit,
        Self::Describe,
        Self::SquashIntoParent,
        Self::MoveChangesToWorkingCopy,
        Self::RebaseOntoTrunk,
        Self::Fix,
        Self::CreateBookmark,
        Self::CreateTag,
        Self::ShowEvolution,
        Self::Duplicate,
        Self::Absorb,
        Self::Revert,
        Self::Abandon,
    ];

    pub fn available(change: &ChangeInfo) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|command| command.is_available(change))
            .collect()
    }

    pub fn is_available(self, change: &ChangeInfo) -> bool {
        let mutable = !change.is_immutable;
        match self {
            Self::NewChange => change.new_change.on_top,
            Self::NewChangeBefore => change.new_change.before,
            Self::NewChangeAfter => change.new_change.after,
            Self::Edit | Self::RebaseOntoTrunk | Self::Fix | Self::Absorb | Self::Abandon => {
                mutable
            }
            // The working copy is described in the commit box, and moving its changes onto itself does nothing.
            Self::Describe | Self::MoveChangesToWorkingCopy => mutable && !change.is_working_copy,
            Self::SquashIntoParent => {
                mutable && !change.parents.is_empty() && !change.has_immutable_parent
            }
            Self::CreateBookmark
            | Self::CreateTag
            | Self::ShowEvolution
            | Self::Duplicate
            | Self::Revert => true,
        }
    }

    pub fn title(self) -> String {
        match self {
            Self::NewChange => menu::new_change_on_top(),
            Self::NewChangeBefore => menu::new_change_before(),
            Self::NewChangeAfter => menu::new_change_after(),
            Self::Edit => menu::edit_change(),
            Self::Describe => menu::describe_ellipsis(),
            Self::SquashIntoParent => menu::squash_into_parent(),
            Self::MoveChangesToWorkingCopy => menu::move_changes_to_working_copy(),
            Self::RebaseOntoTrunk => menu::rebase_onto_trunk(),
            Self::Fix => menu::run_formatters_jj_fix(),
            Self::CreateBookmark => menu::create_bookmark_here_ellipsis(),
            Self::CreateTag => menu::create_tag_here_ellipsis(),
            Self::ShowEvolution => menu::show_evolution_ellipsis(),
            Self::Duplicate => menu::duplicate(),
            Self::Absorb => menu::absorb_into_ancestors(),
            Self::Revert => menu::revert_change(),
            Self::Abandon => menu::abandon(),
        }
    }

    /// The unmodified key that picks this command while its context menu is open.
    pub fn accelerator(self) -> Option<char> {
        match self {
            Self::NewChange => Some('n'),
            Self::Edit => Some('e'),
            Self::Describe => Some('d'),
            Self::SquashIntoParent => Some('s'),
            Self::MoveChangesToWorkingCopy => Some('m'),
            Self::RebaseOntoTrunk => Some('r'),
            Self::CreateBookmark => Some('b'),
            Self::CreateTag => Some('t'),
            Self::ShowEvolution => Some('v'),
            Self::Abandon => Some('a'),
            _ => None,
        }
    }

    pub fn shortcut(self) -> Option<ShortcutCommand> {
        match self {
            Self::NewChange => Some(ShortcutCommand::RepoNewChange),
            Self::Edit => Some(ShortcutCommand::RepoEditChange),
            Self::Describe => Some(ShortcutCommand::RepoDescribeChange),
            Self::SquashIntoParent => Some(ShortcutCommand::RepoSquashChange),
            Self::RebaseOntoTrunk => Some(ShortcutCommand::RepoRebaseOntoTrunk),
            Self::CreateBookmark => Some(ShortcutCommand::RepoNewBookmark),
            Self::ShowEvolution => Some(ShortcutCommand::RepoShowEvolution),
            Self::Abandon => Some(ShortcutCommand::ChangesAbandon),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChangeCommand;
    use crate::mock::change_info;
    use crate::types::ChangeInfo;

    fn change(edit: impl FnOnce(&mut ChangeInfo)) -> ChangeInfo {
        let mut change = change_info("change", "commit");
        change.parents = vec!["parent".to_owned()];
        edit(&mut change);
        change
    }

    #[test]
    fn an_immutable_change_offers_only_commands_that_leave_it_alone() {
        let immutable = change(|change| {
            change.is_immutable = true;
            change.new_change.before = false;
        });
        assert_eq!(
            ChangeCommand::available(&immutable),
            [
                ChangeCommand::NewChange,
                ChangeCommand::NewChangeAfter,
                ChangeCommand::CreateBookmark,
                ChangeCommand::CreateTag,
                ChangeCommand::ShowEvolution,
                ChangeCommand::Duplicate,
                ChangeCommand::Revert,
            ]
        );
    }

    #[test]
    fn the_working_copy_is_neither_described_nor_moved_onto_itself() {
        let working_copy = change(|change| change.is_working_copy = true);
        assert!(!ChangeCommand::Describe.is_available(&working_copy));
        assert!(!ChangeCommand::MoveChangesToWorkingCopy.is_available(&working_copy));
        assert!(ChangeCommand::Edit.is_available(&working_copy));
        assert!(ChangeCommand::SquashIntoParent.is_available(&working_copy));
    }

    #[test]
    fn squash_needs_a_mutable_parent() {
        assert!(ChangeCommand::SquashIntoParent.is_available(&change(|_| {})));
        assert!(
            !ChangeCommand::SquashIntoParent
                .is_available(&change(|change| change.has_immutable_parent = true))
        );
        assert!(
            !ChangeCommand::SquashIntoParent.is_available(&change(|change| change.parents.clear()))
        );
    }
}
