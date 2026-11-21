/// Identified by side and line number, so an expanded fold or a reload keeps the reader in place.
public struct ReaderLine: Identifiable, Sendable {
    public struct ID: Hashable, Sendable {
        public enum Side: Sendable { case new, old, fold, row }
        public var side: Side
        public var number: Int

        public init(side: Side, number: Int) {
            self.side = side
            self.number = number
        }
    }

    public var id: ID
    public var cell: RenderedDiff.Cell

    // The reader lays out every row natively; past these it shows a notice instead.
    static let maxRows = 10000
    static let maxBytes = 1_048_576

    public static func display(file: ReviewFileSummary?, diff: RenderedDiff?) -> (lines: [ReaderLine], placeholder: ReaderPlaceholder?) {
        guard let file else { return ([], nil) }
        if file.added + file.removed > maxRows || (diff?.rows.count ?? 0) > maxRows {
            return ([], .tooLarge)
        }
        guard let diff else { return ([], .noText(file)) }
        var lastNew = 0
        var size = 0
        var lines: [ReaderLine] = []
        for (offset, cell) in diff.rows.compactMap(\.cells.first).enumerated() {
            size += cell.spans.reduce(0) { $0 + $1.text.utf8.count }
            if size > maxBytes {
                return ([], .tooLarge)
            }
            let id: ID
            if let n = cell.newLine {
                lastNew = n
                id = ID(side: .new, number: n)
            } else if case let .number(n) = cell.oldLine {
                id = ID(side: .old, number: n)
            } else if cell.style == .separator {
                id = ID(side: .fold, number: lastNew + 1)
            } else {
                id = ID(side: .row, number: offset)
            }
            lines.append(ReaderLine(id: id, cell: cell))
        }
        return (lines, lines.isEmpty ? .noText(file) : nil)
    }

    public static func readingAnchor(saved: ID?, in lines: [ReaderLine]) -> ID? {
        if let saved {
            return closest(saved, in: lines)
        }
        guard let edit = lines.firstIndex(where: { $0.cell.style == .added || $0.cell.style == .removed }) else { return lines.first?.id }
        var start = edit
        while start > 0, edit - start < 2, lines[start - 1].cell.style == .context {
            start -= 1
        }
        return lines[start].id
    }

    public static func closest(_ id: ID, in lines: [ReaderLine]) -> ID? {
        if lines.contains(where: { $0.id == id }) {
            return id
        }
        let side: ID.Side = id.side == .fold ? .new : id.side
        return lines.lazy.map(\.id).filter { $0.side == side }.min { abs($0.number - id.number) < abs($1.number - id.number) }
            ?? lines.first?.id
    }
}
