import CoreGraphics

/// Remembers how tall each stacked diff last was: a diff scrolled back into view reloads, and at any other height it would move the files below it.
final class StackedDiffHeights {
    struct Basis: Equatable {
        var commitId: String?
        var compareFromRev: String?
        var ignoreWhitespace: Bool
        var typography: Typography
        var columns = Columns.alwaysUnified

        func matches(_ other: Basis) -> Bool {
            var aligned = self
            if columns == .alwaysUnified || other.columns == .alwaysUnified {
                aligned.columns = other.columns
            }
            return aligned == other
        }
    }

    enum Columns {
        case unified
        case sideBySide
        /// A diff that renders unified whatever the setting keeps its height through the toggle.
        case alwaysUnified
    }

    private var commitId: String?
    private var heights: [String: (basis: Basis, height: CGFloat)] = [:]

    func height(of path: String, in basis: Basis) -> CGFloat? {
        guard let entry = heights[path], entry.basis.matches(basis) else { return nil }
        return entry.height
    }

    func record(_ height: CGFloat, of path: String, in basis: Basis) {
        if basis.commitId != commitId {
            commitId = basis.commitId
            heights.removeAll()
        }
        heights[path] = (basis, height)
    }

    /// A diff on screen through a layout change reports a new height only if it has one; the diffs out of view come back at heights nobody knows.
    func carry(_ path: String, to basis: Basis) {
        guard basis.commitId == commitId, let height = heights[path]?.height else { return }
        heights[path] = (basis, height)
    }
}
