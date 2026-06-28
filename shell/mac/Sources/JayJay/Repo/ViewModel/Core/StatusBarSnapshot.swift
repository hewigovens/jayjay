import JayJayCore

/// Status-bar fields a mutation path must refresh alongside the log, so the bar never shows the previous operation or stats until the next full refresh.
struct StatusBarSnapshot {
    let workingCopyStats: DiffStats?
    let currentOperationDescription: String

    static func load(from repo: JayJayRepo) -> StatusBarSnapshot {
        StatusBarSnapshot(
            workingCopyStats: try? repo.diffStats(rev: "@"),
            currentOperationDescription: repo.currentOperationDescription()
        )
    }
}

extension RepoViewModel {
    func apply(_ snapshot: StatusBarSnapshot) {
        workingCopyStats = snapshot.workingCopyStats
        currentOperationDescription = snapshot.currentOperationDescription
    }
}
