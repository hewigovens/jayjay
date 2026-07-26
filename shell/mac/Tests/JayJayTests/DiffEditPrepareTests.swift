@testable import JayJay
import JayJayCore
import XCTest

final class DiffEditPrepareTests: XCTestCase {
    func testMatchingStrictReloadPreservesSelectionBasis() {
        let rendered = loaded(path: "a", old: "one\n", new: "two\n")
        let strict = loaded(path: "a", old: "one\n", new: "two\n")

        XCTAssertNil(DiffEditSession.firstStalePath(
            renderedByPath: ["a": rendered],
            strictByPath: ["a": strict],
            orderedPaths: ["a"]
        ))
    }

    func testChangedContentIsStale() {
        let rendered = loaded(path: "a", old: "one\n", new: "two\n")
        let strict = loaded(path: "a", old: "one\n", new: "three\n")

        XCTAssertEqual(DiffEditSession.firstStalePath(
            renderedByPath: ["a": rendered],
            strictByPath: ["a": strict],
            orderedPaths: ["a"]
        ), "a")
    }

    func testDifferingDiffPresentationIsNotStaleForEqualContent() {
        let rendered = loaded(path: "a", old: "one\n", new: "two\n")
        let strict = loaded(path: "a", old: "one\n", new: "two\n", whitespaceOnlyHidden: true)

        XCTAssertNil(DiffEditSession.firstStalePath(
            renderedByPath: ["a": rendered],
            strictByPath: ["a": strict],
            orderedPaths: ["a"]
        ))
    }

    func testPreviouslyUnloadedFileCanUseStrictSnapshot() {
        let strict = loaded(path: "a", old: "one\n", new: "two\n")

        XCTAssertNil(DiffEditSession.firstStalePath(
            renderedByPath: [:],
            strictByPath: ["a": strict],
            orderedPaths: ["a"]
        ))
    }

    private func loaded(
        path: String,
        old: String,
        new: String,
        whitespaceOnlyHidden: Bool = false
    ) -> DiffEditLoadedFile {
        DiffEditLoadedFile(
            hunk: testHunk(
                path: path,
                oldContent: old,
                newContent: new,
                hunkType: .modified
            ),
            oldContent: old,
            newContent: new,
            diff: FileDiff(
                path: path,
                language: "",
                lines: [],
                whitespaceOnlyHidden: whitespaceOnlyHidden
            )
        )
    }
}
