import Foundation

enum CommandPaletteHistory {
    private static let limit = 20

    struct Recall {
        let query: String
        let historyIndex: Int?
    }

    static func record(_ command: String, in history: [String]) -> [String] {
        let command = command.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !command.isEmpty else { return history }

        var values = [command]
        values.append(contentsOf: history.filter { $0 != command })
        values = Array(values.prefix(limit))
        return values
    }

    static func recall(history: [String], historyIndex: Int?, older: Bool) -> Recall? {
        guard !history.isEmpty else { return nil }
        let nextIndex: Int? = if older {
            min((historyIndex ?? -1) + 1, history.count - 1)
        } else if let historyIndex, historyIndex > 0 {
            historyIndex - 1
        } else {
            nil
        }
        return Recall(
            query: nextIndex.map { "jj \(history[$0])" } ?? "jj ",
            historyIndex: nextIndex
        )
    }
}

enum CommandPaletteSearch {
    static func matches(query: String, title: String, category: String, keywords: [String]) -> Bool {
        let terms = query
            .lowercased()
            .split(whereSeparator: \.isWhitespace)
            .map(String.init)
        guard !terms.isEmpty else { return true }

        let searchable = ([title, category] + keywords).map { $0.lowercased() }
        return terms.allSatisfy { term in
            searchable.contains { $0.contains(term) }
        }
    }
}
