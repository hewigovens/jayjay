import Foundation
import JayJayCore

/// Thin shell over the shared core palette-history logic in jayjay-core.
enum CommandPaletteHistory {
    struct Recall {
        let query: String
        let historyIndex: Int?
    }

    static func record(_ command: String, in history: [String]) -> [String] {
        paletteRecordHistory(command: command, history: history)
    }

    static func recall(history: [String], historyIndex: Int?, older: Bool) -> Recall? {
        let index = historyIndex.map(UInt32.init)
        guard let recall = paletteRecallHistory(history: history, historyIndex: index, older: older)
        else { return nil }
        return Recall(query: recall.query, historyIndex: recall.historyIndex.map(Int.init))
    }
}

enum CommandPaletteSearch {
    /// Fuzzy-rank items against the query, best match first, with help topics after every other match.
    static func rank(query: String, items: [CommandPaletteItem]) -> [CommandPaletteItem] {
        let normalizedQuery = normalize(query: query)
        let candidates = items.map { item in
            ([item.title, item.category, item.detail ?? "", item.shortcut ?? ""] + item.keywords)
                .joined(separator: " ")
        }
        let ranked = fuzzyRank(query: normalizedQuery, candidates: candidates).map { items[Int($0)] }
        return ranked.filter { !$0.isHelpTopic } + ranked.filter(\.isHelpTopic)
    }

    static func normalize(query: String) -> String {
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        let lowercased = trimmed.lowercased()
        for prefix in ["help ", "? ", "how do i ", "how do you ", "how to "] where lowercased.hasPrefix(prefix) {
            return String(trimmed.dropFirst(prefix.count))
                .trimmingCharacters(in: .whitespacesAndNewlines)
        }
        return trimmed
    }
}
