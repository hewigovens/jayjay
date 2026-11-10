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

    /// The run each line belongs to, as `expandedChangedRange` would return for that single line, in one linear pass.
    static func changedRanges(in lines: [DiffLine]) -> [ClosedRange<Int>?] {
        var ranges = [ClosedRange<Int>?](repeating: nil, count: lines.count)
        var start = 0
        while start < lines.count {
            guard lines[start].isChangedInGutter else {
                start += 1
                continue
            }
            var end = start
            while end + 1 < lines.count, lines[end + 1].isChangedInGutter {
                end += 1
            }
            let range = (start + 1) ... (end + 1)
            for index in start ... end {
                ranges[index] = range
            }
            start = end + 1
        }
        return ranges
    }
}

private extension DiffLine {
    var isChangedInGutter: Bool {
        style == .added || style == .removed
    }
}
