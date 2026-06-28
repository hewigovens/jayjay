import Foundation
import JayJayCore

enum DiffRenderRow {
    case line(DiffLine, lineNumber: Int)
    case note(text: String, indent: String, isFirst: Bool, isLast: Bool)
}

func diffRenderRows(
    displayLines: [DiffLine],
    notesByLine: [Int: [DiffReviewNoteSummary]]
) -> [DiffRenderRow] {
    var rows: [DiffRenderRow] = []
    rows.reserveCapacity(displayLines.count)
    for (index, line) in displayLines.enumerated() {
        let lineNumber = index + 1
        rows.append(.line(line, lineNumber: lineNumber))
        let indent = String(line.rawText.prefix { $0 == " " || $0 == "\t" })
        for note in notesByLine[lineNumber] ?? [] where !note.isStale && !note.isResolved {
            let bodyLines = note.body.split(separator: "\n", omittingEmptySubsequences: false)
            for (bodyIndex, text) in bodyLines.enumerated() {
                rows.append(.note(
                    text: String(text),
                    indent: indent,
                    isFirst: bodyIndex == 0,
                    isLast: bodyIndex == bodyLines.count - 1
                ))
            }
        }
    }
    return rows
}
