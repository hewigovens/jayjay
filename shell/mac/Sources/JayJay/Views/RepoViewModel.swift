import Foundation
import JayJayBindings
#if canImport(FoundationModels)
import FoundationModels
#endif

@Observable
final class RepoViewModel {
    let repoPath: String
    private(set) var graphEntries: [GraphEntry] = []
    var changes: [ChangeInfo] { graphEntries.map(\.change) }
    private(set) var selectedChange: ChangeDetail?
    private(set) var selectedChangeId: String?
    private(set) var bookmarks: [BookmarkInfo] = []
    private(set) var workingCopyDescription: String = ""
    var error: String?
    private(set) var isLoading = false

    var revset: String = "@ | ancestors(@, 20)"

    let repo: JayJayRepo

    init(path: String) throws {
        self.repoPath = path
        self.repo = try JayJayRepo.open(path: path)
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
                let graph = try repo.logGraph(revset: revset).filter { Self.isVisibleChange($0.change) }
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
                try repo.refreshWorkingCopy()
                let detail = try repo.show(rev: changeId)
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
            Generate a concise commit message (1-2 lines max) for a version control commit. \
            Only output the message, no quotes or prefixes. \
            Based on these changed files:

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
                try repo.gitFetch(remote: "origin")
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

    func gitPush() {
        Task.detached { [repo] in
            do {
                try repo.gitPush(bookmark: "")
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

    func createBookmark(name: String) {
        Task.detached { [repo] in
            do {
                try repo.createBookmark(name: name, rev: "@")
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

    func split(rev: String, paths: [String]) {
        Task.detached { [repo] in
            do {
                try repo.split(rev: rev, paths: paths)
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
            let detail = try repo.show(rev: candidate)
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
