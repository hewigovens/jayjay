import Foundation
import JayJayBindings
#if canImport(FoundationModels)
import FoundationModels
#endif

@Observable
final class RepoViewModel: ChangeActions {
    let repoPath: String
    private(set) var graphEntries: [GraphEntry] = []
    var changes: [ChangeInfo] { graphEntries.map(\.change) }
    private(set) var selectedChange: ChangeDetail?
    private(set) var selectedChangeId: String?
    private(set) var bookmarks: [BookmarkInfo] = []
    private(set) var workingCopyDescription: String = ""
    private(set) var opLogEntries: [OpLogEntry] = []
    var error: String?
    var info: String?
    private(set) var isLoading = false

    var revset: String = "@ | ancestors(@, 20) | @-+"

    let repo: JayJayRepo

    private(set) var aiProvider: String = ""
    private var fsWatcher: RepoFSWatcher?

    init(path: String) throws {
        self.repoPath = path
        self.repo = try JayJayRepo.open(path: path)
        self.aiProvider = Self.detectAIProvider()
        self.fsWatcher = RepoFSWatcher(repoPath: path) { [weak self] in
            self?.refresh()
        }
    }

    private static func detectAIProvider() -> String {
        let candidates: [(String, [String])] = [
            ("Codex", ["\(NSHomeDirectory())/.local/bin/codex", "/opt/homebrew/bin/codex", "/usr/local/bin/codex"]),
            ("Claude", ["\(NSHomeDirectory())/.local/bin/claude", "/opt/homebrew/bin/claude", "/usr/local/bin/claude"]),
        ]
        for (name, paths) in candidates {
            if paths.contains(where: { FileManager.default.fileExists(atPath: $0) }) {
                return name
            }
        }
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
        isLoading = true
        error = nil
        let currentSelection = selectedChangeId
        Task.detached { [repo, revset] in
            do {
                try repo.refreshWorkingCopy()

                // Try the revset — if it fails, show empty list (not an error alert)
                let graph: [GraphEntry]
                do {
                    graph = try repo.logGraph(revset: revset).filter { Self.isVisibleChange($0.change) }
                } catch {
                    await MainActor.run { [weak self] in
                        self?.graphEntries = []
                        self?.selectedChange = nil
                        self?.selectedChangeId = nil
                        self?.isLoading = false
                    }
                    return
                }

                let log = graph.map(\.change)
                let marks = try repo.listBookmarks()
                let detail = try Self.loadSelectedDetail(
                    repo: repo,
                    log: log,
                    preferredRev: preferredRev ?? currentSelection
                )
                let wcDesc = log.first(where: { $0.isWorkingCopy })?.description ?? ""
                await MainActor.run { [weak self] in
                    self?.graphEntries = graph
                    self?.bookmarks = marks
                    self?.selectedChange = detail
                    self?.selectedChangeId = detail?.info.changeId
                    self?.workingCopyDescription = wcDesc
                    self?.isLoading = false
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
            let cliResult: String? = await Task.detached { [repo] in
                repo.generateCommitMessage(diffSummary: summary)
            }.value
            if let msg = cliResult, !msg.isEmpty {
                return msg
            }

            // 2. Fall back to Apple Foundation Models
            return await Self.generateWithLocalLLM(diffSummary: summary)
        } catch {
            await MainActor.run { [weak self] in
                self?.error = error.localizedDescription
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
            Generate a commit message for a version control commit. Format:
            Category: short summary sentence

            - Bullet point per meaningful change

            Categories: Add, Update, Fix, Refactor, Remove, Docs, Test, Chore.
            Keep the summary under 72 chars. Only output the message, no quotes or markdown fences.
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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
                    self?.error = error.localizedDescription
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

    private static func isVisibleChange(_ change: ChangeInfo) -> Bool {
        let trimmedDescription = change.description.trimmingCharacters(in: .whitespacesAndNewlines)
        let zeroCommitId = change.commitId.allSatisfy { $0 == "0" }
        let syntheticChangeId = change.changeId.allSatisfy { $0 == "z" }
        let hasNoParents = change.parents.isEmpty
        let isRootCommit = !change.isWorkingCopy &&
            trimmedDescription.isEmpty &&
            change.bookmarks.isEmpty &&
            (zeroCommitId || syntheticChangeId || hasNoParents)
        return !isRootCommit
    }
}
