import AppKit
import Carbon
import Foundation

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
        "Help: \(title)"
    }

    var commandPaletteKeywords: [String] {
        keywords + [title, summary, category, menuPath, guideAnchor].compactMap(\.self)
    }
}

enum HelpFeatureIndex {
    static let bundled: [HelpFeature] = load()

    static func load(bundle: Bundle = Bundle(for: HelpFeatureBundleToken.self)) -> [HelpFeature] {
        guard let url = bundle.url(forResource: "HelpFeatures", withExtension: "json"),
              let data = try? Data(contentsOf: url)
        else {
            return []
        }
        return decode(data: data)
    }

    static func decode(data: Data) -> [HelpFeature] {
        (try? JSONDecoder().decode([HelpFeature].self, from: data)) ?? []
    }
}

enum HelpBook {
    private static let bookTitle = "JayJay Manual"
    private static let bookName = NSHelpManager.BookName(bookTitle)
    private static let pageByAnchor = [
        "home": "Start.html",
        "open": "topics/open.html",
        "navigate": "topics/navigate.html",
        "review": "topics/review.html",
        "compare": "topics/compare.html",
        "split": "topics/split.html",
        "operations": "topics/operations.html",
        "bookmarks": "topics/bookmarks.html",
        "stacked-prs": "topics/stacked-prs.html",
        "conflicts": "topics/conflicts.html",
        "inspection": "topics/inspection.html",
        "palette": "topics/palette.html",
        "settings": "topics/settings.html"
    ]

    static let onlineGuideURL = URL(string: "https://jayjay.hewig.dev/guide.html")!
    static let issueURL = URL(string: "https://github.com/hewigovens/jayjay/issues/new")!

    @MainActor
    static func open(anchor: String? = nil) {
        let anchorName = anchor ?? "home"
        guard openSystemHelp(anchor: anchorName) else {
            NSHelpManager.shared.openHelpAnchor(NSHelpManager.AnchorName(anchorName), inBook: bookName)
            return
        }
    }

    static func openOnlineGuide(anchor: String? = nil) {
        guard let anchor, var components = URLComponents(url: onlineGuideURL, resolvingAgainstBaseURL: false) else {
            NSWorkspace.shared.open(onlineGuideURL)
            return
        }
        components.fragment = anchor
        NSWorkspace.shared.open(components.url ?? onlineGuideURL)
    }

    static func requestFeatureURL(query: String) -> URL {
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        var components = URLComponents(url: issueURL, resolvingAgainstBaseURL: false)!
        components.queryItems = [
            URLQueryItem(
                name: "title",
                value: trimmed.isEmpty ? "Feature request from JayJay help" : "Feature request: \(trimmed)"
            ),
            URLQueryItem(
                name: "body",
                value: """
                I searched JayJay help for:

                \(trimmed.isEmpty ? "(empty query)" : trimmed)

                I expected to find:


                """
            )
        ]
        return components.url ?? issueURL
    }

    private static func openSystemHelp(anchor: String) -> Bool {
        guard let relativePath = pageByAnchor[anchor] else {
            return false
        }
        guard let helpBookURL = Bundle.main.url(forResource: "JayJay", withExtension: "help") else {
            return false
        }

        let registerStatus = AHRegisterHelpBookWithURL(helpBookURL as CFURL)
        let gotoStatus = AHGotoPage(bookTitle as CFString, relativePath as CFString, nil)
        return registerStatus == noErr && gotoStatus == noErr
    }
}

private final class HelpFeatureBundleToken {}
