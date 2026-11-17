import Foundation

@MainActor
final class RegistrationWaiters {
    private var waiters: [String: [UUID: CheckedContinuation<RepoViewModel?, Never>]] = [:]

    func wait(for path: String, timeout: Duration) async -> RepoViewModel? {
        let id = UUID()
        let timeoutTask = Task { [weak self] in
            try? await Task.sleep(for: timeout)
            guard !Task.isCancelled else { return }
            self?.waiters[path]?.removeValue(forKey: id)?.resume(returning: nil)
        }
        defer { timeoutTask.cancel() }
        return await withCheckedContinuation { waiters[path, default: [:]][id] = $0 }
    }

    func resume(for path: String, with viewModel: RepoViewModel?) {
        waiters.removeValue(forKey: path)?.values.forEach { $0.resume(returning: viewModel) }
    }
}
