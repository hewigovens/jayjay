public protocol DiffGutterReviewActions: DiffGutterContextActions {
    /// Whether the per-group review column renders; false where review state doesn't apply (e.g., compare/interdiff mode against another revision).
    var reviewModeEnabled: Bool { get }

    func hunkReviewState(groupIndex: UInt32) -> DiffGutterHunkReviewState
    func toggleHunkReviewed(groupIndex: UInt32)
}
