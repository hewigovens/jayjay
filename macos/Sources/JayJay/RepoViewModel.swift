import Foundation
import JayJayBindings

@Observable
final class RepoViewModel {
    let repoPath: String
    private(set) var changes: [ChangeInfo] = []
    private(set) var selectedChange: ChangeDetail?
    private(set) var selectedChangeId: String?
    private(set) var bookmarks: [BookmarkInfo] = []
    var error: String?
    private(set) var isLoading = false

    var revset: String = "@ | ancestors(@, 20)"

    private let repo: JayJayRepo

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
                let log = try repo.log(revset: revset)
                let marks = try repo.listBookmarks()
                let detail = try Self.loadSelectedDetail(
                    repo: repo,
                    log: log,
                    preferredRev: preferredRev ?? currentSelection
                )
                await MainActor.run { [weak self] in
                    self?.changes = log
                    self?.bookmarks = marks
                    self?.selectedChange = detail
                    self?.selectedChangeId = detail?.info.changeId
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
}
