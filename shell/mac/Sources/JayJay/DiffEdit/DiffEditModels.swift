import Foundation
import JayJayCore
import JayJayDiffUI

struct DiffEditLoadedFile {
    let hunk: DiffHunk
    let oldContent: String?
    let newContent: String?
    let diff: FileDiff

    var changedLineNumbers: [Int] {
        diff.lines.enumerated().compactMap { index, line in
            line.isChanged ? index + 1 : nil
        }
    }

    var changedLineSet: Set<Int> {
        Set(changedLineNumbers)
    }

    var supportsDiffEdit: Bool {
        hunk.projection == nil
            && hunk.hunkType != .renamed
            && DiffPlaceholder.isEditableText(oldContent)
            && DiffPlaceholder.isEditableText(newContent)
    }

    func changedLineCount(selectedLines: Set<Int>) -> Int {
        changedLineNumbers.filter(selectedLines.contains).count
    }

    /// Content equality is sufficient: the diff is deterministic in (path, contents, whitespace mode), and comparing diffs would false-trip between highlighted and plain variants.
    func hasSameSelectionBasis(as other: DiffEditLoadedFile) -> Bool {
        oldContent == other.oldContent && newContent == other.newContent
    }

    /// Always the full (uncollapsed) diff — line selection indices must match what the Rust side computes when applying. Selection-only callers pass highlight: false; tree-sitter setup costs tens of milliseconds per file.
    static func make(
        hunk: DiffHunk,
        oldContent: String?,
        newContent: String?,
        repo: JayJayRepo,
        ignoreWhitespace: Bool,
        highlight: Bool = true
    ) async -> DiffEditLoadedFile {
        let path = hunk.path
        let old = oldContent ?? ""
        let new = newContent ?? ""
        let diff = await Task.detached {
            highlight
                ? repo.computeNativeDiffFull(
                    path: path, oldContent: old, newContent: new, ignoreWhitespace: ignoreWhitespace
                )
                : repo.computeNativeDiffFullPlain(
                    path: path, oldContent: old, newContent: new, ignoreWhitespace: ignoreWhitespace
                )
        }.value
        return DiffEditLoadedFile(hunk: hunk, oldContent: oldContent, newContent: newContent, diff: diff)
    }

    func makeSelection(selectedLines: Set<Int>) -> DiffEditFileSelection? {
        let lineNumbers = changedLineNumbers.filter(selectedLines.contains)
        let ranges = collapseRanges(lineNumbers)
        guard !ranges.isEmpty else { return nil }

        return DiffEditFileSelection(
            path: hunk.path,
            oldPath: hunk.oldPath,
            oldContent: oldContent,
            newContent: newContent,
            hunkType: hunk.hunkType,
            lineRanges: ranges.map {
                DiffEditRange(startLine: UInt32($0.lowerBound), endLine: UInt32($0.upperBound))
            }
        )
    }

    func makeInverseSelection(selectedLines: Set<Int>) -> DiffEditFileSelection? {
        makeSelection(selectedLines: changedLineSet.subtracting(selectedLines))
    }
}

private func collapseRanges(_ lineNumbers: [Int]) -> [ClosedRange<Int>] {
    guard let first = lineNumbers.first else { return [] }

    var ranges: [ClosedRange<Int>] = []
    var start = first
    var previous = first

    for lineNumber in lineNumbers.dropFirst() {
        if lineNumber == previous + 1 {
            previous = lineNumber
            continue
        }
        ranges.append(start ... previous)
        start = lineNumber
        previous = lineNumber
    }

    ranges.append(start ... previous)
    return ranges
}

extension DiffLine {
    var isChanged: Bool {
        style == .added || style == .removed
    }
}
