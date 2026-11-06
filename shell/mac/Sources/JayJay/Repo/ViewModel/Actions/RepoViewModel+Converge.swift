import Foundation
import JayJayCore

extension RepoViewModel {
    func previewConvergence(revision: String, review: ConvergeReview) {
        let requestId = UUID()
        review.requestId = requestId
        review.isLoading = true
        review.isCurrent = false
        review.error = nil
        runRepoTask { repo in
            try repo.convergePreview(rev: revision)
        } onSuccess: { _, plan in
            guard review.requestId == requestId else { return }
            review.accept(plan)
        } onFailure: { _, error in
            guard review.requestId == requestId else { return }
            review.isLoading = false
            review.error = error.friendlyDescription
        }
    }

    func applyConvergence(review: ConvergeReview, onSuccess: @escaping () -> Void) {
        guard review.canApply, let plan = review.plan else { return }
        let choices = review.choices
        let requestId = review.requestId
        review.isApplying = true
        let started = performResult(
            selectingResult: { $0.successorCommitId },
            onSuccess: { _, _ in
                guard review.requestId == requestId else { return }
                review.isApplying = false
                onSuccess()
            },
            onFailure: { viewModel, error in
                guard review.requestId == requestId else { return }
                review.isApplying = false
                review.isCurrent = false
                if case JayError.ConvergePlanStale = error {
                    review.wasStale = true
                    viewModel.previewConvergence(revision: plan.changeId, review: review)
                } else {
                    review.error = error.friendlyDescription
                }
            },
            { try $0.convergeApply(plan: plan, choices: choices) }
        )
        if !started {
            review.isApplying = false
            review.isCurrent = false
            review.error = Localized.Converge.unavailable
        }
    }
}
