import Foundation
import JayJayCore

/// One documented shortcut: an action and the key-caps that trigger it. `keys` are display glyphs in press order, e.g. ["⇧", "⌘", "P"] or ["Space"].
struct ShortcutEntry: Identifiable {
    let label: String
    let keys: [String]
    let shortcut: AppShortcut?
    var id: String {
        label + keys.joined()
    }

    init(label: String, keys: [String]) {
        self.label = label
        self.keys = keys
        shortcut = nil
    }

    init(_ label: String, _ shortcut: AppShortcut) {
        self.label = label
        keys = shortcut.keyCaps
        self.shortcut = shortcut
    }
}

/// A titled group of related shortcuts shown as one block in the cheatsheet.
struct ShortcutSection: Identifiable {
    let title: String
    let entries: [ShortcutEntry]
    var id: String {
        title
    }
}

enum ShortcutGuide {
    static let sections: [ShortcutSection] = [
        ShortcutSection(title: Localized.Shortcuts.general, entries: [
            ShortcutEntry(Localized.Repo.openRepository, .openRepository),
            ShortcutEntry(Localized.Menu.commandPalette, .commandPalette),
            ShortcutEntry(Localized.Repo.refresh, .refresh),
            ShortcutEntry(Localized.Menu.keyboardShortcuts, .keyboardShortcuts),
            ShortcutEntry(Localized.Repo.settings, .settings)
        ]),
        ShortcutSection(title: Localized.Menu.view, entries: [
            ShortcutEntry(Localized.Palette.hideShowSidebar, .toggleSidebar),
            ShortcutEntry(Localized.Menu.zoomIn, .zoomIn),
            ShortcutEntry(Localized.Menu.zoomOut, .zoomOut),
            ShortcutEntry(Localized.Menu.resetZoom, .resetZoom)
        ]),
        ShortcutSection(title: Localized.Shortcuts.navigation, entries: [
            ShortcutEntry(label: Localized.Palette.nextPreviousChange, keys: ["J", "K"]),
            ShortcutEntry(label: Localized.Shortcuts.nextPreviousFile, keys: ["J", "K"]),
            ShortcutEntry(label: Localized.Shortcuts.moveUpDown, keys: ["↑", "↓"])
        ]),
        ShortcutSection(title: Localized.Menu.repository, entries: [
            ShortcutEntry(Localized.Shortcuts.filterByRevset, .revsetFilter),
            ShortcutEntry(Localized.Menu.bookmarkManager, .bookmarkManager),
            ShortcutEntry(Localized.Menu.repoOverview, .repoOverview),
            ShortcutEntry(Localized.Shortcuts.filterOverviewLanes, .filterOverviewLanes),
            ShortcutEntry(Localized.Menu.undoLastOperation, .undoLastOperation),
            ShortcutEntry(Localized.Common.showIn(app: "Finder"), .showInFinder)
        ]),
        ShortcutSection(title: Localized.Shortcuts.diffReview, entries: [
            ShortcutEntry(Localized.Palette.findInDiff, .findInDiff),
            ShortcutEntry(label: Localized.Palette.markFileReviewed, keys: ["Space"]),
            ShortcutEntry(Localized.Shortcuts.saveReviewNote, .saveReviewNote),
            ShortcutEntry(Localized.Shortcuts.saveEditedFile, .saveEditedFile),
            ShortcutEntry(Localized.Shortcuts.expandAllFiles, .expandAllFiles),
            ShortcutEntry(Localized.Shortcuts.collapseAllFiles, .collapseAllFiles),
            ShortcutEntry(label: Localized.Shortcuts.nextPreviousFileCard, keys: ["J", "K"]),
            ShortcutEntry(label: Localized.Shortcuts.collapseExpandFileCard, keys: ["←", "→"]),
            ShortcutEntry(label: Localized.Shortcuts.selectFileCard, keys: ["Space"]),
            ShortcutEntry(label: Localized.Shortcuts.toggleFileCard, keys: ["Return"])
        ]),
        ShortcutSection(title: Localized.Repo.presetConflicts, entries: [
            ShortcutEntry(Localized.Merge.acceptLeft, .acceptLeftHunk),
            ShortcutEntry(Localized.Merge.acceptRight, .acceptRightHunk),
            ShortcutEntry(Localized.Shortcuts.previousConflict, .previousHunk),
            ShortcutEntry(Localized.Shortcuts.nextConflict, .nextHunk)
        ]),
        ShortcutSection(title: Localized.Shortcuts.dragDrop, entries: [
            ShortcutEntry(label: Localized.Shortcuts.confirmDrop, keys: ["Return"]),
            ShortcutEntry(label: Localized.Shortcuts.cancelDrag, keys: ["Esc"])
        ])
    ]

    /// Split into two columns balanced by entry count, preserving section order.
    static var columns: [[ShortcutSection]] {
        let half = sections.reduce(0) { $0 + $1.entries.count } / 2
        var left: [ShortcutSection] = []
        var right: [ShortcutSection] = []
        var leftCount = 0
        for section in sections {
            if leftCount < half {
                left.append(section)
                leftCount += section.entries.count
            } else {
                right.append(section)
            }
        }
        return [left, right]
    }
}
