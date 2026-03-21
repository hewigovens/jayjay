import Foundation
import JayJayBindings
#if canImport(FoundationModels)
    import FoundationModels
#endif

@Observable
final class RepoViewModel: ChangeActions, DAGActions, BookmarkActions {
    let repoPath: String
    private(set) var graphEntries: [GraphEntry] = []
    var changes: [ChangeInfo] {
        graphEntries.map(\.change)
    }

    private(set) var selectedChange: ChangeDetail?
    private(set) var selectedChangeId: String?
    private(set) var bookmarks: [BookmarkInfo] = []
    private(set) var workingCopyDescription: String = ""
    private(set) var opLogEntries: [OpLogEntry] = []
    var error: String?
    var info: String?
    private(set) var isLoading = false

    var revset: String = defaultRevset()

    let repo: JayJayRepo

    private(set) var aiProvider: String = ""
    private var fsWatcher: RepoFSWatcher?
    private var refreshTask: Task<Void, Never>?

    init(path: String) throws {
        repoPath = path
        repo = try JayJayRepo.open(path: path)
        aiProvider = Self.detectAIProvider()
        fsWatcher = RepoFSWatcher(repoPath: path) { [weak self] in
            self?.refresh()
        }
    }

    private static func detectAIProvider() -> String {
        let cli = detectAiProvider() // from Rust via uniffi
        if !cli.isEmpty { return cli }
        #if canImport(FoundationModels)
            if #available(macOS 26.0, *) { return "Apple Intelligence" }
        #endif
        return ""
    }

    func applyRevset(_ newRevset: String) {
        revset = newRevset
        refresh(selecting: "@")
    }

    func refresh(selecting preferredRev: String? = nil) {
        refreshTask?.cancel()
        isLoading = true
        error = nil
        let currentSelection = selectedChangeId
        refreshTask = Task.detached { [repo, revset] in
            do {
                try repo.refreshWorkingCopy()
                guard !Task.isCancelled else { return }

                // Try the revset — if it fails, show empty list (not an error alert)
                let graph: [GraphEntry]
                do {
                    graph = try repo.logGraph(revset: revset)
                } catch {
                    guard !Task.isCancelled else { return }
                    await MainActor.run { [weak self] in
                        self?.graphEntries = []
                        self?.selectedChange = nil
                        self?.selectedChangeId = nil
                        self?.isLoading = false
                    }
                    return
                }

                guard !Task.isCancelled else { return }

                let log = graph.map(\.change)
                let marks = try repo.listBookmarks()
                let detail = try Self.loadSelectedDetail(
                    repo: repo,
                    log: log,
                    preferredRev: preferredRev ?? currentSelection
                )
                let wcDesc = log.first(where: { $0.isWorkingCopy })?.description ?? ""
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.graphEntries = graph
                    self?.bookmarks = marks
                    self?.selectedChange = detail
                    self?.selectedChangeId = detail?.info.changeId
                    self?.workingCopyDescription = wcDesc
                    self?.isLoading = false
                }
            } catch {
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                    self?.isLoading = false
                }
            }
        }
    }

    func select(changeId: String?) {
        selectedChangeId = changeId
        guard let changeId else {
            selectedChange = nil
            return
        }

        Task.detached { [repo] in
            do {
                // Fast: file list only, no content loading
                let detail = try repo.showSummary(rev: changeId)
                await MainActor.run { [weak self] in
                    self?.selectedChange = detail
                    self?.selectedChangeId = detail.info.changeId
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func describeChange(rev: String, message: String) {
        describe(rev: rev, message: message)
    }

    func describe(rev: String, message: String) {
        Task.detached { [repo] in
            do {
                try repo.describe(rev: rev, message: message)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: rev)
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func describeWorkingCopy(message: String) {
        Task.detached { [repo] in
            do {
                try repo.describe(rev: "@", message: message)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func commit(message: String) {
        Task.detached { [repo] in
            do {
                try repo.commitWithSubmodules(message: message)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func generateCommitMessage() async -> String? {
        do {
            let summary = try repo.diffSummary()
            if summary.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                return nil
            }

            // 1. Try external AI CLIs (codex, then claude) via Rust
            let cliProvider = detectAiProvider()
            if !cliProvider.isEmpty {
                let cliResult: String? = await Task.detached { [repo] in
                    repo.generateCommitMessage(diffSummary: summary)
                }.value
                if let msg = cliResult, !msg.isEmpty {
                    await MainActor.run { [weak self] in self?.aiProvider = cliProvider }
                    return msg
                }
            }

            // 2. Fall back to Apple Foundation Models
            if let msg = await Self.generateWithLocalLLM(diffSummary: summary) {
                await MainActor.run { [weak self] in self?.aiProvider = "Apple Intelligence" }
                return msg
            }

            return nil
        } catch {
            await MainActor.run { [weak self] in
                self?.error = error.friendlyDescription
            }
            return nil
        }
    }

    @MainActor
    private static func generateWithLocalLLM(diffSummary: String) async -> String? {
        #if canImport(FoundationModels)
            if #available(macOS 26.0, *) {
                return await generateWithFoundationModels(diffSummary: diffSummary)
            }
        #endif
        return nil
    }

    #if canImport(FoundationModels)
        @available(macOS 26.0, *)
        @MainActor
        private static func generateWithFoundationModels(diffSummary: String) async -> String? {
            do {
                let session = FoundationModels.LanguageModelSession()
                let prompt = """
                \(commitMessagePrompt())
                Changed files:

                \(diffSummary)
                """
                let response = try await session.respond(to: prompt)
                let text = response.content.trimmingCharacters(in: .whitespacesAndNewlines)
                return text.isEmpty ? nil : text
            } catch {
                return nil
            }
        }
    #endif

    func newChange(parent: String, message: String = "") {
        Task.detached { [repo] in
            do {
                try repo.newChange(parent: parent, message: message)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func abandon(rev: String) {
        Task.detached { [repo] in
            do {
                try repo.abandon(rev: rev)
                await MainActor.run { [weak self] in
                    self?.selectedChangeId = nil
                    self?.selectedChange = nil
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func squash(rev: String) {
        Task.detached { [repo] in
            do {
                try repo.squash(rev: rev, intoRev: nil)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func squash(rev: String, into destination: String) {
        Task.detached { [repo] in
            do {
                try repo.squash(rev: rev, intoRev: destination)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: destination)
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func edit(rev: String) {
        Task.detached { [repo] in
            do {
                try repo.edit(rev: rev)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: rev)
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func graft(rev: String) {
        Task.detached { [repo] in
            do {
                try repo.graft(rev: rev)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func merge(parents: [String]) {
        Task.detached { [repo] in
            do {
                try repo.merge(parentRevs: parents)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func duplicate(rev: String) {
        Task.detached { [repo] in
            do {
                try repo.duplicate(rev: rev)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func gitFetch() {
        Task.detached { [repo] in
            do {
                let msg = try repo.gitFetch(remote: "origin")
                await MainActor.run { [weak self] in
                    self?.info = msg
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func gitPush(bookmark: String = "") {
        Task.detached { [repo] in
            do {
                let msg = try repo.gitPush(bookmark: bookmark)
                await MainActor.run { [weak self] in
                    self?.info = msg
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func createBookmark(name: String, rev: String = "@") {
        Task.detached { [repo] in
            do {
                try repo.createBookmark(name: name, rev: rev)
                await MainActor.run { [weak self] in
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func moveBookmarkForward(name: String) {
        Task.detached { [repo] in
            do {
                try repo.moveBookmark(name: name, toRev: "@-")
                await MainActor.run { [weak self] in
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func deleteBookmark(name: String) {
        Task.detached { [repo] in
            do {
                try repo.deleteBookmark(name: name)
                await MainActor.run { [weak self] in
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func renameBookmark(oldName: String, newName: String) {
        Task.detached { [repo] in
            do {
                try repo.renameBookmark(oldName: oldName, newName: newName)
                await MainActor.run { [weak self] in
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func trackBookmark(name: String) {
        Task.detached { [repo] in
            do {
                try repo.trackBookmark(name: name, remote: "origin")
                await MainActor.run { [weak self] in
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func restoreFiles(rev: String, paths: [String]) {
        Task.detached { [repo] in
            do {
                try repo.restoreFiles(rev: rev, paths: paths)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: rev)
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func deleteFiles(paths: [String]) {
        Task.detached { [repo] in
            do {
                try repo.deleteFiles(paths: paths)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func ignoreAndUntrack(paths: [String]) {
        Task.detached { [repo] in
            do {
                try repo.ignoreAndUntrack(paths: paths)
                await MainActor.run { [weak self] in
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func split(rev: String, paths: [String], message: String = "") {
        Task.detached { [repo] in
            do {
                try repo.split(rev: rev, paths: paths, message: message)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func opLog() {
        Task.detached { [repo] in
            do {
                let entries = try repo.opLog()
                await MainActor.run { [weak self] in
                    self?.opLogEntries = entries
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func opRestore(opId: String) {
        Task.detached { [repo] in
            do {
                try repo.opRestore(opId: opId)
                await MainActor.run { [weak self] in
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    private static func loadSelectedDetail(
        repo: JayJayRepo,
        log: [ChangeInfo],
        preferredRev: String?
    ) throws -> ChangeDetail? {
        var candidates = [String]()
        if let preferredRev, !preferredRev.isEmpty {
            candidates.append(preferredRev)
        }
        if let firstChange = log.first?.changeId, !candidates.contains(firstChange) {
            candidates.append(firstChange)
        }

        for candidate in candidates {
            guard let detail = try? repo.showSummary(rev: candidate) else { continue }
            if log.contains(where: { $0.changeId == detail.info.changeId }) {
                return detail
            }
        }

        return nil
    }
}
