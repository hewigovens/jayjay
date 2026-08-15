import Foundation

/// One documented shortcut: an action and the key-caps that trigger it. `keys` are display glyphs in press order, e.g. ["⇧", "⌘", "P"] or ["Space"].
struct ShortcutEntry: Identifiable {
    let label: String
    let keys: [String]
    var id: String {
        label + keys.joined()
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

/// The canonical, complete keyboard-shortcut reference surfaced on ⌘/. Keep in sync with the real bindings (menus, palette, and view key handlers).
enum ShortcutGuide {
    static let sections: [ShortcutSection] = [
        ShortcutSection(title: "General", entries: [
            ShortcutEntry(label: "Open Repository", keys: ["⌘", "O"]),
            ShortcutEntry(label: "Command Palette", keys: ["⇧", "⌘", "P"]),
            ShortcutEntry(label: "Refresh", keys: ["⌘", "R"]),
            ShortcutEntry(label: "Keyboard Shortcuts", keys: ["⌘", "/"]),
            ShortcutEntry(label: "Settings", keys: ["⌘", ","])
        ]),
        ShortcutSection(title: "View", entries: [
            ShortcutEntry(label: "Zoom In", keys: ["⌘", "+"]),
            ShortcutEntry(label: "Zoom Out", keys: ["⌘", "−"]),
            ShortcutEntry(label: "Reset Zoom", keys: ["⌘", "0"])
        ]),
        ShortcutSection(title: "Navigation", entries: [
            ShortcutEntry(label: "Next / Previous Change", keys: ["J", "K"]),
            ShortcutEntry(label: "Next / Previous File", keys: ["J", "K"]),
            ShortcutEntry(label: "Move Up / Down", keys: ["↑", "↓"])
        ]),
        ShortcutSection(title: "Repository", entries: [
            ShortcutEntry(label: "Bookmark Manager", keys: ["⇧", "⌘", "B"]),
            ShortcutEntry(label: "Toggle Workspace Sidebar", keys: ["⌥", "⌘", "W"]),
            ShortcutEntry(label: "Undo Last Operation", keys: ["⇧", "⌘", "U"]),
            ShortcutEntry(label: "Show in Finder", keys: ["⌥", "⌘", "F"])
        ]),
        ShortcutSection(title: "Diff & Review", entries: [
            ShortcutEntry(label: "Find in Diff", keys: ["⌘", "F"]),
            ShortcutEntry(label: "Mark File Reviewed", keys: ["Space"]),
            ShortcutEntry(label: "Save Description", keys: ["⌘", "S"]),
            ShortcutEntry(label: "Expand All Files", keys: ["⌥", "⌘", "E"]),
            ShortcutEntry(label: "Collapse All Files", keys: ["⌥", "⌘", "C"]),
            ShortcutEntry(label: "Next / Previous File Card", keys: ["J", "K"]),
            ShortcutEntry(label: "Collapse / Expand File Card", keys: ["←", "→"]),
            ShortcutEntry(label: "Select File Card", keys: ["Space"]),
            ShortcutEntry(label: "Toggle File Card", keys: ["Return"])
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
