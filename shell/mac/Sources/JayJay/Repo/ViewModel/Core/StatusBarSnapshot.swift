import JayJayCore

/// Status-bar fields and the operation list a mutation path must refresh alongside the log, so neither shows the previous operation until the next full refresh.
struct StatusBarSnapshot {
    let workingCopyStats: DiffStats?
    let currentOperationDescription: String
    let undoState: UndoState
    let operations: RecentOperations

    static func load(from repo: JayJayRepo) -> StatusBarSnapshot {
        StatusBarSnapshot(
            workingCopyStats: try? repo.diffStats(rev: "@"),
            currentOperationDescription: repo.currentOperationDescription(),
            undoState: repo.undoState(),
            operations: repo.recentOperations()
        )
    }
}

extension RepoViewModel {
    func apply(_ snapshot: StatusBarSnapshot) {
        workingCopyStats = snapshot.workingCopyStats
        currentOperationDescription = snapshot.currentOperationDescription
        undoState = snapshot.undoState
        recentOperations = snapshot.operations
    }
}
