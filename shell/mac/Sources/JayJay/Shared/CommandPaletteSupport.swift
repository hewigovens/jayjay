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
    /// Fuzzy-rank items against the query, best match first.
    static func rank(query: String, items: [CommandPaletteItem]) -> [CommandPaletteItem] {
        let candidates = items.map { item in
            ([item.title, item.category, item.shortcut ?? ""] + item.keywords).joined(separator: " ")
        }
        return fuzzyRank(query: query, candidates: candidates).map { items[Int($0)] }
    }
}
