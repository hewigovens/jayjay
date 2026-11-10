@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class ExternalMergeSessionTests: XCTestCase {
    func testTextSourceStartingWithPlaceholderPrefixReplacesResult() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-external-merge-prefix-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let left = directory.appending(path: "left")
        let base = directory.appending(path: "base")
        let right = directory.appending(path: "right")
        let output = directory.appending(path: "output")
        try Data("symlink -> left\n".utf8).write(to: left)
        try Data("symlink -> base\n".utf8).write(to: base)
        try Data("symlink -> right\n".utf8).write(to: right)
        try Data().write(to: output)
        let session = ExternalMergeSession(
            left: left.path,
            base: base.path,
            right: right.path,
            output: output.path,
            path: "value.txt",
            markerLength: 7
        )

        await session.load()
        session.useSource(.left)

        XCTAssertTrue(session.isTextMerge)
        XCTAssertEqual(session.result, "symlink -> left\n")
    }

    func testHunkPicksSwitchSidesAndUndoStepByStep() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-external-merge-undo-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let files = ["left", "base", "right"].map { directory.appending(path: $0) }
        for file in files {
            try Data("\(file.lastPathComponent)\n".utf8).write(to: file)
        }
        let output = directory.appending(path: "output")
        try Data().write(to: output)
        let session = ExternalMergeSession(
            left: files[0].path,
            base: files[1].path,
            right: files[2].path,
            output: output.path,
            path: "value.txt",
            markerLength: 7
        )
        await session.load()
        let original = session.result
        let hunk = try XCTUnwrap(session.highlights?.hunks.first?.hunk)
        let undoManager = UndoManager()
        undoManager.groupsByEvent = false
        func step(_ action: () -> Void) {
            undoManager.beginUndoGrouping()
            action()
            undoManager.endUndoGrouping()
        }

        step { XCTAssertTrue(session.useHunkSource(hunk, .left, undoManager: undoManager)) }
        step { XCTAssertTrue(session.useHunkSource(hunk, .right, undoManager: undoManager)) }
        XCTAssertEqual(session.result, "right\n", "a resolved hunk takes the other side")
        undoManager.undo()
        XCTAssertEqual(session.result, "left\n")
        XCTAssertEqual(session.picks, [hunk.index: .left])
        undoManager.undo()
        XCTAssertEqual(session.result, original)
        XCTAssertEqual(session.picks, [:])
        undoManager.redo()
        XCTAssertEqual(session.result, "left\n")

        session.result = "edited by hand\n"
        XCTAssertFalse(session.useHunkSource(hunk, .right, undoManager: undoManager))
        XCTAssertNotNil(session.actionError)
        XCTAssertNil(session.loadError, "a failed pick must keep the editor open")
        XCTAssertEqual(session.result, "edited by hand\n")

        session.resultChanged()
        session.actionError = nil
        XCTAssertFalse(session.useHunkSource(hunk, .right, undoManager: undoManager))
        XCTAssertNil(session.actionError, "a hand edit retires the picks, leaving no side that could only fail")
    }
}
