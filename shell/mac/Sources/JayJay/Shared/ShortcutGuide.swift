import Foundation

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
        ShortcutSection(title: "General", entries: [
            ShortcutEntry("Open Repository", .openRepository),
            ShortcutEntry("Command Palette", .commandPalette),
            ShortcutEntry("Refresh", .refresh),
            ShortcutEntry("Keyboard Shortcuts", .keyboardShortcuts),
            ShortcutEntry("Settings", .settings)
        ]),
        ShortcutSection(title: "View", entries: [
            ShortcutEntry("Hide / Show Sidebar", .toggleSidebar),
            ShortcutEntry("Zoom In", .zoomIn),
            ShortcutEntry("Zoom Out", .zoomOut),
            ShortcutEntry("Reset Zoom", .resetZoom)
        ]),
        ShortcutSection(title: "Navigation", entries: [
            ShortcutEntry(label: "Next / Previous Change", keys: ["J", "K"]),
            ShortcutEntry(label: "Next / Previous File", keys: ["J", "K"]),
            ShortcutEntry(label: "Move Up / Down", keys: ["↑", "↓"])
        ]),
        ShortcutSection(title: "Repository", entries: [
            ShortcutEntry("Filter by Revset", .revsetFilter),
            ShortcutEntry("Bookmark Manager", .bookmarkManager),
            ShortcutEntry("Repo Overview", .repoOverview),
            ShortcutEntry("Filter Overview Lanes", .filterOverviewLanes),
            ShortcutEntry("Undo Last Operation", .undoLastOperation),
            ShortcutEntry("Show in Finder", .showInFinder)
        ]),
        ShortcutSection(title: "Diff & Review", entries: [
            ShortcutEntry("Find in Diff", .findInDiff),
            ShortcutEntry(label: "Mark File Reviewed", keys: ["Space"]),
            ShortcutEntry("Save Review Note", .saveReviewNote),
            ShortcutEntry("Save Edited File", .saveEditedFile),
            ShortcutEntry("Expand All Files", .expandAllFiles),
            ShortcutEntry("Collapse All Files", .collapseAllFiles),
            ShortcutEntry(label: "Next / Previous File Card", keys: ["J", "K"]),
            ShortcutEntry(label: "Collapse / Expand File Card", keys: ["←", "→"]),
            ShortcutEntry(label: "Select File Card", keys: ["Space"]),
            ShortcutEntry(label: "Toggle File Card", keys: ["Return"])
        ]),
        ShortcutSection(title: "Conflicts", entries: [
            ShortcutEntry("Accept Left", .acceptLeftHunk),
            ShortcutEntry("Accept Right", .acceptRightHunk),
            ShortcutEntry("Previous Conflict", .previousHunk),
            ShortcutEntry("Next Conflict", .nextHunk)
        ]),
        ShortcutSection(title: "Drag & Drop", entries: [
            ShortcutEntry(label: "Confirm Drop", keys: ["Return"]),
            ShortcutEntry(label: "Cancel Drag", keys: ["Esc"])
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
