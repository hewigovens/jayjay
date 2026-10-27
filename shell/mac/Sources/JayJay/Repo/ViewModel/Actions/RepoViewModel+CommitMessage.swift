import Foundation
import JayJayCore

@MainActor
extension RepoViewModel {
    func generateCommitMessage(using order: [AiProvider]) async -> String? {
        do {
            guard let excerpt = try await awaitRepoTask({ try $0.diffExcerpt() }),
                  let answer = await order.firstAnswer({ await $0.commitMessage(for: excerpt) })
            else { return nil }
            aiProvider = answer.provider.label
            return answer.text
        } catch {
            present(error: error)
            return nil
        }
    }
}

extension RepoViewModel {
    /// A clean box follows the working copy; a typed draft is never replaced, even when @ moves to a described change.
    func applyWorkingCopy(changeId: String, description: String) {
        let previousDescription = workingCopyDescription
        workingCopyDescription = description
        guard !changeId.isEmpty else { return }
        let identityChanged = changeId != workingCopyChangeId
        let descriptionChanged = description != previousDescription
        guard identityChanged || descriptionChanged else { return }
        workingCopyChangeId = changeId
        let boxIsClean = commitDraftIsClean(
            summary: commitSummaryDraft,
            body: commitDescriptionDraft,
            message: previousDescription
        )
        guard boxIsClean else { return }
        commitSummaryDraft = commitSummary(message: description)
        commitDescriptionDraft = commitBody(message: description)
    }
}
