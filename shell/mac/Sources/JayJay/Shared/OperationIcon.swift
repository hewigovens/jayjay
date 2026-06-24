import SwiftUI

/// Maps a jj operation description to an SF Symbol + tint. Shared by the Operation Log, the status bar's last-operation item, and the evolog so the same operation reads the same everywhere.
enum OperationIcon {
    private struct Entry {
        let keywords: [String]
        let symbol: String
        let color: Color
    }

    /// Keyword → icon, in priority order. Specific verbs come before the generic "commit" fallback, since most descriptions contain "commit" (e.g. "rebase commit X"). First entry with a matching keyword wins.
    private static let table: [Entry] = [
        Entry(keywords: ["snapshot"], symbol: "camera", color: .secondary),
        Entry(keywords: ["bookmark"], symbol: "bookmark", color: .green),
        Entry(keywords: ["rebase", "parallelize"], symbol: "arrow.triangle.branch", color: .blue),
        Entry(keywords: ["abandon"], symbol: "trash", color: .red),
        Entry(keywords: ["squash", "absorb"], symbol: "arrow.down.to.line", color: .blue),
        Entry(keywords: ["describe"], symbol: "text.bubble", color: .orange),
        Entry(keywords: ["duplicate"], symbol: "plus.square.on.square", color: .blue),
        Entry(keywords: ["split"], symbol: "scissors", color: .blue),
        Entry(keywords: ["check out", "edit"], symbol: "pencil.circle", color: .orange),
        Entry(keywords: ["new "], symbol: "plus.circle", color: .green),
        Entry(keywords: ["restore", "undo"], symbol: "arrow.uturn.backward", color: .purple),
        Entry(keywords: ["merge"], symbol: "arrow.triangle.merge", color: .blue),
        Entry(keywords: ["fetch"], symbol: "arrow.down.circle", color: .blue),
        Entry(keywords: ["push"], symbol: "arrow.up.circle", color: .blue),
        Entry(keywords: ["import", "export"], symbol: "arrow.left.arrow.right", color: .secondary),
        Entry(
            keywords: ["reconcile", "concurrent"],
            symbol: "arrow.triangle.2.circlepath",
            color: .secondary
        ),
        Entry(keywords: ["commit"], symbol: "checkmark.circle", color: .green)
    ]

    /// Symbol + tint for a jj operation description.
    static func style(for description: String) -> (symbol: String, color: Color) {
        let d = description.lowercased()
        for entry in table where entry.keywords.contains(where: { d.contains($0) }) {
            return (entry.symbol, entry.color)
        }
        return ("clock.arrow.circlepath", .secondary)
    }

    /// Just the symbol, for call sites that don't tint.
    static func symbol(for description: String) -> String {
        style(for: description).symbol
    }
}
