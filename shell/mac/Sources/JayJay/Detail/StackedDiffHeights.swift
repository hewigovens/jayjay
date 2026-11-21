import CoreGraphics

/// Remembers how tall each stacked diff last was: a diff scrolled back into view reloads, and at any other height it would move the files below it.
final class StackedDiffHeights {
    private var commitId: String?
    private var heights: [String: CGFloat] = [:]

    func height(of path: String, in commitId: String?) -> CGFloat? {
        commitId == self.commitId ? heights[path] : nil
    }

    func record(_ height: CGFloat, of path: String, in commitId: String?) {
        if commitId != self.commitId {
            self.commitId = commitId
            heights.removeAll()
        }
        heights[path] = height
    }
}
