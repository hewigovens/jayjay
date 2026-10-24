import Foundation
import JayJayCore

struct HelpFeature: Decodable, Equatable, Identifiable {
    let id: String
    let title: String
    let summary: String
    let category: String
    let keywords: [String]
    let shortcut: String?
    let menuPath: String?
    let guideAnchor: String

    var helpAnchor: String {
        id
    }

    var commandPaletteTitle: String {
        Localized.Repo.helpTitle(title: title)
    }

    var commandPaletteKeywords: [String] {
        keywords + [title, summary, category, menuPath, guideAnchor].compactMap(\.self)
    }
}
