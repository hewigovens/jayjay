import JayJayCore

extension ChangeInfo {
    var selectionRevision: String {
        isDivergent ? commitId.id : changeId.id
    }

    /// Mirrors `ChangeInfo::accepts_review_marks` in jayjay-primitives.
    var acceptsReviewMarks: Bool {
        !isImmutable && (isWorkingCopy || !isDivergent)
    }

    func matchesRevision(_ rev: String) -> Bool {
        changeId.id == rev || commitId.id == rev
    }
}
