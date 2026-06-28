import Foundation

struct CommandPaletteItem: Identifiable {
    let id = UUID()
    let title: String
    let icon: String
    let category: String
    let detail: String?
    let keywords: [String]
    /// Native shortcut glyphs to show on the row, e.g. "⇧⌘P". Optional.
    let shortcut: String?
    /// Nil for info-only "cheatsheet" rows that document a keybind but run nothing.
    let action: (() -> Void)?

    var isInfo: Bool {
        action == nil
    }

    init(
        title: String,
        icon: String,
        category: String,
        detail: String? = nil,
        keywords: [String] = [],
        shortcut: String? = nil,
        action: (() -> Void)? = nil
    ) {
        self.title = title
        self.icon = icon
        self.category = category
        self.detail = detail
        self.keywords = keywords
        self.shortcut = shortcut
        self.action = action
    }

    /// An info-only row that surfaces a keybind in the palette without executing.
    static func keybind(
        title: String,
        icon: String = "keyboard",
        shortcut: String,
        keywords: [String] = []
    ) -> CommandPaletteItem {
        CommandPaletteItem(
            title: title,
            icon: icon,
            category: "Shortcut",
            keywords: keywords,
            shortcut: shortcut,
            action: nil
        )
    }
}
