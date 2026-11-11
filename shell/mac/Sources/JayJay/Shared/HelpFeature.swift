import Foundation

struct HelpFeature: Decodable, Equatable, Identifiable {
    let id: String
    let title: String
    let summary: String
    let category: String
    let keywords: [String]
    let shortcut: String?
    let command: AppShortcut?
    let menuPath: String?
    let guideAnchor: String

    func effectiveShortcut(in store: ShortcutStore) -> String? {
        if let command {
            return store.byID[command.rawValue]?.primary?.caps.joined()
        }
        return shortcut
    }

    var helpAnchor: String {
        id
    }

    var commandPaletteKeywords: [String] {
        keywords + [title, summary, category, menuPath, guideAnchor].compactMap(\.self)
    }
}
