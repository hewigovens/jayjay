@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class WorkingCopyFileEditorSessionTests: XCTestCase {
    func testLoadPreparesMarkdownHighlightingBeforePresentation() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-file-editor-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        try initJjGitRepo(path: directory.path)
        let content = "# JayJay\n\nEdit **safely**.\n"
        try content.write(to: directory.appending(path: "README.md"), atomically: true, encoding: .utf8)

        let repo = try JayJayRepo.open(path: directory.path)
        let session = WorkingCopyFileEditorSession(repo: repo, path: "README.md")
        await session.load()

        XCTAssertEqual(session.data?.content, content)
        XCTAssertEqual(session.content, content)
        XCTAssertFalse(session.isLoading)
        XCTAssertNil(session.errorMessage)
        XCTAssertTrue(
            session.highlightedLines.joined().contains { $0.token != .plain },
            "Markdown highlighting should be ready before the session is presented"
        )
    }

    func testFailedLoadNeverAppearsModified() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-file-editor-error-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        try initJjGitRepo(path: directory.path)

        let repo = try JayJayRepo.open(path: directory.path)
        let session = WorkingCopyFileEditorSession(repo: repo, path: "missing.md")
        await session.load()

        XCTAssertNil(session.data)
        XCTAssertNotNil(session.errorMessage)
        XCTAssertFalse(session.hasChanges)
    }
}
