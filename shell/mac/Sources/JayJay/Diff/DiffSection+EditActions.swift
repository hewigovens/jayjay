import JayJayCore
import JayJayDiffUI
import SwiftUI

extension DiffSection: DiffGutterEditActions {
    var currentSelectedLineRange: ClosedRange<Int>? {
        selectedLineRange
    }

    var canOpenDiffEdit: Bool {
        onOpenDiffEdit != nil
    }

    var canAbandonSelectedLines: Bool {
        isWorkingCopy
    }

    func didSelectLines(_ lineRange: ClosedRange<Int>) {
        selectedLineRange = lineRange
    }

    func openDiffEdit() {
        onOpenDiffEdit?()
    }

    func abandonSelectedLines(in lineRange: ClosedRange<Int>) {
        abandonSelectedLines(lineRange: lineRange)
    }

    private func abandonSelectedLines(lineRange: ClosedRange<Int>) {
        guard let actions,
              let repo,
              let rev,
              let fileDiff
        else { return }

        let oldContent = loadedOldContent ?? hunk.oldContent
        let newContent = loadedNewContent ?? hunk.newContent
        let selectedKeys: Set<String> = Set(
            fileDiff.lines.enumerated().compactMap { index, line in
                let displayLine = index + 1
                guard lineRange.contains(displayLine),
                      line.style == .added || line.style == .removed
                else { return nil }
                return diffEditLineKey(line)
            }
        )
        guard !selectedKeys.isEmpty else { return }

        let path = hunk.path
        let oldPath = hunk.oldPath
        let hunkType = hunk.hunkType
        let ignoreWhitespace = settings.ignoreWhitespace
        Task.detached { [repo] in
            let fullDiff = repo.computeNativeDiffFull(
                path: path,
                oldContent: oldContent ?? "",
                newContent: newContent ?? "",
                ignoreWhitespace: ignoreWhitespace
            )
            let fullLineIndices = fullDiff.lines.enumerated().compactMap { index, line in
                selectedKeys.contains(diffEditLineKey(line)) ? index + 1 : nil
            }
            let ranges = diffEditCollapsedRanges(fullLineIndices)
            guard !ranges.isEmpty else { return }

            await MainActor.run {
                actions.applyDiffSelection(
                    rev: rev,
                    destination: .removeFromSource,
                    selections: [
                        DiffEditFileSelection(
                            path: path,
                            oldPath: oldPath,
                            oldContent: oldContent,
                            newContent: newContent,
                            hunkType: hunkType,
                            lineRanges: ranges
                        )
                    ],
                    message: "",
                    ignoreWhitespace: ignoreWhitespace
                )
            }
        }
    }
}

private func diffEditLineKey(_ line: DiffLine) -> String {
    let style = switch line.style {
        case .added: "added"
        case .removed: "removed"
        case .context: "context"
        case .separator: "separator"
        case .unchanged: "unchanged"
    }
    return "\(style)|\(line.oldLineNo.map(String.init) ?? "-")|\(line.newLineNo.map(String.init) ?? "-")"
}

private func diffEditCollapsedRanges(_ indices: [Int]) -> [DiffEditRange] {
    guard let first = indices.first else { return [] }

    var ranges: [DiffEditRange] = []
    var start = first
    var previous = first

    for index in indices.dropFirst() {
        if index == previous + 1 {
            previous = index
            continue
        }
        ranges.append(DiffEditRange(startLine: UInt32(start), endLine: UInt32(previous)))
        start = index
        previous = index
    }

    ranges.append(DiffEditRange(startLine: UInt32(start), endLine: UInt32(previous)))
    return ranges
}
