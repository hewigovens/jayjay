import JayJayCore

/// Keeps unsaved commit messages for workspaces visited by a window during its lifetime.
@MainActor
final class WorkspaceDraftStore {
    struct Draft {
        let summary: String
        let body: String
    }

    private var draftsByWindow: [Int: [String: Draft]] = [:]

    func preserve(from viewModel: RepoViewModel, in windowNumber: Int) {
        let path = normalizedRepositoryPath(path: viewModel.repoPath)
        let summary = viewModel.commitSummaryDraft
        let body = viewModel.commitDescriptionDraft
        if commitDraftIsClean(summary: summary, body: body, message: viewModel.workingCopyDescription) {
            draftsByWindow[windowNumber]?[path] = nil
        } else {
            draftsByWindow[windowNumber, default: [:]][path] = Draft(summary: summary, body: body)
        }
    }

    func draft(for path: String, in windowNumber: Int) -> Draft? {
        draftsByWindow[windowNumber]?[normalizedRepositoryPath(path: path)]
    }

    func clear(in windowNumber: Int) {
        draftsByWindow.removeValue(forKey: windowNumber)
    }

    func discard(for path: String) {
        let normalizedPath = normalizedRepositoryPath(path: path)
        for windowNumber in Array(draftsByWindow.keys) {
            draftsByWindow[windowNumber]?[normalizedPath] = nil
        }
    }
}
