import Foundation
import JayJayCore

/// One rendered row of the unified diff body: a display line, or a review-note paragraph spliced in below the line it annotates (GitHub-style embedded comments).
enum DiffRenderRow {
    /// `lineNumber` is the 1-based position in the unspliced display lines; every consumer (groups, selection, note anchors) keeps that numbering.
    case line(DiffLine, lineNumber: Int)
    /// `indent` is the anchor line's leading whitespace, so the bubble starts at the code's first character. `isFirst`/`isLast` bound one note's bubble: reserved spacing above/below and the bubble outline.
    case note(text: String, indent: String, isFirst: Bool, isLast: Bool)
}

enum DiffNoteBubbleMetrics {
    /// Vertical room reserved above and below a bubble; the gutter mirrors it on its blank rows so line alignment survives.
    static let verticalSpacing: CGFloat = 7
    /// Horizontal padding between the bubble's rounded edge and its text.
    static let innerPadding: CGFloat = 7
}

/// Splices each active note's body under its anchor display line, one row per body line so gutter alignment survives wrapping. Stale notes keep their gutter marker but are not expanded (their anchor may no longer point at the right line); resolved notes keep a dimmed marker only.
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
