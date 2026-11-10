use super::entry::ShortcutEntry;
use crate::app::theme::{Theme, ui_font_size};
use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div, px,
    rgb,
};
use jayjay_core::shortcuts::{EffectiveBindings, ShortcutCommand};

pub(super) struct ShortcutSection {
    id: &'static str,
    title: fn() -> String,
    entries: &'static [ShortcutEntry],
}

const SECTIONS: &[ShortcutSection] = &[
    ShortcutSection {
        id: "general",
        title: jayjay_i18n::shortcuts::general,
        entries: &[
            ShortcutEntry::Command(ShortcutCommand::AppOpenRepository),
            ShortcutEntry::Command(ShortcutCommand::RepoCommandPalette),
            ShortcutEntry::Command(ShortcutCommand::RepoRefresh),
            ShortcutEntry::Command(ShortcutCommand::AppKeyboardShortcuts),
            ShortcutEntry::Command(ShortcutCommand::AppSettings),
            ShortcutEntry::Command(ShortcutCommand::AppCloseWindow),
            ShortcutEntry::Command(ShortcutCommand::RepoUndo),
            ShortcutEntry::Command(ShortcutCommand::RepoRedo),
        ],
    },
    ShortcutSection {
        id: "view",
        title: jayjay_i18n::menu::view,
        entries: &[
            ShortcutEntry::Command(ShortcutCommand::RepoToggleSidebar),
            ShortcutEntry::Command(ShortcutCommand::RepoFilterByRevset),
            ShortcutEntry::Command(ShortcutCommand::AppZoomIn),
            ShortcutEntry::Command(ShortcutCommand::AppZoomOut),
            ShortcutEntry::Command(ShortcutCommand::AppResetZoom),
        ],
    },
    ShortcutSection {
        id: "navigation",
        title: jayjay_i18n::shortcuts::navigation,
        entries: &[
            ShortcutEntry::Command(ShortcutCommand::ChangesNext),
            ShortcutEntry::Command(ShortcutCommand::ChangesPrevious),
            ShortcutEntry::Command(ShortcutCommand::FilesNext),
            ShortcutEntry::Command(ShortcutCommand::FilesPrevious),
            ShortcutEntry::Command(ShortcutCommand::RepoSwitchPane),
            ShortcutEntry::Command(ShortcutCommand::RepoCyclePanes),
            ShortcutEntry::Command(ShortcutCommand::RepoOpenContextMenu),
        ],
    },
    ShortcutSection {
        id: "repository",
        title: jayjay_i18n::menu::repository,
        entries: &[
            ShortcutEntry::Command(ShortcutCommand::RepoBookmarkManager),
            ShortcutEntry::Command(ShortcutCommand::RepoOverview),
            ShortcutEntry::Command(ShortcutCommand::OverviewFilterLanes),
            ShortcutEntry::Command(ShortcutCommand::RepoOperationLog),
            ShortcutEntry::Command(ShortcutCommand::RepoShowInFileManager),
            ShortcutEntry::Command(ShortcutCommand::RepoFetch),
            ShortcutEntry::Command(ShortcutCommand::RepoPush),
            ShortcutEntry::Command(ShortcutCommand::RepoNewWorkspace),
            ShortcutEntry::Command(ShortcutCommand::RepoImportPullRequest),
            ShortcutEntry::Command(ShortcutCommand::RepoRebaseOntoTrunk),
            ShortcutEntry::Command(ShortcutCommand::RepoBisect),
            ShortcutEntry::Command(ShortcutCommand::RepoCleanUpStaleBookmarks),
            ShortcutEntry::Command(ShortcutCommand::RepoNewChange),
            ShortcutEntry::Command(ShortcutCommand::RepoEditChange),
            ShortcutEntry::Command(ShortcutCommand::RepoDescribeChange),
            ShortcutEntry::Command(ShortcutCommand::RepoSquashChange),
            ShortcutEntry::Command(ShortcutCommand::RepoNewBookmark),
            ShortcutEntry::Command(ShortcutCommand::RepoShowEvolution),
            ShortcutEntry::Command(ShortcutCommand::ChangesAbandon),
        ],
    },
    ShortcutSection {
        id: "diff-review",
        title: jayjay_i18n::shortcuts::diff_review,
        entries: &[
            ShortcutEntry::Command(ShortcutCommand::RepoFindInDiff),
            ShortcutEntry::Command(ShortcutCommand::RepoCopyDiffSelection),
            ShortcutEntry::Command(ShortcutCommand::FilesMarkReviewed),
            ShortcutEntry::Command(ShortcutCommand::TextSaveReviewNote),
            ShortcutEntry::Command(ShortcutCommand::TextSaveFile),
            ShortcutEntry::Command(ShortcutCommand::DiffEditorExpandAllFiles),
            ShortcutEntry::Command(ShortcutCommand::DiffEditorCollapseAllFiles),
            ShortcutEntry::Command(ShortcutCommand::DiffNextPreviousFileCard),
            ShortcutEntry::Command(ShortcutCommand::DiffCollapseExpandFileCard),
            ShortcutEntry::Command(ShortcutCommand::DiffToggleFileCard),
            ShortcutEntry::Command(ShortcutCommand::RepoToggleSideBySide),
            ShortcutEntry::Command(ShortcutCommand::RepoToggleIgnoreWhitespace),
            ShortcutEntry::Command(ShortcutCommand::RepoToggleTreeFileList),
        ],
    },
    ShortcutSection {
        id: "conflicts",
        title: jayjay_i18n::repo::preset_conflicts,
        entries: &[
            ShortcutEntry::Command(ShortcutCommand::MergeAcceptLeft),
            ShortcutEntry::Command(ShortcutCommand::MergeAcceptRight),
            ShortcutEntry::Command(ShortcutCommand::MergePreviousConflict),
            ShortcutEntry::Command(ShortcutCommand::MergeNextConflict),
        ],
    },
    ShortcutSection {
        id: "drag-drop",
        title: jayjay_i18n::shortcuts::drag_drop,
        entries: &[ShortcutEntry::Dismiss],
    },
];

impl ShortcutSection {
    pub(super) fn visible_count(&self, bindings: &EffectiveBindings) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.is_visible(bindings))
            .count()
    }

    pub(super) fn columns(bindings: &EffectiveBindings) -> [&'static [ShortcutSection]; 2] {
        let target = SECTIONS
            .iter()
            .map(|section| section.visible_count(bindings))
            .sum::<usize>()
            / 2;
        let mut count = 0;
        let split = SECTIONS
            .iter()
            .position(|section| {
                if count >= target {
                    true
                } else {
                    count += section.visible_count(bindings);
                    false
                }
            })
            .unwrap_or(SECTIONS.len());
        let (left, right) = SECTIONS.split_at(split);
        [left, right]
    }

    pub(super) fn column(
        sections: &[ShortcutSection],
        bindings: &EffectiveBindings,
        t: &Theme,
        cx: &App,
    ) -> AnyElement {
        let mut column = div().flex().flex_col().gap(px(20.)).flex_1().min_w_0();
        for section in sections
            .iter()
            .filter(|section| section.visible_count(bindings) > 0)
        {
            column = column.child(section.render(bindings, t, cx));
        }
        column.into_any_element()
    }

    fn render(&self, bindings: &EffectiveBindings, t: &Theme, cx: &App) -> AnyElement {
        let id = self.id;
        let mut block = div()
            .id(SharedString::from(format!("shortcut-section-{id}")))
            .debug_selector(move || format!("shortcut-section-{id}"))
            .flex()
            .flex_col()
            .gap(px(9.))
            .child(
                div()
                    .text_size(ui_font_size(12.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(t.fg_dim))
                    .child((self.title)().to_uppercase()),
            );
        for (ix, entry) in self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.is_visible(bindings))
        {
            block = block.child(entry.render(&format!("{id}-{ix}"), bindings, t, cx));
        }
        block.into_any_element()
    }
}
