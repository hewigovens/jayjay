@testable import JayJay
import XCTest

@MainActor
final class ExternalDiffSessionTests: XCTestCase {
    func testBinaryFileUsesWholeFileSelection() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-external-binary-tests-\(UUID().uuidString)")
        let left = directory.appending(path: "left")
        let right = directory.appending(path: "right")
        try FileManager.default.createDirectory(at: left, withIntermediateDirectories: true)
        try FileManager.default.createDirectory(at: right, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        try Data([0, 1, 2]).write(to: left.appending(path: "data.bin"))
        try Data([0, 3, 4]).write(to: right.appending(path: "data.bin"))
        let session = ExternalDiffSession(left: left.path, right: right.path, editable: true)

        await session.load()

        let file = try XCTUnwrap(session.files.first)
        XCTAssertFalse(file.supportsEditing)
        XCTAssertEqual(file.selection?.wholeFileSide, .new)
        session.toggleFile(file)
        XCTAssertEqual(file.selection?.wholeFileSide, .old)
    }

    func testTopologyTransitionTogglesAsOneGroup() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-external-topology-tests-\(UUID().uuidString)")
        let left = directory.appending(path: "left")
        let right = directory.appending(path: "right")
        try FileManager.default.createDirectory(at: left, withIntermediateDirectories: true)
        try FileManager.default.createDirectory(
            at: right.appending(path: "item"),
            withIntermediateDirectories: true
        )
        defer { try? FileManager.default.removeItem(at: directory) }
        try Data("old file\n".utf8).write(to: left.appending(path: "item"))
        try Data("new file\n".utf8).write(to: right.appending(path: "item/new.txt"))
        let session = ExternalDiffSession(left: left.path, right: right.path, editable: true)

        await session.load()

        let parent = try XCTUnwrap(session.files.first { $0.hunk.path == "item" })
        let child = try XCTUnwrap(session.files.first { $0.hunk.path == "item/new.txt" })
        XCTAssertEqual(parent.topologyGroup, child.topologyGroup)
        session.toggleFile(parent)
        XCTAssertTrue(parent.selectedExists)
        XCTAssertFalse(child.selectedExists)
    }

    func testLineSelectionUpdatesAddedAndDeletedFileExistence() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-external-existence-tests-\(UUID().uuidString)")
        let left = directory.appending(path: "left")
        let right = directory.appending(path: "right")
        try FileManager.default.createDirectory(at: left, withIntermediateDirectories: true)
        try FileManager.default.createDirectory(at: right, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let deletedURL = left.appending(path: "deleted.rs")
        try Data("fn deleted() {}\n".utf8).write(to: deletedURL)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: deletedURL.path)
        try Data("fn added() {}\n".utf8).write(to: right.appending(path: "added.rs"))
        let session = ExternalDiffSession(left: left.path, right: right.path, editable: true)

        await session.load()

        let added = try XCTUnwrap(session.files.first { $0.hunk.path == "added.rs" })
        try added.toggleDisplayLine(XCTUnwrap(added.displayToFull.keys.first))
        XCTAssertFalse(added.selectedExists)

        let deleted = try XCTUnwrap(session.files.first { $0.hunk.path == "deleted.rs" })
        try deleted.toggleDisplayLine(XCTUnwrap(deleted.displayToFull.keys.first))
        XCTAssertTrue(deleted.selectedExists)
        XCTAssertEqual(deleted.selection?.selectedExecutable, true)
    }

    func testFailedLoadDisablesSaving() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-external-diff-error-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let right = directory.appending(path: "right")
        try FileManager.default.createDirectory(at: right, withIntermediateDirectories: true)
        let exitState = ExternalToolExitState()
        let session = ExternalDiffSession(
            left: directory.appending(path: "missing-left").path,
            right: right.path,
            editable: true,
            onLoadFailure: exitState.markLoadFailure
        )

        await session.load()

        XCTAssertNotNil(session.errorMessage)
        XCTAssertFalse(session.canSave)
        XCTAssertEqual(
            exitState.exitCode(for: .diff(left: session.left, right: session.right, editable: true)),
            1
        )
    }
}
