import Foundation
import JayJayCore

struct DiffEditLoadedFile {
    let hunk: DiffHunk
    let oldContent: String?
    let newContent: String?
    let diff: FileDiff

    var changedLineNumbers: [Int] {
        diffEditChangedLines(diff: diff).map(Int.init)
    }

    var changedLineSet: Set<Int> {
        Set(changedLineNumbers)
    }

    var supportsDiffEdit: Bool {
        diffEditSupportsFile(
            hunkType: hunk.hunkType,
            oldContent: effectiveOldContent,
            newContent: effectiveNewContent
        )
    }

    func changedLineCount(selectedLines: Set<Int>) -> Int {
        changedLineNumbers.filter(selectedLines.contains).count
    }

    func makeSelection(selectedLines: Set<Int>) -> DiffEditFileSelection? {
        buildDiffEditFileSelection(
            hunk: hunk,
            diff: diff,
            oldContent: effectiveOldContent,
            newContent: effectiveNewContent,
            selectedLines: selectedLines.map(UInt32.init).sorted(),
            inverse: false
        )
    }

    func makeInverseSelection(selectedLines: Set<Int>) -> DiffEditFileSelection? {
        buildDiffEditFileSelection(
            hunk: hunk,
            diff: diff,
            oldContent: effectiveOldContent,
            newContent: effectiveNewContent,
            selectedLines: selectedLines.map(UInt32.init).sorted(),
            inverse: true
        )
    }

    private var effectiveOldContent: String? {
        oldContent ?? hunk.oldContent
    }

    private var effectiveNewContent: String? {
        newContent ?? hunk.newContent
    }
}

func loadDiffEditFile(
    hunk: DiffHunk,
    rev: String,
    repo: JayJayRepo?,
    diffStore: DiffStore,
    ignoreWhitespace: Bool
) async -> DiffEditLoadedFile? {
    guard let repo else { return nil }

    let cached = await diffStore.loadDiff(
        hunk: hunk, rev: rev, repo: repo, ignoreWhitespace: ignoreWhitespace
    )
    let old = cached?.oldContent ?? hunk.oldContent
    let new = cached?.newContent ?? hunk.newContent
    let path = hunk.path

    let diff = await Task.detached {
        repo.computeNativeDiffFull(
            path: path,
            oldContent: old ?? "",
            newContent: new ?? "",
            ignoreWhitespace: ignoreWhitespace
        )
    }.value

    return DiffEditLoadedFile(hunk: hunk, oldContent: old, newContent: new, diff: diff)
}

func buildDiffEditSelections(
    loadedFiles: [String: DiffEditLoadedFile],
    selectedChangedLinesByPath: [String: Set<Int>],
    destination: DiffEditDestination
) -> [DiffEditFileSelection] {
    loadedFiles.compactMap { path, loaded in
        guard loaded.supportsDiffEdit else { return nil }
        let selectedLines = selectedChangedLinesByPath[path] ?? []
        if destination == .removeFromSource {
            return loaded.makeInverseSelection(selectedLines: selectedLines)
        }
        return loaded.makeSelection(selectedLines: selectedLines)
    }
}
