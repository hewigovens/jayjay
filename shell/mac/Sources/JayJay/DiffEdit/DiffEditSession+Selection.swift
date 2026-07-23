import JayJayCore
import JayJayDiffUI

extension DiffEditSession {
    var selectionSummary: String {
        let selectedFiles = builtSelections().count
        let selectedLines = selectedChangedLinesByPath.reduce(into: 0) { count, entry in
            guard let loaded = loadedFiles[entry.key] else { return }
            count += loaded.changedLineCount(selectedLines: entry.value)
        }
        if selectedFiles == 0 {
            return "Select files, hunks, or line ranges to edit"
        }
        let fileLabel = selectedFiles == 1 ? "file" : "files"
        let lineLabel = selectedLines == 1 ? "line" : "lines"
        return "\(selectedFiles) \(fileLabel), \(selectedLines) \(lineLabel) selected"
    }

    var hasVisibleSelection: Bool {
        selectedChangedLinesByPath.values.contains { !$0.isEmpty }
    }

    var selectionToggleTitle: String {
        if isSelectingAll {
            return "Selecting..."
        }
        return shouldDeselectFromSelectionToggle ? "Deselect All" : "Select All"
    }

    var selectionToggleSystemImage: String {
        shouldDeselectFromSelectionToggle ? "square" : "checkmark.square"
    }

    var selectionToggleDisabled: Bool {
        if isSelectingAll {
            return true
        }
        return shouldDeselectFromSelectionToggle ? false : repo == nil
    }

    private var shouldDeselectFromSelectionToggle: Bool {
        hasVisibleSelection || selectsNewlyLoadedFiles
    }

    func builtSelections(for destination: DiffEditDestination = .newChild) -> [DiffEditFileSelection] {
        loadedFiles.compactMap { path, loaded in
            guard loaded.supportsDiffEdit else { return nil }
            let selectedLines = selectedChangedLinesByPath[path] ?? []
            if destination == .removeFromSource {
                return loaded.makeInverseSelection(selectedLines: selectedLines)
            }
            return loaded.makeSelection(selectedLines: selectedLines)
        }
    }

    func apply(_ destination: DiffEditDestination) {
        guard hasVisibleSelection else {
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
        let selections = builtSelections(for: destination)
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
        loadedFiles[path] = loaded
        syncSelection(path: path, loaded: loaded)
    }

    private func syncSelection(path: String, loaded: DiffEditLoadedFile) {
        let changedLines = loaded.changedLineSet
        guard loaded.supportsDiffEdit, !changedLines.isEmpty else {
            selectedChangedLinesByPath.removeValue(forKey: path)
            return
        }

        if let existing = selectedChangedLinesByPath[path] {
            selectedChangedLinesByPath[path] = existing.intersection(changedLines)
        } else if selectsNewlyLoadedFiles {
            selectedChangedLinesByPath[path] = changedLines
        } else {
            selectedChangedLinesByPath[path] = []
        }
    }

    func toggleFileSelection(path: String) {
        guard let loaded = loadedFiles[path], loaded.supportsDiffEdit else { return }
        let changedLines = loaded.changedLineSet
        let selected = selectedChangedLinesByPath[path] ?? []
        if changedLines.isSubset(of: selected) {
            selectedChangedLinesByPath[path] = []
        } else {
            selectedChangedLinesByPath[path] = changedLines
        }
    }

    func selectFile(path: String) {
        guard let loaded = loadedFiles[path], loaded.supportsDiffEdit else { return }
        selectedChangedLinesByPath[path] = loaded.changedLineSet
    }

    func toggleLineSelection(path: String, lineNumber: Int) {
        guard let loaded = loadedFiles[path], loaded.changedLineSet.contains(lineNumber) else { return }
        var selected = selectedChangedLinesByPath[path] ?? []
        if selected.contains(lineNumber) {
            selected.remove(lineNumber)
        } else {
            selected.insert(lineNumber)
        }
        selectedChangedLinesByPath[path] = selected
    }

    func selectHunk(path: String, range: ClosedRange<Int>) {
        guard let loaded = loadedFiles[path], loaded.supportsDiffEdit else { return }
        let changedLines = Set(loaded.changedLineNumbers.filter(range.contains))
        guard !changedLines.isEmpty else { return }
        var selected = selectedChangedLinesByPath[path] ?? []
        selected.formUnion(changedLines)
        selectedChangedLinesByPath[path] = selected
    }

    func toggleBulkSelection() {
        if shouldDeselectFromSelectionToggle {
            deselectAllChangedLines()
        } else {
            selectAllChangedLines()
        }
    }

    func selectAllChangedLines() {
        guard let repo else { return }
        bulkSelectionTask?.cancel()
        selectsNewlyLoadedFiles = true
        isSelectingAll = true

        let hunks = detail.diff
        let request = DiffEditLoadRequest(
            rev: sessionCommit,
            commitId: sessionCommit,
            ignoreWhitespace: settings.ignoreWhitespace
        )
        let diffStore = diffStore

        bulkSelectionTask = Task {
            var loadedByPath: [String: DiffEditLoadedFile] = [:]
            for hunk in hunks {
                if Task.isCancelled {
                    return
                }
                guard let loaded = await Self.loadEditableFile(
                    hunk: hunk,
                    request: request,
                    repo: repo,
                    diffStore: diffStore
                ) else {
                    continue
                }
                loadedByPath[hunk.path] = loaded
            }

            // The captured request outlives a whitespace-mode change; old-mode files and indices must not overwrite reloaded cards.
            if Task.isCancelled || settings.ignoreWhitespace != request.ignoreWhitespace {
                return
            }
            loadedFiles.merge(loadedByPath) { _, new in new }
            for loaded in loadedByPath.values {
                selectedChangedLinesByPath[loaded.hunk.path] = loaded.changedLineSet
            }
            isSelectingAll = false
        }
    }

    func deselectAllChangedLines() {
        bulkSelectionTask?.cancel()
        selectsNewlyLoadedFiles = false
        isSelectingAll = false
        selectedChangedLinesByPath = loadedFiles.reduce(into: [:]) { selections, entry in
            if entry.value.supportsDiffEdit {
                selections[entry.key] = []
            }
        }
    }

    private static func loadEditableFile(
        hunk: DiffHunk,
        request: DiffEditLoadRequest,
        repo: JayJayRepo,
        diffStore: DiffStore
    ) async -> DiffEditLoadedFile? {
        guard hunk.projection == nil,
              hunk.hunkType != .renamed,
              let cached = await diffStore.loadDiff(
                  hunk: hunk,
                  rev: request.rev,
                  commitId: request.commitId,
                  repo: repo,
                  ignoreWhitespace: request.ignoreWhitespace
              ),
              DiffPlaceholder.isEditableText(cached.content.oldContent),
              DiffPlaceholder.isEditableText(cached.content.newContent)
        else {
            return nil
        }

        let loaded = await DiffEditLoadedFile.make(
            hunk: hunk, oldContent: cached.content.oldContent, newContent: cached.content.newContent,
            repo: repo, ignoreWhitespace: request.ignoreWhitespace, highlight: false
        )
        return loaded.changedLineSet.isEmpty ? nil : loaded
    }
}

/// Revision context for bulk-loading editable files, bundled so the hot loop passes one value per hunk.
private struct DiffEditLoadRequest {
    let rev: String
    let commitId: String?
    let ignoreWhitespace: Bool
}
