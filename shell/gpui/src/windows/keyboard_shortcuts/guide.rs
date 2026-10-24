use gpui::{Action, App};

use crate::app::actions::{
    CloseWindow, CopyDiffSelection, DiffEditCollapseAll, DiffEditExpandAll, Dismiss,
    FilterByRevset, MergeNextHunk, MergePreviousHunk, MergeUseLeftHunk, MergeUseRightHunk,
    OpenBookmarkManager, OpenCommandPalette, OpenFind, OpenKeyboardShortcuts, OpenOperationLog,
    OpenOverview, OpenRepository, OpenSettings, Refresh, ResetZoom, SaveFileEditor,
    SaveNoteComposer, ShowRepoInFileManager, ToggleSidebar, ZoomIn, ZoomOut,
};
use crate::app::key_caps::KeyCaps;

enum Keys {
    Bound(&'static dyn Action),
    Fixed(&'static [&'static str]),
}

pub(super) struct ShortcutEntry {
    pub(super) label: fn() -> String,
    keys: Keys,
}

impl ShortcutEntry {
    const fn bound(label: fn() -> String, action: &'static dyn Action) -> Self {
        Self {
            label,
            keys: Keys::Bound(action),
        }
    }

    const fn fixed(label: fn() -> String, keys: &'static [&'static str]) -> Self {
        Self {
            label,
            keys: Keys::Fixed(keys),
        }
    }

    pub(super) fn key_caps(&self, cx: &App) -> Vec<String> {
        match self.keys {
            Keys::Bound(action) => KeyCaps::for_action(action, cx)
                .map(|caps| caps.caps().to_vec())
                .unwrap_or_default(),
            Keys::Fixed(keys) => keys.iter().map(|key| (*key).to_owned()).collect(),
        }
    }
}

pub(super) struct ShortcutSection {
    pub(super) id: &'static str,
    pub(super) title: fn() -> String,
    pub(super) entries: &'static [ShortcutEntry],
}

pub(super) const SECTIONS: &[ShortcutSection] = &[
    ShortcutSection {
        id: "general",
        title: jayjay_i18n::shortcuts::general,
        entries: &[
            ShortcutEntry::bound(jayjay_i18n::repo::open_repository, &OpenRepository),
            ShortcutEntry::bound(jayjay_i18n::menu::command_palette, &OpenCommandPalette),
            ShortcutEntry::bound(jayjay_i18n::repo::refresh, &Refresh),
            ShortcutEntry::bound(
                jayjay_i18n::menu::keyboard_shortcuts,
                &OpenKeyboardShortcuts,
            ),
            ShortcutEntry::bound(jayjay_i18n::repo::settings, &OpenSettings),
            ShortcutEntry::bound(jayjay_i18n::shortcuts::close_window, &CloseWindow),
        ],
    },
    ShortcutSection {
        id: "view",
        title: jayjay_i18n::menu::view,
        entries: &[
            ShortcutEntry::bound(jayjay_i18n::palette::hide_show_sidebar, &ToggleSidebar),
            ShortcutEntry::bound(jayjay_i18n::shortcuts::filter_by_revset, &FilterByRevset),
            ShortcutEntry::bound(jayjay_i18n::menu::zoom_in, &ZoomIn),
            ShortcutEntry::bound(jayjay_i18n::menu::zoom_out, &ZoomOut),
            ShortcutEntry::bound(jayjay_i18n::menu::reset_zoom, &ResetZoom),
        ],
    },
    ShortcutSection {
        id: "navigation",
        title: jayjay_i18n::shortcuts::navigation,
        entries: &[
            ShortcutEntry::fixed(jayjay_i18n::shortcuts::next_previous_item, &["J", "K"]),
            ShortcutEntry::fixed(jayjay_i18n::shortcuts::move_up_down, &["↑", "↓"]),
            ShortcutEntry::fixed(
                jayjay_i18n::shortcuts::alternate_up_down,
                &["Ctrl", "P", "/", "N"],
            ),
            ShortcutEntry::fixed(jayjay_i18n::shortcuts::switch_pane, &["Tab"]),
        ],
    },
    ShortcutSection {
        id: "repository",
        title: jayjay_i18n::menu::repository,
        entries: &[
            ShortcutEntry::bound(jayjay_i18n::menu::bookmark_manager, &OpenBookmarkManager),
            ShortcutEntry::bound(jayjay_i18n::menu::repo_overview, &OpenOverview),
            ShortcutEntry::bound(jayjay_i18n::shortcuts::filter_overview_lanes, &OpenFind),
            ShortcutEntry::bound(jayjay_i18n::menu::undo_last_operation, &OpenOperationLog),
            ShortcutEntry::bound(
                jayjay_i18n::palette::show_in_file_manager,
                &ShowRepoInFileManager,
            ),
        ],
    },
    ShortcutSection {
        id: "diff-review",
        title: jayjay_i18n::shortcuts::diff_review,
        entries: &[
            ShortcutEntry::bound(jayjay_i18n::palette::find_in_diff, &OpenFind),
            ShortcutEntry::bound(
                jayjay_i18n::shortcuts::copy_diff_selection,
                &CopyDiffSelection,
            ),
            ShortcutEntry::fixed(jayjay_i18n::palette::mark_file_reviewed, &["Space"]),
            ShortcutEntry::bound(jayjay_i18n::shortcuts::save_review_note, &SaveNoteComposer),
            ShortcutEntry::bound(jayjay_i18n::shortcuts::save_edited_file, &SaveFileEditor),
            ShortcutEntry::bound(jayjay_i18n::shortcuts::expand_all_files, &DiffEditExpandAll),
            ShortcutEntry::bound(
                jayjay_i18n::shortcuts::collapse_all_files,
                &DiffEditCollapseAll,
            ),
            ShortcutEntry::fixed(jayjay_i18n::shortcuts::collapse_expand_file, &["←", "→"]),
            ShortcutEntry::fixed(jayjay_i18n::merge::toggle_file, &["Enter"]),
        ],
    },
    ShortcutSection {
        id: "conflicts",
        title: jayjay_i18n::repo::preset_conflicts,
        entries: &[
            ShortcutEntry::bound(jayjay_i18n::merge::accept_left, &MergeUseLeftHunk),
            ShortcutEntry::bound(jayjay_i18n::merge::accept_right, &MergeUseRightHunk),
            ShortcutEntry::bound(
                jayjay_i18n::shortcuts::previous_conflict,
                &MergePreviousHunk,
            ),
            ShortcutEntry::bound(jayjay_i18n::shortcuts::next_conflict, &MergeNextHunk),
        ],
    },
    ShortcutSection {
        id: "drag-drop",
        title: jayjay_i18n::shortcuts::drag_drop,
        entries: &[ShortcutEntry::bound(
            jayjay_i18n::shortcuts::cancel_drag,
            &Dismiss,
        )],
    },
];

pub(super) fn columns() -> [&'static [ShortcutSection]; 2] {
    let target = SECTIONS
        .iter()
        .map(|section| section.entries.len())
        .sum::<usize>()
        / 2;
    let mut left_entries = 0;
    let split = SECTIONS
        .iter()
        .position(|section| {
            if left_entries >= target {
                true
            } else {
                left_entries += section.entries.len();
                false
            }
        })
        .unwrap_or(SECTIONS.len());
    let (left, right) = SECTIONS.split_at(split);
    [left, right]
}

#[cfg(test)]
mod tests {
    use super::{Keys, SECTIONS};
    use crate::app::actions::{Dismiss, Quit, SubmitStackedPr, app_key_bindings};

    #[test]
    fn guide_documents_every_app_binding() {
        let documented: Vec<_> = SECTIONS
            .iter()
            .flat_map(|section| section.entries)
            .filter_map(|entry| match entry.keys {
                Keys::Bound(action) => Some(action),
                Keys::Fixed(_) => None,
            })
            .chain([&Quit as &dyn gpui::Action, &Dismiss, &SubmitStackedPr])
            .collect();
        let missing: Vec<_> = app_key_bindings()
            .iter()
            .map(|binding| binding.action())
            .filter(|action| !action.name().starts_with("text_area::"))
            .filter(|action| !documented.iter().any(|known| known.partial_eq(*action)))
            .map(|action| action.name())
            .collect();
        assert!(
            missing.is_empty(),
            "bound but missing from the guide: {missing:?}"
        );
    }
}
