@testable import JayJay
import XCTest

final class RepositoryActionsTests: XCTestCase {
    private var tempDir: URL!

    override func setUpWithError() throws {
        tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("jayjay-actions-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        if let tempDir {
            try? FileManager.default.removeItem(at: tempDir)
        }
        tempDir = nil
    }

    func testFileViewerSelectionURLUsesExistingFile() throws {
        let file = tempDir.appendingPathComponent("Sources/App.swift")
        try FileManager.default.createDirectory(
            at: file.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try "app".write(to: file, atomically: true, encoding: .utf8)

        XCTAssertEqual(
            RepositoryActions.fileViewerSelectionURL(repoPath: tempDir.path, path: "Sources/App.swift").path,
            file.path
        )
    }

    func testFileViewerSelectionURLFallsBackToExistingParent() throws {
        let directory = tempDir.appendingPathComponent("Sources")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)

        XCTAssertEqual(
            RepositoryActions.fileViewerSelectionURL(repoPath: tempDir.path, path: "Sources/Missing.swift").path,
            directory.path
        )
    }
}
