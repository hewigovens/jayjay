import JayJayCore

enum DiffGutterGrouping {
    static func expandedChangedRange(
        in lines: [DiffLine],
        containing selection: ClosedRange<Int>
    ) -> ClosedRange<Int>? {
        guard !lines.isEmpty else { return nil }

        let selectedChangedIndices = selection.compactMap { lineNumber -> Int? in
            let index = lineNumber - 1
            guard lines.indices.contains(index), lines[index].isChangedInGutter else { return nil }
            return index
        }
        guard let anchor = selectedChangedIndices.first else { return nil }

        var lower = anchor
        while lower > 0, lines[lower - 1].isChangedInGutter {
            lower -= 1
        }

        var upper = anchor
        while upper + 1 < lines.count, lines[upper + 1].isChangedInGutter {
            upper += 1
        }

        guard selectedChangedIndices.allSatisfy({ lower ... upper ~= $0 }) else { return nil }
        return (lower + 1) ... (upper + 1)
    }
}

private extension DiffLine {
    var isChangedInGutter: Bool {
        style == .added || style == .removed
    }
}
