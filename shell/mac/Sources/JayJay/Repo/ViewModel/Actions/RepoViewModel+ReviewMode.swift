import JayJayCore

extension RepoViewModel {
    var selectedReviewMode: ReviewMode {
        guard comparison == nil, let change = selectedChange?.info else { return .unavailable }
        return reviewStore.reviewMode(for: change)
    }

    func toggleReviewMode() {
        let mode = selectedReviewMode
        guard mode.isSwitchable, let change = selectedChange?.info else { return }
        reviewStore.setReviewMode(mode == .off, for: change)
    }
}
