import JayJayCore

struct DiffMovedRowInfo {
    let moved: MovedRow?
    let style: DiffSpanStyle
    let number: UInt32?

    var badgeTitle: String? {
        guard let moved, moved.startsBlock, number != nil else { return nil }
        let arrow = style == .removed ? "→" : "←"
        return "\(arrow) L\(moved.counterpartLine)"
    }

    static let none = DiffMovedRowInfo(moved: nil, style: .context, number: nil)
}

extension DiffMovedRowInfo {
    init(_ line: DiffLine) {
        self.init(moved: line.moved, style: line.style, number: line.style == .removed ? line.oldLineNo : line.newLineNo)
    }

    init(_ side: RowSide) {
        self.init(moved: side.moved, style: side.style, number: UInt32(side.lineNo))
    }
}
