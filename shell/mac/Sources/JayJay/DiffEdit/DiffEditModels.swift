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
        hunk.hunkType != .renamed
            && DiffPlaceholder.isEditableText(oldContent)
            && DiffPlaceholder.isEditableText(newContent)
    }

    func changedLineCount(selectedLines: Set<Int>) -> Int {
        changedLineNumbers.filter(selectedLines.contains).count
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

private extension DiffLine {
    var isChanged: Bool {
        style == .added || style == .removed
    }
}
