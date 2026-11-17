import Foundation

extension RepoViewModel {
    /// Resolves false when no detail pane showing `revision` takes the request within `timeout`.
    @MainActor
    func revealFile(_ path: String, in revision: String, timeout: Duration = .seconds(5)) async -> Bool {
        finishFileReveal(shown: false)
        let request = FileRevealRequest(revision: revision, path: path)
        let timeoutTask = Task { [weak self] in
            try? await Task.sleep(for: timeout)
            guard !Task.isCancelled, self?.fileRevealRequest?.id == request.id else { return }
            self?.finishFileReveal(shown: false)
        }
        defer { timeoutTask.cancel() }
        return await withCheckedContinuation {
            fileRevealCompletion = $0
            fileRevealRequest = request
        }
    }

    func finishFileReveal(shown: Bool) {
        fileRevealRequest = nil
        fileRevealCompletion?.resume(returning: shown)
        fileRevealCompletion = nil
    }
}
