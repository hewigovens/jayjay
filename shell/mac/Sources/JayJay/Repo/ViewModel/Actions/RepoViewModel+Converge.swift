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
        let filter = revsetFilter.revset
        review.isApplying = true
        let started = performResult(
            selectingResult: { (result: ConvergeApplyResult) in result.outcome.successorCommitId },
            onSuccess: { (viewModel: RepoViewModel, result: ConvergeApplyResult) in
                let outcome = result.outcome
                viewModel.pendingConflictEditor = outcome.conflictedPaths.first.map { (outcome.successorCommitId, $0) }
                guard review.requestId == requestId else { return }
                review.isApplying = false
                onSuccess()
            },
            afterRefresh: { (viewModel: RepoViewModel, result: ConvergeApplyResult) in
                if result.isHiddenByFilter {
                    viewModel.applyFilter("", selecting: result.outcome.successorCommitId)
                }
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
            { (repo: JayJayRepo) throws -> ConvergeApplyResult in
                let outcome = try repo.convergeApply(plan: plan, choices: choices)
                let isHiddenByFilter = (try? repo.log(revset: "(\(filter)) & \(outcome.successorCommitId)").isEmpty) ?? false
                return ConvergeApplyResult(outcome: outcome, isHiddenByFilter: isHiddenByFilter)
            }
        )
        if !started {
            review.isApplying = false
            review.isCurrent = false
            review.error = Localized.Converge.unavailable
        }
    }

    func takePendingConflictEditor(commitId: String) -> String? {
        guard let pending = pendingConflictEditor, pending.commitId == commitId else { return nil }
        pendingConflictEditor = nil
        return pending.path
    }

    func divergentVersionCount(of changeId: String) -> Int {
        changes.count { $0.changeId.id == changeId }
    }
}

private struct ConvergeApplyResult {
    let outcome: ConvergeOutcome
    let isHiddenByFilter: Bool
}
