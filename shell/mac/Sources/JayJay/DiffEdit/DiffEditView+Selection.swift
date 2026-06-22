import JayJayCore
import JayJayDiffUI
import SwiftUI

extension DiffEditView {
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

    func syncSelection(path: String, loaded: DiffEditLoadedFile) {
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
        let rev = detailRevision
        let commitId = detail.info.commitId.id
        let ignoreWhitespace = settings.ignoreWhitespace
        let diffStore = diffStore

        bulkSelectionTask = Task {
            var loadedByPath: [String: DiffEditLoadedFile] = [:]
            for hunk in hunks {
                if Task.isCancelled { return }
                guard let loaded = await loadEditableFile(
                    hunk: hunk,
                    rev: rev,
                    commitId: commitId,
                    repo: repo,
                    diffStore: diffStore,
                    ignoreWhitespace: ignoreWhitespace
                ) else {
                    continue
                }
                loadedByPath[hunk.path] = loaded
            }

            if Task.isCancelled { return }
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

    private func loadEditableFile(
        hunk: DiffHunk,
        rev: String,
        commitId: String?,
        repo: JayJayRepo,
        diffStore: DiffStore,
        ignoreWhitespace: Bool
    ) async -> DiffEditLoadedFile? {
        guard hunk.hunkType != .renamed,
              let cached = await diffStore.loadDiff(
                  hunk: hunk,
                  rev: rev,
                  commitId: commitId,
                  repo: repo,
                  ignoreWhitespace: ignoreWhitespace
              ),
              DiffPlaceholder.isEditableText(cached.oldContent),
              DiffPlaceholder.isEditableText(cached.newContent)
        else {
            return nil
        }

        let path = hunk.path
        let diff = await Task.detached {
            repo.computeNativeDiffFull(
                path: path,
                oldContent: cached.oldContent,
                newContent: cached.newContent,
                ignoreWhitespace: ignoreWhitespace
            )
        }.value
        let loaded = DiffEditLoadedFile(
            hunk: hunk,
            oldContent: cached.oldContent,
            newContent: cached.newContent,
            diff: diff
        )
        return loaded.changedLineSet.isEmpty ? nil : loaded
    }
}
