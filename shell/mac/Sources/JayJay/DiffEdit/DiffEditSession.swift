import AppKit
import JayJayCore
import Observation

/// Owns all Diff Edit state and jj-facing operations; DiffEditView renders it and forwards user intent.
@MainActor
@Observable
final class DiffEditSession {
    let detail: ChangeDetail
    let repo: JayJayRepo?
    let diffStore: DiffStore
    let actions: (any ChangeActions)?
    let settings: AppSettings
    let onDone: () -> Void
    let fileSelectionByPath: [String: DiffEditFileSelectionState]

    var loadedFiles: [String: DiffEditLoadedFile] = [:]
    var newChangeMessage: String
    var showEmptySelectionAlert = false
    var selectsNewlyLoadedFiles = false
    var isSelectingAll = false
    var bulkSelectionTask: Task<Void, Never>?
    var collapsedPaths: Set<String>
    var collapseTouched = false
    var fileStats: [String: FileDiffStats] = [:]
    var isPreparingRemoval = false
    var removalTask: Task<Void, Never>?
    var applyLoadFailurePath: String?
    var applyStalePath: String?
    var focusedPath: String?

    init(
        detail: ChangeDetail,
        repo: JayJayRepo?,
        diffStore: DiffStore,
        actions: (any ChangeActions)?,
        diffStats: DiffStats?,
        settings: AppSettings,
        onDone: @escaping () -> Void
    ) {
        self.detail = detail
        self.repo = repo
        self.diffStore = diffStore
        self.actions = actions
        self.settings = settings
        self.onDone = onDone
        fileSelectionByPath = Dictionary(uniqueKeysWithValues: detail.diff.map {
            ($0.path, DiffEditFileSelectionState())
        })
        newChangeMessage = detail.info.description
        // Seeded before the first frame from the whole-change stats so a large diff never flashes expanded while per-file stats compute; the per-file pass replaces this approximation with the precise policy.
        let fileCount = UInt64(detail.diff.count)
        let startsCollapsed: Bool = if let diffStats {
            diffEditStartsCollapsed(
                fileCount: fileCount,
                totalChangedLines: UInt64(diffStats.insertions) + UInt64(diffStats.deletions)
            )
        } else {
            diffEditCollapsesWhileStatsPending(fileCount: fileCount)
        }
        collapsedPaths = startsCollapsed ? Set(detail.diff.map(\.path)) : []
    }

    var detailRevision: String {
        detail.info.selectionRevision
    }

    /// Every session load resolves this immutable commit, never the floating revision, so mid-session drift fails the core staleness guard instead of being silently absorbed.
    var sessionCommit: String {
        detail.info.commitId.id
    }

    func fileSelection(for path: String) -> DiffEditFileSelectionState {
        guard let selection = fileSelectionByPath[path] else {
            preconditionFailure("Missing Diff Edit selection state for \(path)")
        }
        return selection
    }

    func cancelTasks() {
        bulkSelectionTask?.cancel()
        removalTask?.cancel()
    }

    /// Loaded diffs and selections are full-diff row indices under one whitespace mode; a toggle silently remaps them, so the session resets both instead of letting apply submit old-mode indices.
    func whitespaceModeChanged() {
        bulkSelectionTask?.cancel()
        removalTask?.cancel()
        isSelectingAll = false
        isPreparingRemoval = false
        selectsNewlyLoadedFiles = false
        loadedFiles = [:]
        for selection in fileSelectionByPath.values {
            selection.reset()
        }
        // Old-mode stats must not outlive the reset; the per-file pass rebuilds the folds from fresh stats.
        fileStats = [:]
    }

    func toggleCollapse(path: String) {
        collapseTouched = true
        if collapsedPaths.contains(path) {
            collapsedPaths.remove(path)
        } else {
            collapsedPaths.insert(path)
        }
    }

    func expandAllFiles() {
        collapseTouched = true
        collapsedPaths = []
    }

    func collapseAllFiles() {
        collapseTouched = true
        collapsedPaths = Set(detail.diff.map(\.path))
    }

    func loadFileStats() async {
        guard let repo else { return }
        let rev = sessionCommit
        let ignoreWhitespace = settings.ignoreWhitespace
        let stats = await Task.detached {
            try? repo.diffFileStats(rev: rev, ignoreWhitespace: ignoreWhitespace)
        }.value
        // The detached call outlives .task(id:) cancellation; a superseded mode's result must not overwrite the current one.
        guard let stats, !Task.isCancelled, settings.ignoreWhitespace == ignoreWhitespace
        else { return }
        fileStats = Dictionary(uniqueKeysWithValues: stats.map { ($0.path, $0) })
        guard !collapseTouched else { return }
        // Synthetic cards (dirty-only submodules) are absent from the jj tree diff; count them so the file threshold matches the cards on screen.
        var policyStats = stats
        let knownPaths = Set(stats.map(\.path))
        for hunk in detail.diff where !knownPaths.contains(hunk.path) {
            policyStats.append(FileDiffStats(path: hunk.path, insertions: 0, deletions: 0))
        }
        // The aggregate seed can overcount the displayed rows (placeholders, projections), so the precise pass recomputes the whole policy and replaces the seed outright.
        let total = policyStats.reduce(UInt64(0)) { $0 + UInt64($1.insertions) + UInt64($1.deletions) }
        collapsedPaths = diffEditStartsCollapsed(
            fileCount: UInt64(policyStats.count),
            totalChangedLines: total
        )
            ? Set(detail.diff.map(\.path))
            : Set(diffEditAutoCollapsedPaths(stats: policyStats))
    }

    nonisolated static func nextFocusedPath(current: String?, paths: [String], forward: Bool) -> String? {
        guard !paths.isEmpty else { return nil }
        guard let current, let index = paths.firstIndex(of: current) else {
            return forward ? paths.first : paths.last
        }
        let target = forward ? index + 1 : index - 1
        guard paths.indices.contains(target) else { return current }
        return paths[target]
    }

    func handleKey(_ event: NSEvent) -> Bool {
        // Keypad Enter and arrows always carry numericPad/function flags, so only reject real modifiers.
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        guard modifiers.subtracting([.numericPad, .function]).isEmpty else { return false }
        let paths = detail.diff.map(\.path)
        switch event.keyCode {
            case KeyCode.returnKey, KeyCode.keypadEnter:
                return withFocusedCard(in: paths) { toggleCollapse(path: $0) }
            case KeyCode.space:
                // Consumed even unfocused; falling through would toggle the file column's review mark.
                _ = withFocusedCard(in: paths) { toggleFileSelection(path: $0) }
                return true
            case KeyCode.leftArrow:
                return setFocusedCollapsed(true, paths: paths)
            case KeyCode.rightArrow:
                return setFocusedCollapsed(false, paths: paths)
            default:
                break
        }
        switch (event.keyCode, event.charactersIgnoringModifiers) {
            case (KeyCode.downArrow, _), (_, "j"):
                return moveFocus(forward: true, paths: paths)
            case (KeyCode.upArrow, _), (_, "k"):
                return moveFocus(forward: false, paths: paths)
            default:
                return false
        }
    }

    private func withFocusedCard(in paths: [String], _ action: (String) -> Void) -> Bool {
        guard let focusedPath, paths.contains(focusedPath) else { return false }
        action(focusedPath)
        return true
    }

    private func moveFocus(forward: Bool, paths: [String]) -> Bool {
        focusedPath = Self.nextFocusedPath(current: focusedPath, paths: paths, forward: forward)
        return true
    }

    private func setFocusedCollapsed(_ collapsed: Bool, paths: [String]) -> Bool {
        guard let focusedPath, paths.contains(focusedPath),
              collapsedPaths.contains(focusedPath) != collapsed
        else { return false }
        toggleCollapse(path: focusedPath)
        return true
    }
}
