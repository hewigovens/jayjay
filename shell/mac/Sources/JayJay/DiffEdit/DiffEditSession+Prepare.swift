import JayJayCore
import JayJayDiffUI

extension DiffEditSession {
    /// removeFromSource keeps only selected lines, so every editable hunk is strictly reloaded first — card loads go through DiffStore, which converts repository errors into empty content that would silently keep a file whole.
    func prepareRemoveFromSource() {
        guard let repo else {
            finishApply(.removeFromSource)
            return
        }
        removalTask?.cancel()
        isPreparingRemoval = true
        let hunks = detail.diff
        let renderedByPath = loadedFiles
        let ignoreWhitespace = settings.ignoreWhitespace
        removalTask = Task {
            defer { isPreparingRemoval = false }
            let outcome = await strictReloadAll(
                hunks: hunks, repo: repo, ignoreWhitespace: ignoreWhitespace
            )
            // One guard for every published outcome: a mode toggle mid-preparation cancels this task and wipes the selections, so nothing from the old-mode reload — the failure alert included — may reach the replacement session.
            guard !Task.isCancelled, settings.ignoreWhitespace == ignoreWhitespace else { return }
            switch outcome {
                case .canceled:
                    return
                case let .failed(path):
                    applyLoadFailurePath = path
                case let .loaded(strictByPath):
                    if let stalePath = Self.firstStalePath(
                        renderedByPath: renderedByPath,
                        strictByPath: strictByPath,
                        orderedPaths: hunks.map(\.path)
                    ) {
                        applyStalePath = stalePath
                        return
                    }
                    loadedFiles = strictByPath.merging(renderedByPath) { _, rendered in rendered }
                    finishApply(.removeFromSource)
            }
        }
    }

    private enum StrictReloadOutcome {
        case loaded([String: DiffEditLoadedFile])
        case failed(String)
        case canceled
    }

    nonisolated static func firstStalePath(
        renderedByPath: [String: DiffEditLoadedFile],
        strictByPath: [String: DiffEditLoadedFile],
        orderedPaths: [String]
    ) -> String? {
        orderedPaths.first { path in
            guard let rendered = renderedByPath[path], let strict = strictByPath[path] else {
                return false
            }
            return !rendered.hasSameSelectionBasis(as: strict)
        }
    }

    private func strictReloadAll(
        hunks: [DiffHunk],
        repo: JayJayRepo,
        ignoreWhitespace: Bool
    ) async -> StrictReloadOutcome {
        let rev = sessionCommit
        var strictByPath: [String: DiffEditLoadedFile] = [:]
        for hunk in hunks {
            if Task.isCancelled {
                return .canceled
            }
            do {
                if let loaded = try await loadFileStrict(
                    hunk: hunk, rev: rev, ignoreWhitespace: ignoreWhitespace, repo: repo
                ) {
                    strictByPath[hunk.path] = loaded
                }
            } catch {
                return .failed(hunk.path)
            }
        }
        return Task.isCancelled ? .canceled : .loaded(strictByPath)
    }

    private func loadFileStrict(
        hunk: DiffHunk,
        rev: String,
        ignoreWhitespace: Bool,
        repo: JayJayRepo
    ) async throws -> DiffEditLoadedFile? {
        // Submodule placeholders are synthesized shell-side and absent from the jj tree diff; reloading one would fail and abort the whole apply.
        guard hunk.projection == nil, hunk.hunkType != .renamed, !hunk.isSubmodulePlaceholder
        else { return nil }
        let content = try await diffStore.loadContentStrict(hunk: hunk, rev: rev, repo: repo)
        return await DiffEditLoadedFile.make(
            hunk: hunk, oldContent: content.oldContent, newContent: content.newContent,
            repo: repo, ignoreWhitespace: ignoreWhitespace, highlight: false
        )
    }
}
