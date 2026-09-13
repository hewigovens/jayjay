import JayJayCore
import JayJayDiffUI

extension DiffEditViewModel {
    var selectionToggleTitle: String {
        if isSelectingAll {
            return "Selecting..."
        }
        return shouldDeselect ? "Deselect All" : "Select All"
    }

    var selectionToggleSystemImage: String {
        shouldDeselect ? "square" : "checkmark.square"
    }

    var selectionToggleDisabled: Bool {
        if isSelectingAll {
            return true
        }
        return shouldDeselect ? false : repo == nil
    }

    func apply(_ destination: DiffEditDestination) {
        guard core.hasSelection() else {
            showEmptySelectionAlert = true
            return
        }
        if destination == .removeFromSource {
            prepareRemoveFromSource()
            return
        }
        finishApply(destination)
    }

    func finishApply(_ destination: DiffEditDestination) {
        let selections = core.selections(paths: cardPaths, destination: destination)
        guard !selections.isEmpty else {
            showEmptySelectionAlert = true
            return
        }
        actions?.applyDiffSelection(
            rev: detailRevision,
            destination: destination,
            selections: selections,
            message: newChangeMessage,
            ignoreWhitespace: settings.ignoreWhitespace
        )
        onDone()
    }

    func fileLoaded(path: String, loaded: DiffEditLoadedFile) {
        guard loaded.supportsDiffEdit else {
            loadedFiles[path] = nil
            core.skip(path: path)
            refreshSelection(paths: [path])
            return
        }
        loadedFiles[path] = loaded.file
        core.load(file: loaded.file)
        refreshSelection(paths: [path])
    }

    func toggleFileSelection(path: String) {
        core.toggleFile(path: path)
        refreshSelection(paths: [path])
    }

    func selectFile(path: String) {
        core.selectFile(path: path)
        refreshSelection(paths: [path])
    }

    func toggleLineSelection(path: String, lineNumber: Int) {
        core.toggleLine(path: path, line: UInt32(lineNumber))
        refreshSelection(paths: [path])
    }

    func selectHunk(path: String, range: ClosedRange<Int>) {
        core.selectLines(path: path, lines: range.map(UInt32.init))
        refreshSelection(paths: [path])
    }

    func toggleBulkSelection() {
        let pending = core.toggleAll(paths: cardPaths)
        refreshSelection(paths: cardPaths)
        guard !pending.isEmpty else {
            bulkSelectionTask?.cancel()
            return
        }
        loadPendingSelection(paths: pending)
    }

    /// Loads what Select All is still waiting on; core claims each file the moment it registers.
    private func loadPendingSelection(paths: [String]) {
        guard let repo else { return }
        bulkSelectionTask?.cancel()
        let pending = Set(paths)
        let hunks = detail.diff.filter { pending.contains($0.path) }
        let rev = sessionCommit
        let ignoreWhitespace = settings.ignoreWhitespace
        let diffStore = diffStore

        bulkSelectionTask = Task {
            for hunk in hunks {
                if Task.isCancelled {
                    return
                }
                let loaded = await Self.loadEditableFile(
                    hunk: hunk,
                    rev: rev,
                    ignoreWhitespace: ignoreWhitespace,
                    repo: repo,
                    diffStore: diffStore
                )
                // The captured mode outlives a whitespace-mode change; old-mode files and indices must not overwrite reloaded cards.
                if Task.isCancelled || settings.ignoreWhitespace != ignoreWhitespace {
                    return
                }
                if let loaded {
                    loadedFiles[hunk.path] = loaded
                    core.load(file: loaded)
                } else {
                    core.skip(path: hunk.path)
                }
                refreshSelection(paths: [hunk.path])
            }
        }
    }

    private static func loadEditableFile(
        hunk: DiffHunk,
        rev: String,
        ignoreWhitespace: Bool,
        repo: JayJayRepo,
        diffStore: DiffStore
    ) async -> DiffEditFile? {
        guard hunk.projection == nil,
              hunk.hunkType != .renamed,
              let cached = await diffStore.loadDiff(
                  hunk: hunk,
                  rev: rev,
                  commitId: rev,
                  repo: repo,
                  ignoreWhitespace: ignoreWhitespace
              )
        else {
            return nil
        }
        let loaded = await DiffEditLoadedFile.make(
            hunk: hunk,
            oldContent: cached.content.oldContent,
            newContent: cached.content.newContent,
            ignoreWhitespace: ignoreWhitespace,
            highlight: false
        )
        return loaded.supportsDiffEdit && !loaded.file.changedLines.isEmpty ? loaded.file : nil
    }
}
