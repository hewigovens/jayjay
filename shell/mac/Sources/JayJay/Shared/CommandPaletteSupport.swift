import Foundation
import JayJayCore

enum CommandPaletteHistory {
    private static let key = "jayjay.commandPalette.jjHistory"
    private static let limit = 20

    struct Recall {
        let query: String
        let historyIndex: Int?
    }

    static func load(defaults: UserDefaults = .standard) -> [String] {
        defaults.stringArray(forKey: key) ?? []
    }

    static func record(_ command: String, defaults: UserDefaults = .standard) -> [String] {
        let values = recordJjCommandHistory(
            command: command,
            existing: load(defaults: defaults),
            limit: UInt32(limit)
        )
        defaults.set(values, forKey: key)
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
