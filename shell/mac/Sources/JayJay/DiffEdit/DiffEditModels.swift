import JayJayCore
import Foundation

enum DiffEditSelectionMode: Equatable {
    case file
    case hunk(ClosedRange<Int>)
    case lines(ClosedRange<Int>)

    var selectedLineRange: ClosedRange<Int>? {
        switch self {
            case .file: nil
            case let .hunk(range), let .lines(range): range
        }
    }

    var badgeText: String {
        switch self {
            case .file: "File"
            case .hunk: "Hunk"
            case .lines: "Lines"
        }
    }
}

struct DiffEditLoadedFile {
    let hunk: DiffHunk
    let oldContent: String?
    let newContent: String?
    let diff: FileDiff

    var changedLineRanges: [ClosedRange<Int>] {
        var ranges: [ClosedRange<Int>] = []
        var start: Int?

        for (index, line) in diff.lines.enumerated() {
            let lineNumber = index + 1
            let isChanged = line.style == .added || line.style == .removed
            if isChanged {
                start = start ?? lineNumber
            } else if let currentStart = start {
                ranges.append(currentStart ... (lineNumber - 1))
                start = nil
            }
        }

        if let start {
            ranges.append(start ... diff.lines.count)
        }

        return ranges
    }

    func changedLineCount(for mode: DiffEditSelectionMode) -> Int {
        switch mode {
            case .file:
                changedLineRanges.reduce(0) { $0 + $1.count }
            case let .hunk(range), let .lines(range):
                diff.lines[range].reduce(into: 0) { count, line in
                    if line.style == .added || line.style == .removed {
                        count += 1
                    }
                }
        }
    }

    func makeSelection(mode: DiffEditSelectionMode) -> DiffEditFileSelection? {
        let ranges: [ClosedRange<Int>] = switch mode {
            case .file: changedLineRanges
            case let .hunk(range), let .lines(range): [range]
        }

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
}

private extension ClosedRange where Bound == Int {
    var count: Int {
        upperBound - lowerBound + 1
    }
}
