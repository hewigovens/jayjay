import JayJayCore
import Observation

@MainActor
@Observable
final class RepoListViewModel {
    private(set) var groups = RepoListGroups(pinned: [], recent: [])
    private var pinned: [String] = []
    private var recents: [String] = []
    private var groupingTask: Task<Void, Never>?
    private var isStale = false

    /// Entries render flat until core has resolved their primary roots, which reads the filesystem.
    func show(pinned: [String], recents: [String]) {
        guard pinned != self.pinned || recents != self.recents else { return }
        self.pinned = pinned
        self.recents = recents
        let pinnedSet = Set(pinned)
        groups = RepoListGroups(
            pinned: pinned.map { RepoGroup(path: $0, workspaces: []) },
            recent: recents.filter { !pinnedSet.contains($0) }.map { RepoGroup(path: $0, workspaces: []) }
        )
        regroup()
    }

    /// A path can be replaced on disk while the list stays open, so activation regroups the same entries.
    func regroup() {
        guard groupingTask == nil else {
            isStale = true
            return
        }
        let (pinned, recents) = (pinned, recents)
        groupingTask = Task {
            let grouped = await Task.detached { repositoryListGroups(pinned: pinned, recents: recents) }.value
            if pinned == self.pinned, recents == self.recents {
                groups = grouped
            }
            groupingTask = nil
            if isStale {
                isStale = false
                regroup()
            }
        }
    }
}
