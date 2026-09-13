import AppKit
import JayJayCore
import Observation

@MainActor
@Observable
final class DiffEditViewModel {
    let detail: ChangeDetail
    let repo: JayJayRepo?
    let diffStore: DiffStore
    let actions: (any ChangeActions)?
    let settings: AppSettings
    let onDone: () -> Void

    let core = JayJayCore.DiffEditSession()
    let fileSelectionByPath: [String: DiffEditFileSelectionState]

    var loadedFiles: [String: DiffEditFile] = [:]
    var newChangeMessage: String
    var showEmptySelectionAlert = false
    var bulkSelectionTask: Task<Void, Never>?
    var collapsedPaths: Set<String>
    var selectionSummary: String
    var shouldDeselect = false
    var isSelectingAll = false
    var focusedPath: String?
    var fileStats: [String: FileDiffStats] = [:]
    var isPreparingRemoval = false
    var removalTask: Task<Void, Never>?
    var applyLoadFailurePath: String?
    var applyStalePath: String?

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
        let paths = detail.diff.map(\.path)
        core.seedCollapse(
            paths: paths,
            totalChangedLines: diffStats.map { UInt64($0.insertions) + UInt64($0.deletions) }
        )
        collapsedPaths = Set(core.collapsedPaths(paths: paths))
        selectionSummary = core.summaryText()
    }

    var cardPaths: [String] {
        detail.diff.map(\.path)
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

    /// Pulls core's selection into the per-file observables; only the listed cards re-render.
    func refreshSelection(paths: [String]) {
        for path in paths {
            fileSelection(for: path).replace(with: Set(core.selectedLines(path: path).map(Int.init)))
        }
        selectionSummary = core.summaryText()
        shouldDeselect = core.shouldDeselect()
        isSelectingAll = core.isSelectingAll()
    }

    func refreshCollapse() {
        collapsedPaths = Set(core.collapsedPaths(paths: cardPaths))
    }

    func cancelTasks() {
        bulkSelectionTask?.cancel()
        removalTask?.cancel()
    }

    /// Loaded diffs and selections are full-diff row indices under one whitespace mode; a toggle silently remaps them, so the session resets both instead of letting apply submit old-mode indices.
    func whitespaceModeChanged() {
        bulkSelectionTask?.cancel()
        removalTask?.cancel()
        isPreparingRemoval = false
        core.unload()
        loadedFiles = [:]
        refreshSelection(paths: cardPaths)
        // Old-mode stats must not outlive the reset; the per-file pass rebuilds the folds from fresh stats.
        fileStats = [:]
    }

    func focusCard(path: String) {
        core.setFocused(path: path)
        focusedPath = path
    }

    func toggleCollapse(path: String) {
        core.toggleCollapse(path: path)
        refreshCollapse()
    }

    func expandAllFiles() {
        core.expandAll()
        refreshCollapse()
    }

    func collapseAllFiles() {
        core.collapseAll(paths: cardPaths)
        refreshCollapse()
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
        core.applyStats(paths: cardPaths, stats: stats)
        refreshCollapse()
    }

    func handleKey(_ event: NSEvent) -> Bool {
        // Keypad Enter and arrows always carry numericPad/function flags, so only reject real modifiers.
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        guard modifiers.subtracting([.numericPad, .function]).isEmpty else { return false }
        switch event.keyCode {
            case KeyCode.returnKey, KeyCode.keypadEnter:
                return withFocusedCard { toggleCollapse(path: $0) }
            case KeyCode.space:
                // Consumed even unfocused; falling through would toggle the file column's review mark.
                _ = withFocusedCard { toggleFileSelection(path: $0) }
                return true
            case KeyCode.leftArrow:
                return setFocusedCollapsed(true)
            case KeyCode.rightArrow:
                return setFocusedCollapsed(false)
            default:
                break
        }
        switch (event.keyCode, event.charactersIgnoringModifiers) {
            case (KeyCode.downArrow, _), (_, "j"):
                return moveFocus(forward: true)
            case (KeyCode.upArrow, _), (_, "k"):
                return moveFocus(forward: false)
            default:
                return false
        }
    }

    private func withFocusedCard(_ action: (String) -> Void) -> Bool {
        guard let focusedPath, cardPaths.contains(focusedPath) else { return false }
        action(focusedPath)
        return true
    }

    private func moveFocus(forward: Bool) -> Bool {
        focusedPath = core.moveFocus(paths: cardPaths, forward: forward)
        return true
    }

    private func setFocusedCollapsed(_ collapsed: Bool) -> Bool {
        guard let focusedPath, cardPaths.contains(focusedPath),
              core.setCollapsed(path: focusedPath, collapsed: collapsed)
        else { return false }
        refreshCollapse()
        return true
    }
}
