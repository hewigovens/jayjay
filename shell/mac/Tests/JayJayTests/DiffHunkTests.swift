@testable import JayJay
import JayJayCore
import XCTest

final class DiffHunkTests: XCTestCase {
    private func hunk(
        type: HunkType = .renamed,
        old: String? = nil,
        new: String? = nil,
        oldPreview: DiffPreview? = nil,
        newPreview: DiffPreview? = nil
    ) -> DiffHunk {
        DiffHunk(
            path: "new/path", oldPath: "old/path",
            oldContent: old, newContent: new,
            oldPreview: oldPreview, newPreview: newPreview,
            hunkType: type, reviewIdentity: ""
        )
    }

    func testByteIdenticalRenameIsContentFree() {
        // The core clears both sides for a pure rename; this is what lets the UI hide the diff.
        XCTAssertTrue(hunk().isContentFreeRename)
    }

    func testRenameWithContentChangesShowsDiff() {
        XCTAssertFalse(hunk(old: "a\n", new: "b\n").isContentFreeRename)
    }

    func testImageRenameKeepsItsPreview() {
        // Regression guard: image renames carry previews and must still render, not be hidden.
        XCTAssertFalse(hunk(oldPreview: .image(path: "/tmp/x.png")).isContentFreeRename)
    }

    func testAddedFileIsNotARename() {
        XCTAssertFalse(hunk(type: .added, new: "a\n").isContentFreeRename)
    }
}
