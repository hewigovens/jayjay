use gpui::{Action, App};

use crate::app::actions::{
    CloseWindow, CopyDiffSelection, DiffEditCollapseAll, DiffEditExpandAll, FilterByRevset,
    MergeNextHunk, MergePreviousHunk, MergeUseLeftHunk, MergeUseRightHunk, OpenBookmarkManager,
    OpenCommandPalette, OpenFind, OpenKeyboardShortcuts, OpenOperationLog, OpenOverview,
    OpenRepository, OpenSettings, Refresh, ResetZoom, SaveFileEditor, SaveNoteComposer,
    ShowRepoInFileManager, ToggleSidebar, ZoomIn, ZoomOut,
};
use crate::app::key_caps::KeyCaps;

enum Keys {
    Bound(&'static dyn Action),
    Fixed(&'static [&'static str]),
}

pub(super) struct ShortcutEntry {
    pub(super) label: &'static str,
    keys: Keys,
}

impl ShortcutEntry {
    const fn bound(label: &'static str, action: &'static dyn Action) -> Self {
        Self {
            label,
            keys: Keys::Bound(action),
        }
    }

    const fn fixed(label: &'static str, keys: &'static [&'static str]) -> Self {
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
    pub(super) title: &'static str,
    pub(super) entries: &'static [ShortcutEntry],
}

pub(super) const SECTIONS: &[ShortcutSection] = &[
    ShortcutSection {
        title: "General",
        entries: &[
            ShortcutEntry::bound("Open Repository", &OpenRepository),
            ShortcutEntry::bound("Command Palette", &OpenCommandPalette),
            ShortcutEntry::bound("Refresh", &Refresh),
            ShortcutEntry::bound("Keyboard Shortcuts", &OpenKeyboardShortcuts),
            ShortcutEntry::bound("Settings", &OpenSettings),
            ShortcutEntry::bound("Close Window", &CloseWindow),
        ],
    },
    ShortcutSection {
        title: "View",
        entries: &[
            ShortcutEntry::bound("Hide / Show Sidebar", &ToggleSidebar),
            ShortcutEntry::bound("Filter by Revset", &FilterByRevset),
            ShortcutEntry::bound("Zoom In", &ZoomIn),
            ShortcutEntry::bound("Zoom Out", &ZoomOut),
            ShortcutEntry::bound("Reset Zoom", &ResetZoom),
        ],
    },
    ShortcutSection {
        title: "Navigation",
        entries: &[
            ShortcutEntry::fixed("Next / Previous Item", &["J", "K"]),
            ShortcutEntry::fixed("Move Up / Down", &["↑", "↓"]),
            ShortcutEntry::fixed("Alternate Up / Down", &["Ctrl", "P", "/", "N"]),
            ShortcutEntry::fixed("Switch Pane", &["Tab"]),
        ],
    },
    ShortcutSection {
        title: "Repository",
        entries: &[
            ShortcutEntry::bound("Bookmark Manager", &OpenBookmarkManager),
            ShortcutEntry::bound("Repo Overview", &OpenOverview),
            ShortcutEntry::bound("Undo Last Operation", &OpenOperationLog),
            ShortcutEntry::bound("Show in File Manager", &ShowRepoInFileManager),
        ],
    },
    ShortcutSection {
        title: "Diff & Review",
        entries: &[
            ShortcutEntry::bound("Find in Diff", &OpenFind),
            ShortcutEntry::bound("Copy Diff Selection", &CopyDiffSelection),
            ShortcutEntry::fixed("Mark File Reviewed", &["Space"]),
            ShortcutEntry::bound("Save Review Note", &SaveNoteComposer),
            ShortcutEntry::bound("Save Edited File", &SaveFileEditor),
            ShortcutEntry::bound("Expand All Files", &DiffEditExpandAll),
            ShortcutEntry::bound("Collapse All Files", &DiffEditCollapseAll),
            ShortcutEntry::fixed("Collapse / Expand File", &["←", "→"]),
            ShortcutEntry::fixed("Toggle File", &["Enter"]),
        ],
    },
    ShortcutSection {
        title: "Conflicts",
        entries: &[
            ShortcutEntry::bound("Accept Left", &MergeUseLeftHunk),
            ShortcutEntry::bound("Accept Right", &MergeUseRightHunk),
            ShortcutEntry::bound("Previous Conflict", &MergePreviousHunk),
            ShortcutEntry::bound("Next Conflict", &MergeNextHunk),
        ],
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
