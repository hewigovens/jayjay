import Foundation
import JayJayCore
import JayJayDiffUI

struct DiffEditLoadedFile: Sendable {
    let file: DiffEditFile
    let supportsDiffEdit: Bool
    let display: FileDiff
    let displayToFull: [Int: Int]

    /// Selection-only callers pass highlight: false; tree-sitter setup costs tens of milliseconds per file.
    static func make(
        hunk: DiffHunk,
        oldContent: String?,
        newContent: String?,
        ignoreWhitespace: Bool,
        highlight: Bool = true
    ) async -> DiffEditLoadedFile {
        let path = hunk.path
        let old = oldContent ?? ""
        let new = newContent ?? ""
        let computed = await Task.detached {
            computeDiffEditFile(
                path: path,
                oldContent: old,
                newContent: new,
                ignoreWhitespace: ignoreWhitespace,
                highlight: highlight
            )
        }.value
        return DiffEditLoadedFile(
            file: DiffEditFile(
                path: path,
                oldPath: hunk.oldPath,
                hunkType: hunk.hunkType,
                oldContent: oldContent,
                newContent: newContent,
                changedLines: computed.changedLines
            ),
            supportsDiffEdit: hunk.projection == nil
                && hunk.hunkType != .renamed
                && DiffPlaceholder.isEditableText(oldContent)
                && DiffPlaceholder.isEditableText(newContent),
            display: computed.display,
            displayToFull: Dictionary(
                uniqueKeysWithValues: computed.displayToFull.map {
                    (Int($0.displayLine), Int($0.fullLine))
                }
            )
        )
    }
}

extension DiffEditFile {
    /// Content equality only: comparing diffs would false-trip between highlighted and plain variants.
    func hasSameSelectionBasis(as other: DiffEditFile) -> Bool {
        oldContent == other.oldContent && newContent == other.newContent
    }
}
