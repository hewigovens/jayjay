@testable import JayJay
import JayJayCore
import XCTest

final class DiffEditPrepareTests: XCTestCase {
    func testMatchingStrictReloadPreservesSelectionBasis() {
        let rendered = loaded(path: "a", old: "one\n", new: "two\n")
        let strict = loaded(path: "a", old: "one\n", new: "two\n")

        XCTAssertNil(DiffEditViewModel.firstStalePath(
            renderedByPath: ["a": rendered],
            strictByPath: ["a": strict],
            orderedPaths: ["a"]
        ))
    }

    func testChangedContentIsStale() {
        let rendered = loaded(path: "a", old: "one\n", new: "two\n")
        let strict = loaded(path: "a", old: "one\n", new: "three\n")

        XCTAssertEqual(DiffEditViewModel.firstStalePath(
            renderedByPath: ["a": rendered],
            strictByPath: ["a": strict],
            orderedPaths: ["a"]
        ), "a")
    }

    func testPreviouslyUnloadedFileCanUseStrictSnapshot() {
        let strict = loaded(path: "a", old: "one\n", new: "two\n")

        XCTAssertNil(DiffEditViewModel.firstStalePath(
            renderedByPath: [:],
            strictByPath: ["a": strict],
            orderedPaths: ["a"]
        ))
    }

    private func loaded(path: String, old: String, new: String) -> DiffEditFile {
        DiffEditFile(
            path: path,
            oldPath: nil,
            hunkType: .modified,
            oldContent: old,
            newContent: new,
            changedLines: []
        )
    }
}
