@testable import JayJay
import JayJayCore
import XCTest

final class DiffEditSelectionTests: XCTestCase {
    func testBuildSelectionsIncludesEveryLoadedFile() {
        let first = loadedFile(path: "a.txt", content: "one\n")
        let second = loadedFile(path: "b.txt", content: "two\n")
        let loadedFiles = [
            first.hunk.path: first,
            second.hunk.path: second
        ]
        let selected = loadedFiles.mapValues(\.changedLineSet)

        let selections = buildDiffEditSelections(
            loadedFiles: loadedFiles,
            selectedChangedLinesByPath: selected,
            destination: .newChild
        )

        XCTAssertEqual(Set(selections.map(\.path)), ["a.txt", "b.txt"])
    }

    private func loadedFile(path: String, content: String) -> DiffEditLoadedFile {
        let hunk = DiffHunk(
            path: path,
            oldPath: nil,
            oldContent: "",
            newContent: content,
            oldPreview: nil,
            newPreview: nil,
            hunkType: .added,
            reviewIdentity: "identity-\(path)"
        )
        let diff = FileDiff(
            path: path,
            language: "text",
            lines: [
                DiffLine(
                    oldLineNo: nil,
                    newLineNo: 1,
                    style: .added,
                    spans: [DiffSpan(text: content.trimmingCharacters(in: .newlines), style: .added, token: .plain)],
                    noEofNewline: false
                )
            ],
            whitespaceOnlyHidden: false
        )
        return DiffEditLoadedFile(hunk: hunk, oldContent: "", newContent: content, diff: diff)
    }
}
