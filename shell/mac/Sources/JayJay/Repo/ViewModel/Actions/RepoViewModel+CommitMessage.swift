import Foundation
import JayJayCore

extension RepoViewModel {
    @MainActor
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
