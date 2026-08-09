@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class ExternalMergeSessionTests: XCTestCase {
    func testGitMergeRequiresEveryConflictToBeResolvedBeforeSave() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-git-merge-save-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let left = directory.appending(path: "left")
        let base = directory.appending(path: "base")
        let right = directory.appending(path: "right")
        let output = directory.appending(path: "output")
        try Data("left\n".utf8).write(to: left)
        try Data("base\n".utf8).write(to: base)
        try Data("right\n".utf8).write(to: right)
        try Data().write(to: output)
        let generated = try loadExternalMerge(
            left: left.path,
            base: base.path,
            right: right.path,
            output: output.path,
            outputIsInitialized: false,
            markerLength: 7
        )
        try Data(generated.result.utf8).write(to: output)
        let session = ExternalMergeSession(
            left: left.path,
            base: base.path,
            right: right.path,
            output: output.path,
            path: "value.txt",
            outputIsInitialized: true,
            markerLength: 7
        )

        await session.load()

        XCTAssertGreaterThan(session.unresolvedCount, 0)
        XCTAssertFalse(session.canSave)
        session.result = session.result
            .split(separator: "\n", omittingEmptySubsequences: false)
            .filter { !$0.hasPrefix("<<<<<<<") }
            .joined(separator: "\n")
        XCTAssertEqual(session.unresolvedCount, 0)
        XCTAssertFalse(session.canSave)
        session.result = "Markdown heading\n=======\nbody\n"
        XCTAssertEqual(session.unresolvedCount, 0)
        XCTAssertTrue(session.canSave)
        session.useSource(.left)
        XCTAssertTrue(session.canSave)
    }

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
            outputIsInitialized: false,
            markerLength: 7
        )

        await session.load()
        session.useSource(.left)

        XCTAssertTrue(session.isTextMerge)
        XCTAssertEqual(session.result, "symlink -> left\n")
    }
}
